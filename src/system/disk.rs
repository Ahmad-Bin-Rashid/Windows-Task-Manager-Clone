//! Disk I/O statistics using Win32 APIs
//!
//! This module provides functions to query per-process disk I/O
//! using GetProcessIoCounters.

use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{
    GetProcessIoCounters, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    IO_COUNTERS,
};

/// Per-process disk I/O statistics
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct ProcessDiskInfo {
    /// Total bytes read from disk
    pub read_bytes: u64,
    /// Total bytes written to disk
    pub write_bytes: u64,
    /// Number of read operations
    pub read_ops: u64,
    /// Number of write operations
    pub write_ops: u64,
}

impl ProcessDiskInfo {
    /// Returns total I/O (read + write) in bytes
    #[allow(dead_code)]
    pub fn total_io(&self) -> u64 {
        self.read_bytes + self.write_bytes
    }
}

/// Gets disk I/O information for a specific process.
///
/// Uses GetProcessIoCounters to retrieve read/write bytes and operation counts.
/// Gracefully returns default values if the process cannot be accessed.
///
/// # Arguments
/// * `pid` - The process ID to query
///
/// # Returns
/// * `ProcessDiskInfo` - Disk I/O statistics (zeros if access denied)
pub fn get_process_disk_info(pid: u32) -> ProcessDiskInfo {
    // Try to open the process with limited query rights
    // SAFETY: OpenProcess is safe to call with valid parameters.
    let handle = unsafe {
        OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
    };

    let handle = match handle {
        Ok(h) => h,
        Err(_) => return ProcessDiskInfo::default(), // Access denied or process gone
    };

    let mut io_counters = IO_COUNTERS::default();

    // SAFETY: GetProcessIoCounters is safe with a valid handle and initialized struct.
    let result = unsafe { GetProcessIoCounters(handle, &mut io_counters) };

    // Always close the handle
    // SAFETY: We own this handle and it's valid.
    unsafe {
        let _ = CloseHandle(handle);
    }

    if result.is_ok() {
        ProcessDiskInfo {
            read_bytes: io_counters.ReadTransferCount,
            write_bytes: io_counters.WriteTransferCount,
            read_ops: io_counters.ReadOperationCount,
            write_ops: io_counters.WriteOperationCount,
        }
    } else {
        ProcessDiskInfo::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_disk_info() {
        let pid = std::process::id();
        let info = get_process_disk_info(pid);
        // Our process should have done some I/O
        println!("Read: {} bytes, Write: {} bytes", info.read_bytes, info.write_bytes);
    }
}
