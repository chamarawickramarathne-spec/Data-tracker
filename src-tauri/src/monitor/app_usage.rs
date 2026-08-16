use std::collections::HashMap;
use std::ffi::c_void;
use std::mem;
use std::sync::Mutex;

use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::NetworkManagement::IpHelper::*;
use windows_sys::Win32::Networking::WinSock::AF_INET;
use windows_sys::Win32::System::Diagnostics::ToolHelp::*;

const ESTABLISHED: u32 = 5;

type ConnKey = (u32, u32, u32, u32, u32);

#[derive(Debug, Clone)]
pub struct AppUsageSample {
    pub app_name: String,
    pub upload_bytes: u64,
    pub download_bytes: u64,
}

pub struct AppUsageTracker {
    prev: Mutex<HashMap<ConnKey, (u32, u64, u64)>>,
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
        let cur = tcp_snapshot();
        let mut prev = self.prev.lock().unwrap();
        let mut pending = self.pending.lock().unwrap();

        for (key, &(pid, cur_in, cur_out)) in &cur {
            if let Some(&(_, prev_in, prev_out)) = prev.get(key) {
                let delta_in = cur_in.saturating_sub(prev_in);
                let delta_out = cur_out.saturating_sub(prev_out);
                if delta_in + delta_out > 0 {
                    let entry = pending.entry(pid).or_insert((0, 0));
                    entry.0 += delta_in;
                    entry.1 += delta_out;
                }
            }
        }
        *prev = cur;
    }

    pub fn flush(&self) -> Vec<AppUsageSample> {
        let pending = {
            let mut p = self.pending.lock().unwrap();
            let data = std::mem::take(&mut *p);
            data
        };
        if pending.is_empty() {
            return Vec::new();
        }

        let mut names = self.names.lock().unwrap();
        let live = process_names();
        for (pid, name) in &live {
            names.insert(*pid, name.clone());
        }

        pending
            .into_iter()
            .filter(|(pid, _)| *pid != 0)
            .map(|(pid, (up, down))| AppUsageSample {
                app_name: names.get(&pid).cloned().unwrap_or_else(|| "Unknown".to_string()),
                upload_bytes: up,
                download_bytes: down,
            })
            .filter(|s| s.upload_bytes + s.download_bytes > 0)
            .collect()
    }
}

fn tcp_snapshot() -> HashMap<ConnKey, (u32, u64, u64)> {
    let mut out = HashMap::new();

    let mut size: u32 = 0;
    unsafe {
        GetExtendedTcpTable(std::ptr::null_mut(), &mut size, 0, AF_INET as u32, TCP_TABLE_OWNER_PID_ALL, 0);
    }
    if size == 0 {
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
        return out;
    }

    let count = unsafe { (buf.as_ptr() as *const u32).read() } as usize;
    let row_size = mem::size_of::<MIB_TCPROW_OWNER_PID>();
    let table_start = unsafe { buf.as_ptr().add(mem::size_of::<u32>()) };

    for i in 0..count {
        let row_ptr = unsafe { table_start.add(i * row_size) as *const MIB_TCPROW_OWNER_PID };
        let row = unsafe { &*row_ptr };
        if row.dwState != ESTABLISHED || row.dwOwningPid == 0 {
            continue;
        }

        let tcp_row = MIB_TCPROW_LH {
            Anonymous: MIB_TCPROW_LH_0 {
                dwState: row.dwState,
            },
            dwLocalAddr: row.dwLocalAddr,
            dwLocalPort: row.dwLocalPort,
            dwRemoteAddr: row.dwRemoteAddr,
            dwRemotePort: row.dwRemotePort,
        };

        if let Some((recv, send)) = tcp_byte_counters(&tcp_row) {
            let key = (
                row.dwOwningPid,
                row.dwLocalAddr,
                row.dwLocalPort,
                row.dwRemoteAddr,
                row.dwRemotePort,
            );
            out.insert(key, (row.dwOwningPid, recv, send));
        }
    }

    out
}

fn tcp_byte_counters(row: &MIB_TCPROW_LH) -> Option<(u64, u64)> {
    let rw = TCP_ESTATS_DATA_RW_v0 {
        EnableCollection: 1,
    };
    let rw_size = mem::size_of::<TCP_ESTATS_DATA_RW_v0>();
    unsafe {
        SetPerTcpConnectionEStats(
            row,
            TcpConnectionEstatsData,
            &rw as *const _ as *const u8,
            0,
            rw_size as u32,
            0,
        );
    }

    let mut rod: TCP_ESTATS_DATA_ROD_v0 = unsafe { mem::zeroed() };
    let rod_size = mem::size_of::<TCP_ESTATS_DATA_ROD_v0>();
    let ret = unsafe {
        GetPerTcpConnectionEStats(
            row,
            TcpConnectionEstatsData,
            std::ptr::null_mut(),
            0,
            0,
            std::ptr::null_mut(),
            0,
            0,
            &mut rod as *mut _ as *mut u8,
            0,
            rod_size as u32,
        )
    };
    if ret != 0 {
        return None;
    }
    Some((rod.DataBytesIn, rod.DataBytesOut))
}

fn process_names() -> HashMap<u32, String> {
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
