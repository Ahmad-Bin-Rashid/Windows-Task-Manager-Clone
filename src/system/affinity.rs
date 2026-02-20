//! CPU affinity management using Win32 APIs
//!
//! This module provides functions to get process CPU affinity,
//! which determines which CPU cores a process is allowed to use.

use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::SystemInformation::GetSystemInfo;
use windows::Win32::System::Threading::{
    GetProcessAffinityMask, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
};

/// CPU affinity information for a process
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CpuAffinity {
    /// Bitmask of cores the process can use
    pub process_mask: usize,
    /// Bitmask of cores available on the system
    pub system_mask: usize,
    /// Number of cores the process can use
    pub allowed_cores: u32,
    /// Total number of cores on the system
    pub total_cores: u32,
    /// List of allowed core indices (0-based)
    pub core_list: Vec<u32>,
}

impl CpuAffinity {
    /// Returns a formatted string describing the affinity
    pub fn format(&self) -> String {
        if self.allowed_cores == self.total_cores {
            format!("{}/{} cores (All cores)", self.allowed_cores, self.total_cores)
        } else if self.allowed_cores == 0 {
            "Unknown".to_string()
        } else {
            let cores_str = self.core_list
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}/{} cores (Cores: {})", self.allowed_cores, self.total_cores, cores_str)
        }
    }
}

/// Get the total number of logical processors (cores) on the system
pub fn get_system_core_count() -> u32 {
    unsafe {
        let mut sys_info = std::mem::zeroed();
        GetSystemInfo(&mut sys_info);
        sys_info.dwNumberOfProcessors
    }
}

/// Get CPU affinity information for a process
pub fn get_process_affinity(pid: u32) -> Option<CpuAffinity> {
    // Skip system processes
    if pid == 0 || pid == 4 {
        return None;
    }

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
        
        if handle.is_err() {
            return None;
        }
        
        let handle = handle.unwrap();
        if handle.is_invalid() {
            return None;
        }

        let mut process_mask: usize = 0;
        let mut system_mask: usize = 0;

        let result = GetProcessAffinityMask(
            handle,
            &mut process_mask,
            &mut system_mask,
        );

        let _ = CloseHandle(handle);

        if result.is_err() {
            return None;
        }

        // Count bits and build core list
        let total_cores = get_system_core_count();
        let mut allowed_cores = 0u32;
        let mut core_list = Vec::new();

        for i in 0..64 {
            if (process_mask >> i) & 1 == 1 {
                allowed_cores += 1;
                core_list.push(i as u32);
            }
        }

        Some(CpuAffinity {
            process_mask,
            system_mask,
            allowed_cores,
            total_cores,
            core_list,
        })
    }
}
