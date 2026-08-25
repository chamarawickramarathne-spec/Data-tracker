use std::collections::HashMap;
use std::ffi::c_void;
use std::mem;
use std::sync::Mutex;

use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::NetworkManagement::IpHelper::*;
use windows_sys::Win32::Networking::WinSock::*;
use windows_sys::Win32::System::Diagnostics::ToolHelp::*;

const ESTABLISHED: u32 = 5;

#[derive(Clone, PartialEq, Eq, Hash)]
enum ConnKey {
    V4 { pid: u32, la: u32, lp: u32, ra: u32, rp: u32 },
    V6 { pid: u32, la: [u8; 16], lp: u32, ra: [u8; 16], rp: u32 },
}

#[derive(Debug, Clone)]
pub struct AppUsageSample {
    pub app_name: String,
    pub upload_bytes: u64,
    pub download_bytes: u64,
}

#[repr(C)]
#[allow(dead_code)]
struct TcpEstatsDataRod {
    data_bytes_out: u64,
    data_segs_out: u64,
    data_bytes_in: u64,
    data_segs_in: u64,
    non_recov_da: u32,
    ul_ack_dups: u32,
    rto_o: u32,
    rto_r: u32,
    rto_s: u32,
    rto_t: u32,
}

pub struct AppUsageTracker {
    prev: Mutex<HashMap<ConnKey, (u64, u64)>>,
    pending: Mutex<HashMap<u32, (u64, u64)>>,
    names: Mutex<HashMap<u32, String>>,
}

impl AppUsageTracker {
    pub fn new() -> Self {
        Self {
            prev: Mutex::new(HashMap::new()),
            pending: Mutex::new(HashMap::new()),
            names: Mutex::new(HashMap::new()),
        }
    }

    pub fn capture(&self) {
        let mut cur = tcp_snapshot();
        let cur6 = tcp6_snapshot();
        cur.extend(cur6);

        let mut prev = self.prev.lock().unwrap();
        let mut pending = self.pending.lock().unwrap();

        for (key, &(pid, cur_in, cur_out)) in &cur {
            if let Some(&(prev_in, prev_out)) = prev.get(key) {
                let delta_in = cur_in.saturating_sub(prev_in);
                let delta_out = cur_out.saturating_sub(prev_out);
                if delta_in + delta_out > 0 {
                    let entry = pending.entry(pid).or_insert((0, 0));
                    entry.0 += delta_in;
                    entry.1 += delta_out;
                }
            }
        }

        *prev = cur.iter().map(|(k, &(_, bi, bo))| (k.clone(), (bi, bo))).collect();
        log::debug!("capture: {} connections, {} pending PIDs", cur.len(), pending.len());
    }

    pub fn flush(&self) -> Vec<AppUsageSample> {
        let pending = {
            let mut p = self.pending.lock().unwrap();
            std::mem::take(&mut *p)
        };
        if pending.is_empty() {
            log::debug!("flush: no pending per-app data");
            return Vec::new();
        }

        let mut names = self.names.lock().unwrap();
        let live = process_names();
        for (pid, name) in &live {
            names.insert(*pid, name.clone());
        }

        let samples: Vec<AppUsageSample> = pending
            .into_iter()
            .filter(|(pid, _)| *pid != 0)
            .map(|(pid, (in_bytes, out_bytes))| AppUsageSample {
                app_name: names.get(&pid).cloned().unwrap_or_else(|| "Unknown".to_string()),
                download_bytes: in_bytes,
                upload_bytes: out_bytes,
            })
            .filter(|s| s.upload_bytes + s.download_bytes > 0)
            .collect();

        log::debug!(
            "flush: {} samples, total {}",
            samples.len(),
            crate::monitor::aggregator::format_bytes(samples.iter().map(|s| s.upload_bytes + s.download_bytes).sum::<u64>())
        );
        samples
    }

    pub fn active_pid_counts(&self) -> Vec<(u32, usize)> {
        let prev = self.prev.lock().unwrap();
        let mut counts: HashMap<u32, usize> = HashMap::new();
        for key in prev.keys() {
            let pid = match key {
                ConnKey::V4 { pid, .. } | ConnKey::V6 { pid, .. } => *pid,
            };
            if pid != 0 {
                *counts.entry(pid).or_insert(0) += 1;
            }
        }
        counts.into_iter().collect()
    }
}

fn tcp_snapshot() -> HashMap<ConnKey, (u32, u64, u64)> {
    let mut out = HashMap::new();

    let mut size: u32 = 0;
    unsafe {
        GetExtendedTcpTable(std::ptr::null_mut(), &mut size, 0, AF_INET as u32, TCP_TABLE_OWNER_PID_ALL, 0);
    }
    if size == 0 {
        log::debug!("tcp_snapshot: no IPv4 TCP table");
        return out;
    }

    let mut buf = vec![0u8; size as usize];
    let ret = unsafe {
        GetExtendedTcpTable(
            buf.as_mut_ptr() as *mut c_void,
            &mut size,
            0,
            AF_INET as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        )
    };
    if ret != 0 {
        log::warn!("GetExtendedTcpTable(IPv4) failed: {ret}");
        return out;
    }

    let count = unsafe { (buf.as_ptr() as *const u32).read() } as usize;
    let row_size = mem::size_of::<MIB_TCPROW_OWNER_PID>();
    let table_start = unsafe { buf.as_ptr().add(mem::size_of::<u32>()) };

    let mut estats_ok = 0u32;
    let mut estats_skip = 0u32;

    for i in 0..count {
        let row_ptr = unsafe { table_start.add(i * row_size) as *const MIB_TCPROW_OWNER_PID };
        let row = unsafe { &*row_ptr };
        if row.dwState != ESTABLISHED || row.dwOwningPid == 0 {
            continue;
        }

        let tcp_row = MIB_TCPROW_LH {
            Anonymous: MIB_TCPROW_LH_0 { dwState: row.dwState },
            dwLocalAddr: row.dwLocalAddr,
            dwLocalPort: row.dwLocalPort,
            dwRemoteAddr: row.dwRemoteAddr,
            dwRemotePort: row.dwRemotePort,
        };

        if let Some((recv, send)) = tcp_byte_counters(&tcp_row) {
            out.insert(
                ConnKey::V4 { pid: row.dwOwningPid, la: row.dwLocalAddr, lp: row.dwLocalPort, ra: row.dwRemoteAddr, rp: row.dwRemotePort },
                (row.dwOwningPid, recv, send),
            );
            estats_ok += 1;
        } else {
            estats_skip += 1;
        }
    }

    log::debug!("tcp_snapshot: {count} IPv4 rows, {estats_ok} EStats ok, {estats_skip} EStats skipped");
    out
}

fn tcp6_snapshot() -> HashMap<ConnKey, (u32, u64, u64)> {
    let mut out = HashMap::new();

    let mut size: u32 = 0;
    unsafe {
        GetExtendedTcpTable(std::ptr::null_mut(), &mut size, 0, AF_INET6 as u32, TCP_TABLE_OWNER_PID_ALL, 0);
    }
    if size == 0 {
        log::debug!("tcp6_snapshot: no IPv6 TCP table");
        return out;
    }

    let mut buf = vec![0u8; size as usize];
    let ret = unsafe {
        GetExtendedTcpTable(
            buf.as_mut_ptr() as *mut c_void,
            &mut size,
            0,
            AF_INET6 as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        )
    };
    if ret != 0 {
        log::warn!("GetExtendedTcpTable(IPv6) failed: {ret}");
        return out;
    }

    let count = unsafe { (buf.as_ptr() as *const u32).read() } as usize;
    let row_size = mem::size_of::<MIB_TCP6ROW_OWNER_PID>();
    let table_start = unsafe { buf.as_ptr().add(mem::size_of::<u32>()) };

    let mut estats_ok = 0u32;
    let mut estats_skip = 0u32;

    for i in 0..count {
        let row_ptr = unsafe { table_start.add(i * row_size) as *const MIB_TCP6ROW_OWNER_PID };
        let row = unsafe { &*row_ptr };
        if row.dwState != ESTABLISHED || row.dwOwningPid == 0 {
            continue;
        }

        let tcp6_row = MIB_TCP6ROW {
            State: row.dwState as i32,
            LocalAddr: unsafe { mem::transmute(row.ucLocalAddr) },
            dwLocalScopeId: row.dwLocalScopeId,
            dwLocalPort: row.dwLocalPort,
            RemoteAddr: unsafe { mem::transmute(row.ucRemoteAddr) },
            dwRemoteScopeId: row.dwRemoteScopeId,
            dwRemotePort: row.dwRemotePort,
        };

        if let Some((recv, send)) = tcp6_byte_counters(&tcp6_row) {
            out.insert(
                ConnKey::V6 { pid: row.dwOwningPid, la: row.ucLocalAddr, lp: row.dwLocalPort, ra: row.ucRemoteAddr, rp: row.dwRemotePort },
                (row.dwOwningPid, recv, send),
            );
            estats_ok += 1;
        } else {
            estats_skip += 1;
        }
    }

    log::debug!("tcp6_snapshot: {count} IPv6 rows, {estats_ok} EStats ok, {estats_skip} EStats skipped");
    out
}

fn tcp_byte_counters(row: &MIB_TCPROW_LH) -> Option<(u64, u64)> {
    let rw = TCP_ESTATS_DATA_RW_v0 { EnableCollection: 1 };
    let rw_size = mem::size_of::<TCP_ESTATS_DATA_RW_v0>();
    let set_ret = unsafe {
        SetPerTcpConnectionEStats(row, TcpConnectionEstatsData, &rw as *const _ as *const u8, 0, rw_size as u32, 0)
    };
    if set_ret != 0 {
        log::trace!("SetPerTcpConnectionEStats failed: {set_ret}");
    }

    let mut rod: TcpEstatsDataRod = unsafe { mem::zeroed() };
    let rod_size = mem::size_of::<TcpEstatsDataRod>();
    let ret = unsafe {
        GetPerTcpConnectionEStats(
            row, TcpConnectionEstatsData,
            std::ptr::null_mut(), 0, 0,
            std::ptr::null_mut(), 0, 0,
            &mut rod as *mut _ as *mut u8, 0, rod_size as u32,
        )
    };
    if ret != 0 {
        log::trace!("GetPerTcpConnectionEStats failed: {ret}");
        return None;
    }
    Some((rod.data_bytes_in, rod.data_bytes_out))
}

fn tcp6_byte_counters(row: &MIB_TCP6ROW) -> Option<(u64, u64)> {
    let rw = TCP_ESTATS_DATA_RW_v0 { EnableCollection: 1 };
    let rw_size = mem::size_of::<TCP_ESTATS_DATA_RW_v0>();
    let set_ret = unsafe {
        SetPerTcp6ConnectionEStats(row, TcpConnectionEstatsData, &rw as *const _ as *const u8, 0, rw_size as u32, 0)
    };
    if set_ret != 0 {
        log::trace!("SetPerTcp6ConnectionEStats failed: {set_ret}");
    }

    let mut rod: TcpEstatsDataRod = unsafe { mem::zeroed() };
    let rod_size = mem::size_of::<TcpEstatsDataRod>();
    let ret = unsafe {
        GetPerTcp6ConnectionEStats(
            row, TcpConnectionEstatsData,
            std::ptr::null_mut(), 0, 0,
            std::ptr::null_mut(), 0, 0,
            &mut rod as *mut _ as *mut u8, 0, rod_size as u32,
        )
    };
    if ret != 0 {
        log::trace!("GetPerTcp6ConnectionEStats failed: {ret}");
        return None;
    }
    Some((rod.data_bytes_in, rod.data_bytes_out))
}

pub fn process_names() -> HashMap<u32, String> {
    let mut map = HashMap::new();
    let snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snap == INVALID_HANDLE_VALUE {
        return map;
    }

    let mut entry: PROCESSENTRY32W = unsafe { mem::zeroed() };
    entry.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;

    let mut ok = unsafe { Process32FirstW(snap, &mut entry) };
    while ok != 0 {
        let name = String::from_utf16_lossy(&entry.szExeFile);
        if let Some(end) = name.find('\0') {
            map.insert(entry.th32ProcessID, name[..end].to_string());
        } else {
            map.insert(entry.th32ProcessID, name);
        }
        ok = unsafe { Process32NextW(snap, &mut entry) };
    }

    unsafe {
        CloseHandle(snap);
    }
    map
}
