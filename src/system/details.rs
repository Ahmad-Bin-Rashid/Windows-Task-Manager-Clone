//! Detailed process information gathering
//!
//! Provides functions to retrieve in-depth process details including:
//! - Loaded modules/DLLs
//! - Command line arguments  
//! - Network connections

use std::ffi::OsString;
use std::mem;
use std::os::windows::ffi::OsStringExt;

use windows::Win32::Foundation::{CloseHandle, MAX_PATH, HMODULE};
use windows::Win32::System::ProcessStatus::{
    EnumProcessModules, GetModuleBaseNameW, GetModuleFileNameExW,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
};
use windows::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, GetExtendedUdpTable,
    TCP_TABLE_OWNER_PID_ALL, UDP_TABLE_OWNER_PID,
};
use windows::Win32::Networking::WinSock::AF_INET;

/// Information about a loaded module/DLL
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    /// Module name (e.g., "kernel32.dll")
    pub name: String,
    /// Full path to the module
    pub path: String,
    /// Base address in memory
    pub base_address: usize,
}

/// TCP connection information
#[derive(Debug, Clone)]
pub struct TcpConnectionInfo {
    /// Local IP address
    pub local_addr: String,
    /// Local port
    pub local_port: u16,
    /// Remote IP address
    pub remote_addr: String,
    /// Remote port
    pub remote_port: u16,
    /// Connection state
    pub state: String,
}

/// UDP endpoint information
#[derive(Debug, Clone)]
pub struct UdpEndpointInfo {
    /// Local IP address
    pub local_addr: String,
    /// Local port
    pub local_port: u16,
}

/// Complete process details
#[derive(Debug, Clone)]
pub struct ProcessDetails {
    /// Process ID
    pub pid: u32,
    /// Process name
    pub name: String,
    /// Full executable path
    pub path: Option<String>,
    /// Command line (if accessible)
    pub command_line: Option<String>,
    /// Loaded modules/DLLs
    pub modules: Vec<ModuleInfo>,
    /// TCP connections owned by this process
    pub tcp_connections: Vec<TcpConnectionInfo>,
    /// UDP endpoints owned by this process
    pub udp_endpoints: Vec<UdpEndpointInfo>,
    /// CPU percentage
    pub cpu_percent: f64,
    /// Memory in bytes
    pub memory_bytes: u64,
    /// Thread count
    pub thread_count: u32,
    /// Handle count
    pub handle_count: u32,
    /// Priority
    pub priority: String,
    /// Uptime
    pub uptime_seconds: u64,
    /// Disk read rate
    pub disk_read_rate: f64,
    /// Disk write rate
    pub disk_write_rate: f64,
}

/// Get loaded modules for a process
pub fn get_process_modules(pid: u32) -> Vec<ModuleInfo> {
    let mut modules = Vec::new();
    
    // Skip system processes
    if pid == 0 || pid == 4 {
        return modules;
    }

    unsafe {
        let handle = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            false,
            pid,
        );

        let handle = match handle {
            Ok(h) if !h.is_invalid() => h,
            _ => return modules,
        };

        // Get module handles
        let mut h_mods: [HMODULE; 1024] = [HMODULE::default(); 1024];
        let mut cb_needed: u32 = 0;

        let result = EnumProcessModules(
            handle,
            h_mods.as_mut_ptr(),
            (h_mods.len() * mem::size_of::<HMODULE>()) as u32,
            &mut cb_needed,
        );

        if result.is_ok() {
            let count = cb_needed as usize / mem::size_of::<HMODULE>();
            
            for i in 0..count.min(h_mods.len()) {
                let h_mod = h_mods[i];
                
                // Get module name
                let mut name_buf = [0u16; MAX_PATH as usize];
                let name_len = GetModuleBaseNameW(handle, h_mod, &mut name_buf);
                let name = if name_len > 0 {
                    OsString::from_wide(&name_buf[..name_len as usize])
                        .to_string_lossy()
                        .into_owned()
                } else {
                    continue;
                };

                // Get module path
                let mut path_buf = [0u16; MAX_PATH as usize];
                let path_len = GetModuleFileNameExW(handle, h_mod, &mut path_buf);
                let path = if path_len > 0 {
                    OsString::from_wide(&path_buf[..path_len as usize])
                        .to_string_lossy()
                        .into_owned()
                } else {
                    String::new()
                };

                modules.push(ModuleInfo {
                    name,
                    path,
                    base_address: h_mod.0 as usize,
                });
            }
        }

        let _ = CloseHandle(handle);
    }

    modules
}

/// Get TCP connections for a specific process
pub fn get_process_tcp_connections(pid: u32) -> Vec<TcpConnectionInfo> {
    let mut connections = Vec::new();

    unsafe {
        // First call to get required buffer size
        let mut size: u32 = 0;
        let _ = GetExtendedTcpTable(
            None,
            &mut size,
            false,
            AF_INET.0 as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );

        if size == 0 {
            return connections;
        }

        // Allocate buffer
        let mut buffer = vec![0u8; size as usize];
        
        let result = GetExtendedTcpTable(
            Some(buffer.as_mut_ptr() as *mut _),
            &mut size,
            false,
            AF_INET.0 as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );

        if result != 0 {
            return connections;
        }

        // Parse the table
        // MIB_TCPTABLE_OWNER_PID structure:
        // DWORD dwNumEntries
        // MIB_TCPROW_OWNER_PID table[ANY_SIZE]
        let num_entries = *(buffer.as_ptr() as *const u32);
        
        #[repr(C)]
        struct MibTcpRowOwnerPid {
            state: u32,
            local_addr: u32,
            local_port: u32,
            remote_addr: u32,
            remote_port: u32,
            owning_pid: u32,
        }

        let table_ptr = buffer.as_ptr().add(4) as *const MibTcpRowOwnerPid;
        
        for i in 0..num_entries as usize {
            let row = &*table_ptr.add(i);
            
            if row.owning_pid == pid {
                let state_str = match row.state {
                    1 => "CLOSED",
                    2 => "LISTEN",
                    3 => "SYN_SENT",
                    4 => "SYN_RCVD",
                    5 => "ESTABLISHED",
                    6 => "FIN_WAIT1",
                    7 => "FIN_WAIT2",
                    8 => "CLOSE_WAIT",
                    9 => "CLOSING",
                    10 => "LAST_ACK",
                    11 => "TIME_WAIT",
                    12 => "DELETE_TCB",
                    _ => "UNKNOWN",
                };

                connections.push(TcpConnectionInfo {
                    local_addr: format_ipv4(row.local_addr),
                    local_port: u16::from_be(row.local_port as u16),
                    remote_addr: format_ipv4(row.remote_addr),
                    remote_port: u16::from_be(row.remote_port as u16),
                    state: state_str.to_string(),
                });
            }
        }
    }

    connections
}

/// Get UDP endpoints for a specific process
pub fn get_process_udp_endpoints(pid: u32) -> Vec<UdpEndpointInfo> {
    let mut endpoints = Vec::new();

    unsafe {
        // First call to get required buffer size
        let mut size: u32 = 0;
        let _ = GetExtendedUdpTable(
            None,
            &mut size,
            false,
            AF_INET.0 as u32,
            UDP_TABLE_OWNER_PID,
            0,
        );

        if size == 0 {
            return endpoints;
        }

        // Allocate buffer
        let mut buffer = vec![0u8; size as usize];
        
        let result = GetExtendedUdpTable(
            Some(buffer.as_mut_ptr() as *mut _),
            &mut size,
            false,
            AF_INET.0 as u32,
            UDP_TABLE_OWNER_PID,
            0,
        );

        if result != 0 {
            return endpoints;
        }

        // Parse the table
        let num_entries = *(buffer.as_ptr() as *const u32);
        
        #[repr(C)]
        struct MibUdpRowOwnerPid {
            local_addr: u32,
            local_port: u32,
            owning_pid: u32,
        }

        let table_ptr = buffer.as_ptr().add(4) as *const MibUdpRowOwnerPid;
        
        for i in 0..num_entries as usize {
            let row = &*table_ptr.add(i);
            
            if row.owning_pid == pid {
                endpoints.push(UdpEndpointInfo {
                    local_addr: format_ipv4(row.local_addr),
                    local_port: u16::from_be(row.local_port as u16),
                });
            }
        }
    }

    endpoints
}

/// Format an IPv4 address from a u32
fn format_ipv4(addr: u32) -> String {
    let bytes = addr.to_le_bytes();
    format!("{}.{}.{}.{}", bytes[0], bytes[1], bytes[2], bytes[3])
}

/// Get command line for a process (simplified - returns path as fallback)
pub fn get_process_command_line(pid: u32) -> Option<String> {
    // Getting the actual command line requires reading the PEB from the process
    // which is complex. For now, we'll use the executable path.
    // A full implementation would use NtQueryInformationProcess with ProcessBasicInformation
    // then read RTL_USER_PROCESS_PARAMETERS from the PEB.
    crate::system::path::get_process_path(pid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_ipv4() {
        assert_eq!(format_ipv4(0x0100007F), "127.0.0.1"); // localhost
        assert_eq!(format_ipv4(0), "0.0.0.0");
    }

    #[test]
    fn test_get_current_process_modules() {
        let pid = std::process::id();
        let modules = get_process_modules(pid);
        // Should have at least the main executable
        assert!(!modules.is_empty());
        // Should have ntdll.dll or similar
        let has_system_dll = modules.iter().any(|m| 
            m.name.to_lowercase().contains("ntdll") || 
            m.name.to_lowercase().contains("kernel32")
        );
        assert!(has_system_dll);
    }
}
