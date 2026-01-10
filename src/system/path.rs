//! Process path and handle count using Win32 APIs
//!
//! This module provides functions to get the full executable path
//! and handle count for a process.

use windows::Win32::Foundation::{CloseHandle, MAX_PATH};
use windows::Win32::System::Threading::{
    GetProcessHandleCount, OpenProcess, QueryFullProcessImageNameW,
    PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};

/// Gets the full executable path for a process.
///
/// # Arguments
/// * `pid` - The process ID to query
///
/// # Returns
/// * `Option<String>` - Full path or None if inaccessible
pub fn get_process_path(pid: u32) -> Option<String> {
    // SAFETY: OpenProcess is safe with valid parameters
    let handle = unsafe {
        OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
    };

    let handle = match handle {
        Ok(h) => h,
        Err(_) => return None,
    };

    let mut buffer = [0u16; MAX_PATH as usize];
    let mut size = buffer.len() as u32;

    // SAFETY: QueryFullProcessImageNameW is safe with valid handle and buffer
    let result = unsafe {
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buffer.as_mut_ptr()),
            &mut size,
        )
    };

    // Always close the handle
    unsafe {
        let _ = CloseHandle(handle);
    }

    if result.is_ok() && size > 0 {
        Some(String::from_utf16_lossy(&buffer[..size as usize]))
    } else {
        None
    }
}

/// Gets the handle count for a process.
///
/// # Arguments
/// * `pid` - The process ID to query
///
/// # Returns
/// * `u32` - Number of handles, or 0 if inaccessible
pub fn get_process_handle_count(pid: u32) -> u32 {
    // SAFETY: OpenProcess is safe with valid parameters
    let handle = unsafe {
        OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
    };

    let handle = match handle {
        Ok(h) => h,
        Err(_) => return 0,
    };

    let mut count: u32 = 0;

    // SAFETY: GetProcessHandleCount is safe with valid handle and pointer
    let result = unsafe { GetProcessHandleCount(handle, &mut count) };

    // Always close the handle
    unsafe {
        let _ = CloseHandle(handle);
    }

    if result.is_ok() {
        count
    } else {
        0
    }
}

/// Extracts just the filename from a full path
pub fn path_to_filename(path: &str) -> &str {
    path.rsplit('\\').next().unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_process_path() {
        let pid = std::process::id();
        let path = get_process_path(pid);
        assert!(path.is_some(), "Should get path for current process");
        println!("Current process path: {:?}", path);
    }

    #[test]
    fn test_current_process_handles() {
        let pid = std::process::id();
        let count = get_process_handle_count(pid);
        assert!(count > 0, "Should have some handles");
        println!("Current process handles: {}", count);
    }
}
