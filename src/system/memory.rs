//! Memory information using Win32 APIs
//!
//! This module provides functions to query system-wide and per-process
//! memory usage using GlobalMemoryStatusEx and GetProcessMemoryInfo.

use std::mem;
use windows::Win32::System::ProcessStatus::{
    GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
};
use windows::Win32::System::SystemInformation::{
    GlobalMemoryStatusEx, MEMORYSTATUSEX,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
};
use windows::Win32::Foundation::CloseHandle;

/// System-wide memory statistics
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SystemMemoryInfo {
    /// Percentage of physical memory in use (0-100)
    pub memory_load_percent: u32,
    /// Total physical memory in bytes
    pub total_physical: u64,
    /// Available physical memory in bytes
    pub available_physical: u64,
    /// Total page file size in bytes
    pub total_page_file: u64,
    /// Available page file size in bytes
    pub available_page_file: u64,
    /// Total virtual memory in bytes
    pub total_virtual: u64,
    /// Available virtual memory in bytes
    pub available_virtual: u64,
}

impl SystemMemoryInfo {
    /// Returns the used physical memory in bytes.
    ///
    /// Calculated as `total_physical - available_physical`.
    pub fn used_physical(&self) -> u64 {
        self.total_physical - self.available_physical
    }

    /// Returns used memory as a formatted string.
    ///
    /// # Returns
    /// A string like "8.5 GB / 16.0 GB (53%)"
    #[allow(dead_code)]
    pub fn format_usage(&self) -> String {
        format!(
            "{:.1} GB / {:.1} GB ({:.0}%)",
            bytes_to_gb(self.used_physical()),
            bytes_to_gb(self.total_physical),
            self.memory_load_percent
        )
    }
}

/// Per-process memory statistics
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct ProcessMemoryInfo {
    /// Working set size in bytes (physical memory used)
    pub working_set: u64,
    /// Peak working set size in bytes
    pub peak_working_set: u64,
    /// Private bytes (committed memory)
    pub private_bytes: u64,
}

impl ProcessMemoryInfo {
    /// Returns working set as a formatted string.
    ///
    /// # Returns
    /// A string like "125.4 MB"
    #[allow(dead_code)]
    pub fn format_working_set(&self) -> String {
        format_bytes(self.working_set)
    }
}

/// Converts bytes to gigabytes
#[allow(dead_code)]
fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0 * 1024.0)
}

/// Formats bytes into a human-readable string
#[must_use]
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Gets system-wide memory information.
///
/// Uses GlobalMemoryStatusEx to retrieve total and available memory.
///
/// # Returns
/// * `Ok(SystemMemoryInfo)` - Memory statistics for the entire system
/// * `Err` - If the API call fails
#[must_use]
pub fn get_system_memory_info() -> windows::core::Result<SystemMemoryInfo> {
    // Initialize the struct - dwLength must be set!
    let mut mem_status = MEMORYSTATUSEX {
        dwLength: mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    
    // SAFETY: GlobalMemoryStatusEx is safe to call with a properly initialized struct.
    unsafe {
        GlobalMemoryStatusEx(&mut mem_status)?;
    }
    
    Ok(SystemMemoryInfo {
        memory_load_percent: mem_status.dwMemoryLoad,
        total_physical: mem_status.ullTotalPhys,
        available_physical: mem_status.ullAvailPhys,
        total_page_file: mem_status.ullTotalPageFile,
        available_page_file: mem_status.ullAvailPageFile,
        total_virtual: mem_status.ullTotalVirtual,
        available_virtual: mem_status.ullAvailVirtual,
    })
}

/// Gets memory information for a specific process.
///
/// Uses GetProcessMemoryInfo to retrieve working set and private bytes.
/// Gracefully returns default values if the process cannot be accessed.
///
/// # Arguments
/// * `pid` - The process ID to query
///
/// # Returns
/// * `ProcessMemoryInfo` - Memory statistics (zeros if access denied)
pub fn get_process_memory_info(pid: u32) -> ProcessMemoryInfo {
    // Try to open the process with limited query rights
    // SAFETY: OpenProcess is safe to call with valid parameters.
    let handle = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ,
            false,
            pid,
        )
    };
    
    let handle = match handle {
        Ok(h) => h,
        Err(_) => return ProcessMemoryInfo::default(), // Access denied or process gone
    };
    
    // Initialize the counters struct
    let mut counters = PROCESS_MEMORY_COUNTERS {
        cb: mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        ..Default::default()
    };
    
    // SAFETY: GetProcessMemoryInfo is safe with a valid handle and initialized struct.
    let result = unsafe {
        GetProcessMemoryInfo(
            handle,
            &mut counters,
            mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        )
    };
    
    // Always close the handle
    // SAFETY: We own this handle and it's valid.
    unsafe {
        let _ = CloseHandle(handle);
    }
    
    if result.is_ok() {
        ProcessMemoryInfo {
            working_set: counters.WorkingSetSize as u64,
            peak_working_set: counters.PeakWorkingSetSize as u64,
            private_bytes: counters.PagefileUsage as u64,
        }
    } else {
        ProcessMemoryInfo::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_system_memory_info() {
        let info = get_system_memory_info().expect("Should get system memory");
        assert!(info.total_physical > 0, "Should have some physical memory");
        assert!(info.memory_load_percent <= 100, "Load should be percentage");
    }
    
    #[test]
    fn test_process_memory_info() {
        let pid = std::process::id();
        let info = get_process_memory_info(pid);
        assert!(info.working_set > 0, "Our process should use some memory");
    }
    
    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1572864), "1.5 MB");
        assert_eq!(format_bytes(1610612736), "1.5 GB");
    }
}
