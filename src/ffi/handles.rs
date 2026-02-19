//! Safe RAII wrappers for Windows HANDLEs
//!
//! These wrappers ensure that handles are properly closed when they
//! go out of scope, preventing resource leaks.

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_ACCESS_RIGHTS};

/// A safe wrapper around a Windows process HANDLE.
/// Automatically closes the handle when dropped.
#[allow(dead_code)]
pub struct ProcessHandle(HANDLE);

#[allow(dead_code)]
impl ProcessHandle {
    /// Opens a process by PID with the specified access rights.
    ///
    /// # Arguments
    /// * `pid` - The process identifier
    /// * `access` - The access rights requested for the process
    ///
    /// # Returns
    /// * `Ok(ProcessHandle)` - A wrapped handle to the process
    /// * `Err` - If the process cannot be opened (access denied, process exited, etc.)
    pub fn open(pid: u32, access: PROCESS_ACCESS_RIGHTS) -> windows::core::Result<Self> {
        // SAFETY: OpenProcess is safe to call with valid parameters.
        // We handle the error case where the handle is invalid.
        let handle = unsafe { OpenProcess(access, false, pid)? };
        Ok(Self(handle))
    }

    /// Returns the raw HANDLE for use with Win32 APIs.
    /// 
    /// # Safety
    /// The caller must ensure the handle is not used after the ProcessHandle is dropped.
    pub fn as_raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        // SAFETY: We own this handle and it's valid (we got it from OpenProcess).
        // CloseHandle is safe to call on a valid handle exactly once.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

/// A safe wrapper around a ToolHelp32 snapshot HANDLE.
/// Automatically closes the handle when dropped.
pub struct SnapshotHandle(HANDLE);

impl SnapshotHandle {
    /// Creates a snapshot of all processes in the system.
    ///
    /// # Returns
    /// * `Ok(SnapshotHandle)` - A wrapped handle to the snapshot
    /// * `Err` - If the snapshot cannot be created
    pub fn create_process_snapshot() -> windows::core::Result<Self> {
        // SAFETY: CreateToolhelp32Snapshot is safe to call.
        // TH32CS_SNAPPROCESS captures all processes.
        // The second parameter (0) is ignored for process snapshots.
        let handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)? };
        Ok(Self(handle))
    }

    /// Returns the raw HANDLE for use with Win32 APIs.
    pub fn as_raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for SnapshotHandle {
    fn drop(&mut self) {
        // SAFETY: We own this handle and it's valid.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}
