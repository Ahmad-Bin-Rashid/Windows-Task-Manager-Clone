//! Admin/elevation status detection
//!
//! This module provides functions to detect whether the application
//! is running with elevated (administrator) privileges.

use std::mem;

use windows::Win32::Foundation::CloseHandle;
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

/// Checks if the current process is running with elevated (administrator) privileges.
///
/// This is important because many process operations require elevation:
/// - Killing system processes
/// - Changing priority of other users' processes
/// - Accessing memory info of protected processes
///
/// # Returns
/// * `true` if running as administrator
/// * `false` if running as standard user or if the check fails
///
/// # Example
/// ```no_run
/// if is_elevated() {
///     println!("Running as Administrator");
/// } else {
///     println!("Running as Standard User");
/// }
/// ```
#[must_use]
pub fn is_elevated() -> bool {
    // SAFETY: These Win32 API calls are safe when used correctly.
    // We properly handle the token and close it when done.
    unsafe {
        // Get the current process token
        let mut token_handle = windows::Win32::Foundation::HANDLE::default();
        
        let result = OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_QUERY,
            &mut token_handle,
        );

        if result.is_err() {
            return false;
        }

        // Query the elevation status
        let mut elevation = TOKEN_ELEVATION::default();
        let mut return_length: u32 = 0;
        let elevation_size = mem::size_of::<TOKEN_ELEVATION>() as u32;

        let result = GetTokenInformation(
            token_handle,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            elevation_size,
            &mut return_length,
        );

        // Always close the token handle
        let _ = CloseHandle(token_handle);

        // Check if the query succeeded and if we're elevated
        result.is_ok() && elevation.TokenIsElevated != 0
    }
}

/// Returns a display string for the current elevation status
#[must_use]
#[allow(dead_code)]
pub fn elevation_status_string() -> &'static str {
    if is_elevated() {
        "Administrator"
    } else {
        "Standard User"
    }
}

/// Returns a short indicator for the elevation status (for compact displays)
#[must_use]
#[allow(dead_code)]
pub fn elevation_indicator() -> &'static str {
    if is_elevated() {
        "[Admin]"
    } else {
        "[User]"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_elevated() {
        // This test just verifies the function runs without panicking
        // The actual result depends on how the test is run
        let _elevated = is_elevated();
    }

    #[test]
    fn test_elevation_status_string() {
        let status = elevation_status_string();
        assert!(status == "Administrator" || status == "Standard User");
    }
}
