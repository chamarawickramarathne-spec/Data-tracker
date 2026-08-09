use std::mem;
use windows_sys::Win32::NetworkManagement::IpHelper::*;
use windows_sys::Win32::NetworkManagement::Ndis::*;

#[derive(Debug, Clone)]
pub struct AdapterStats {
    pub name: String,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub speed: u64,
    pub is_connected: bool,
}

#[derive(Debug, Clone)]
pub struct AdapterInfo {
    pub name: String,
    pub description: String,
    pub is_connected: bool,
    pub speed: u64,
}

pub fn get_adapter_stats() -> Result<AdapterStats, String> {
    let mut table_ptr: *mut MIB_IF_TABLE2 = std::ptr::null_mut();

    let result = unsafe { GetIfTable2(&mut table_ptr) };
    if result != 0 {
        return Err(format!("GetIfTable2 failed with error: {}", result));
    }

    if table_ptr.is_null() {
        return Err("GetIfTable2 returned null".to_string());
    }

    let table = unsafe { &*table_ptr };
    let entry_size = mem::size_of::<MIB_IF_ROW2>();
    let table_start = unsafe { (table_ptr as *const u8).add(mem::size_of::<MIB_IF_TABLE2>()) };

    let mut best: Option<AdapterStats> = None;
    let mut max_traffic: u64 = 0;

    for i in 0..table.NumEntries {
        let row_ptr = unsafe {
            table_start.add(i as usize * entry_size) as *const MIB_IF_ROW2
        };
        let row = unsafe { &*row_ptr };

        let luid_val = unsafe { row.InterfaceLuid.Value };
        if luid_val == 0 {
            continue;
        }

        let is_connected = row.OperStatus == IfOperStatusUp;
        let total_traffic = row.InOctets + row.OutOctets;

        if is_connected && total_traffic > max_traffic {
            max_traffic = total_traffic;
            let name = unsafe { get_if_name(row) };

            best = Some(AdapterStats {
                name,
                bytes_received: row.InOctets,
                bytes_sent: row.OutOctets,
                speed: row.TransmitLinkSpeed,
                is_connected,
            });
        }
    }

    unsafe { FreeMibTable(table_ptr as *const _); }

    best.ok_or_else(|| "No active network adapter found".to_string())
}

pub fn get_network_adapters() -> Result<Vec<AdapterInfo>, String> {
    let mut table_ptr: *mut MIB_IF_TABLE2 = std::ptr::null_mut();

    let result = unsafe { GetIfTable2(&mut table_ptr) };
    if result != 0 {
        return Err(format!("GetIfTable2 failed: {}", result));
    }

    if table_ptr.is_null() {
        return Ok(Vec::new());
    }

    let table = unsafe { &*table_ptr };
    let entry_size = mem::size_of::<MIB_IF_ROW2>();
    let table_start = unsafe { (table_ptr as *const u8).add(mem::size_of::<MIB_IF_TABLE2>()) };

    let mut adapters = Vec::new();

    for i in 0..table.NumEntries {
        let row_ptr = unsafe {
            table_start.add(i as usize * entry_size) as *const MIB_IF_ROW2
        };
        let row = unsafe { &*row_ptr };

        let luid_val = unsafe { row.InterfaceLuid.Value };
        if luid_val == 0 {
            continue;
        }

        let name = unsafe { get_if_name(row) };
        let description = unsafe { get_if_description(row) };
        let is_connected = row.OperStatus == IfOperStatusUp;

        adapters.push(AdapterInfo {
            name,
            description,
            is_connected,
            speed: row.TransmitLinkSpeed,
        });
    }

    unsafe { FreeMibTable(table_ptr as *const _); }

    Ok(adapters)
}

unsafe fn get_if_name(row: &MIB_IF_ROW2) -> String {
    let name_wide: Vec<u16> = row.Alias.iter()
        .take_while(|&&c| c != 0)
        .cloned()
        .collect();
    String::from_utf16_lossy(&name_wide)
}

unsafe fn get_if_description(row: &MIB_IF_ROW2) -> String {
    let desc_wide: Vec<u16> = row.Description.iter()
        .take_while(|&&c| c != 0)
        .cloned()
        .collect();
    String::from_utf16_lossy(&desc_wide)
}
