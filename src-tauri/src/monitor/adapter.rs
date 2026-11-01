use std::mem;
use windows_sys::Win32::NetworkManagement::IpHelper::*;
use windows_sys::Win32::NetworkManagement::Ndis::*;

const IF_FLAG_HARDWARE_INTERFACE: u8 = 0x01;
const IF_FLAG_FILTER_INTERFACE: u8 = 0x02;
const IF_FLAG_ENDPOINT_INTERFACE: u8 = 0x80;
const DEFAULT_ROUTE_PROBE: u32 = 0x0808_0808;
const MAX_INTERFACE_ROWS: u32 = 4096;

#[derive(Debug, Clone)]
pub struct AdapterStats {
    pub name: String,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub receive_link_speed: u64,
    pub transmit_link_speed: u64,
    pub luid: u64,
}

#[derive(Debug, Clone)]
pub struct AdapterInfo {
    pub name: String,
    pub description: String,
    pub is_connected: bool,
    pub speed: u64,
}

struct MibTableGuard(*mut MIB_IF_TABLE2);

impl Drop for MibTableGuard {
    fn drop(&mut self) {
        unsafe { FreeMibTable(self.0 as *const _) };
    }
}

fn with_if_table<T>(f: impl FnOnce(&MIB_IF_TABLE2, *const u8) -> T) -> Result<T, String> {
    let mut table_ptr: *mut MIB_IF_TABLE2 = std::ptr::null_mut();
    let result = unsafe { GetIfTable2(&mut table_ptr) };
    if result != 0 {
        return Err(format!("GetIfTable2 failed with error: {result}"));
    }
    if table_ptr.is_null() {
        return Err("GetIfTable2 returned null".to_string());
    }

    let guard = MibTableGuard(table_ptr);
    let table = unsafe { &*guard.0 };
    if table.NumEntries > MAX_INTERFACE_ROWS {
        return Err(format!(
            "GetIfTable2 returned too many rows: {}",
            table.NumEntries
        ));
    }

    let row_offset = std::mem::offset_of!(MIB_IF_TABLE2, Table);
    let start = unsafe { (table_ptr as *const u8).add(row_offset) };
    Ok(f(table, start))
}

pub fn get_adapter_stats(cached_luid: Option<u64>) -> Result<AdapterStats, String> {
    if let Some(luid) = cached_luid {
        if let Some(stats) = get_adapter_by_luid(luid) {
            return Ok(stats);
        }
    }

    if let Some(index) = default_interface_index() {
        if let Some(stats) = get_adapter_by_index(index) {
            return Ok(stats);
        }
    }

    with_if_table(|table, start| select_best_adapter(table, start)).and_then(|stats| {
        stats.ok_or_else(|| "No active physical network adapter found".to_string())
    })
}

pub fn get_network_adapters() -> Result<Vec<AdapterInfo>, String> {
    with_if_table(|table, start| {
        let entry_size = mem::size_of::<MIB_IF_ROW2>();
        let mut adapters = Vec::new();

        for i in 0..table.NumEntries as usize {
            let row = unsafe { &*(start.add(i * entry_size) as *const MIB_IF_ROW2) };
            if !is_physical_interface(row) {
                continue;
            }

            adapters.push(AdapterInfo {
                name: unsafe { get_if_name(row) },
                description: unsafe { get_if_description(row) },
                is_connected: is_connected(row),
                speed: row.TransmitLinkSpeed,
            });
        }

        adapters
    })
}

fn get_adapter_by_luid(luid: u64) -> Option<AdapterStats> {
    let mut row: MIB_IF_ROW2 = unsafe { mem::zeroed() };
    row.InterfaceLuid.Value = luid;
    if unsafe { GetIfEntry2(&mut row) } != 0 {
        return None;
    }
    is_usable(&row).then(|| adapter_from_row(&row))
}

fn get_adapter_by_index(index: u32) -> Option<AdapterStats> {
    let mut row: MIB_IF_ROW2 = unsafe { mem::zeroed() };
    row.InterfaceIndex = index;
    if unsafe { GetIfEntry2(&mut row) } != 0 {
        return None;
    }
    is_usable(&row).then(|| adapter_from_row(&row))
}

fn default_interface_index() -> Option<u32> {
    let mut index = 0u32;
    let result = unsafe { GetBestInterface(DEFAULT_ROUTE_PROBE, &mut index) };
    (result == 0 && index != 0).then_some(index)
}

fn select_best_adapter(table: &MIB_IF_TABLE2, start: *const u8) -> Option<AdapterStats> {
    let entry_size = mem::size_of::<MIB_IF_ROW2>();
    let mut best: Option<&MIB_IF_ROW2> = None;

    for i in 0..table.NumEntries as usize {
        let row = unsafe { &*(start.add(i * entry_size) as *const MIB_IF_ROW2) };
        if !is_usable(row) {
            continue;
        }

        best = match best {
            None => Some(row),
            Some(current) => {
                let current_key = (
                    current.TransmitLinkSpeed.max(current.ReceiveLinkSpeed),
                    std::cmp::Reverse(current.InterfaceIndex),
                );
                let row_key = (
                    row.TransmitLinkSpeed.max(row.ReceiveLinkSpeed),
                    std::cmp::Reverse(row.InterfaceIndex),
                );
                (row_key > current_key).then_some(row).or(Some(current))
            }
        };
    }

    best.map(adapter_from_row)
}

fn is_physical_interface(row: &MIB_IF_ROW2) -> bool {
    let flags = row.InterfaceAndOperStatusFlags._bitfield;
    flags & IF_FLAG_HARDWARE_INTERFACE != 0
        && flags & IF_FLAG_FILTER_INTERFACE == 0
        && flags & IF_FLAG_ENDPOINT_INTERFACE == 0
        && unsafe { row.InterfaceLuid.Value } != 0
}

fn is_connected(row: &MIB_IF_ROW2) -> bool {
    row.OperStatus == IfOperStatusUp && row.MediaConnectState == MediaConnectStateConnected
}

fn is_usable(row: &MIB_IF_ROW2) -> bool {
    is_physical_interface(row) && is_connected(row)
}

fn adapter_from_row(row: &MIB_IF_ROW2) -> AdapterStats {
    AdapterStats {
        name: unsafe { get_if_name(row) },
        bytes_received: row.InOctets,
        bytes_sent: row.OutOctets,
        receive_link_speed: row.ReceiveLinkSpeed,
        transmit_link_speed: row.TransmitLinkSpeed,
        luid: unsafe { row.InterfaceLuid.Value },
    }
}

unsafe fn get_if_name(row: &MIB_IF_ROW2) -> String {
    let name_wide: Vec<u16> = row.Alias.iter().take_while(|&&c| c != 0).cloned().collect();
    String::from_utf16_lossy(&name_wide)
}

unsafe fn get_if_description(row: &MIB_IF_ROW2) -> String {
    let desc_wide: Vec<u16> = row
        .Description
        .iter()
        .take_while(|&&c| c != 0)
        .cloned()
        .collect();
    String::from_utf16_lossy(&desc_wide)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_rows_start_after_num_entries() {
        let offset = std::mem::offset_of!(MIB_IF_TABLE2, Table);
        assert_eq!(offset, std::mem::align_of::<MIB_IF_ROW2>());
        assert_eq!(
            std::mem::size_of::<MIB_IF_TABLE2>(),
            offset + std::mem::size_of::<MIB_IF_ROW2>()
        );
    }

    #[test]
    fn select_best_prefers_link_speed_then_lowest_index() {
        let mut fast: MIB_IF_ROW2 = unsafe { mem::zeroed() };
        fast.InterfaceLuid.Value = 1;
        fast.InterfaceIndex = 20;
        fast.InterfaceAndOperStatusFlags._bitfield = IF_FLAG_HARDWARE_INTERFACE;
        fast.OperStatus = IfOperStatusUp;
        fast.MediaConnectState = MediaConnectStateConnected;
        fast.ReceiveLinkSpeed = 1_000_000_000;
        fast.TransmitLinkSpeed = 1_000_000_000;
        set_alias(&mut fast, "fast");

        let mut slow: MIB_IF_ROW2 = fast;
        slow.InterfaceLuid.Value = 2;
        slow.InterfaceIndex = 10;
        slow.ReceiveLinkSpeed = 100_000_000;
        slow.TransmitLinkSpeed = 100_000_000;
        set_alias(&mut slow, "slow");

        let mut filter: MIB_IF_ROW2 = fast;
        filter.InterfaceLuid.Value = 3;
        filter.InterfaceIndex = 5;
        filter.InterfaceAndOperStatusFlags._bitfield = IF_FLAG_FILTER_INTERFACE;
        filter.ReceiveLinkSpeed = 10_000_000_000;
        filter.TransmitLinkSpeed = 10_000_000_000;
        set_alias(&mut filter, "filter");

        let rows = [slow, filter, fast];
        let mut table: MIB_IF_TABLE2 = unsafe { mem::zeroed() };
        table.NumEntries = rows.len() as u32;

        let selected = select_best_adapter(&table, rows.as_ptr() as *const u8).unwrap();
        assert_eq!(selected.name, "fast");
    }

    fn set_alias(row: &mut MIB_IF_ROW2, name: &str) {
        row.Alias = [0; 257];
        for (slot, unit) in row.Alias.iter_mut().zip(name.encode_utf16()) {
            *slot = unit;
        }
    }
}
