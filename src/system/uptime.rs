//! Process uptime tracking using Win32 APIs
//!
//! This module provides functions to get process creation time
//! and calculate uptime.

use windows::Win32::Foundation::{CloseHandle, FILETIME};
use windows::Win32::System::Threading::{
    GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
};

/// Converts a FILETIME to a u64 (100-nanosecond intervals since 1601)
fn filetime_to_u64(ft: &FILETIME) -> u64 {
    ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64)
}

/// Gets the current system time as FILETIME (100-nanosecond intervals since 1601)
pub fn get_current_filetime() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    // FILETIME epoch is January 1, 1601
    // Unix epoch is January 1, 1970
    // Difference is 11644473600 seconds
    const FILETIME_UNIX_DIFF: u64 = 11644473600;
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    
    // Convert to 100-nanosecond intervals and add the epoch difference
    (now.as_secs() + FILETIME_UNIX_DIFF) * 10_000_000 + (now.subsec_nanos() as u64 / 100)
}

/// Gets the creation time of a process as FILETIME
///
/// # Arguments
/// * `pid` - The process ID to query
///
/// # Returns
/// * `Option<u64>` - Creation time as FILETIME, or None if inaccessible
pub fn get_process_start_time(pid: u32) -> Option<u64> {
    // SAFETY: OpenProcess is safe with valid parameters
    let handle = unsafe {
        OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
    };

    let handle = match handle {
        Ok(h) => h,
        Err(_) => return None,
    };

    let mut creation_time = FILETIME::default();
    let mut exit_time = FILETIME::default();
    let mut kernel_time = FILETIME::default();
    let mut user_time = FILETIME::default();

    // SAFETY: GetProcessTimes is safe with valid handle and pointers
    let result = unsafe {
        GetProcessTimes(
            handle,
            &mut creation_time,
            &mut exit_time,
            &mut kernel_time,
            &mut user_time,
        )
    };

    // Always close the handle
    unsafe {
        let _ = CloseHandle(handle);
    }

    if result.is_ok() {
        Some(filetime_to_u64(&creation_time))
    } else {
        None
    }
}

/// Calculates process uptime in seconds
///
/// # Arguments
/// * `start_time` - Process creation time as FILETIME
///
/// # Returns
/// * `u64` - Uptime in seconds
pub fn calculate_uptime_seconds(start_time: u64) -> u64 {
    let now = get_current_filetime();
    if now > start_time {
        // Convert from 100-nanosecond intervals to seconds
        (now - start_time) / 10_000_000
    } else {
        0
    }
}

/// Formats uptime as a human-readable string
///
/// # Arguments
/// * `seconds` - Uptime in seconds
///
/// # Returns
/// * `String` - Formatted string like "5s", "2m", "1h 30m", "2d 5h"
pub fn format_uptime(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        let mins = seconds / 60;
        let secs = seconds % 60;
        if secs > 0 {
            format!("{}m {}s", mins, secs)
        } else {
            format!("{}m", mins)
        }
    } else if seconds < 86400 {
        let hours = seconds / 3600;
        let mins = (seconds % 3600) / 60;
        if mins > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}h", hours)
        }
    } else {
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        if hours > 0 {
            format!("{}d {}h", days, hours)
        } else {
            format!("{}d", days)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_uptime() {
        assert_eq!(format_uptime(30), "30s");
        assert_eq!(format_uptime(90), "1m 30s");
        assert_eq!(format_uptime(3660), "1h 1m");
        assert_eq!(format_uptime(90000), "1d 1h");
    }

    #[test]
    fn test_current_filetime() {
        let ft = get_current_filetime();
        assert!(ft > 0, "Should get valid filetime");
    }
}
