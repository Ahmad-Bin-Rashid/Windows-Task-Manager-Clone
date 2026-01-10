//! Process suspend and resume functionality
//!
//! Uses NtSuspendProcess and NtResumeProcess from ntdll.dll
//! to suspend and resume processes.

use std::collections::HashSet;
use std::sync::Mutex;

use windows::Win32::Foundation::{CloseHandle, NTSTATUS, STATUS_SUCCESS};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_SUSPEND_RESUME,
};

// We need to dynamically load NtSuspendProcess and NtResumeProcess from ntdll
use windows::core::PCSTR;
use windows::Win32::System::LibraryLoader::{GetProcAddress, GetModuleHandleA};
use windows::Win32::Foundation::HANDLE;

/// Type alias for NtSuspendProcess/NtResumeProcess function signature
type NtSuspendResumeProcess = unsafe extern "system" fn(HANDLE) -> NTSTATUS;

/// Global set of PIDs that we've suspended (to track state)
static SUSPENDED_PIDS: Mutex<Option<HashSet<u32>>> = Mutex::new(None);

/// Initialize the suspended PIDs set
fn init_suspended_pids() {
    let mut guard = SUSPENDED_PIDS.lock().unwrap();
    if guard.is_none() {
        *guard = Some(HashSet::new());
    }
}

/// Check if a process is suspended (by our tracking)
pub fn is_process_suspended(pid: u32) -> bool {
    init_suspended_pids();
    let guard = SUSPENDED_PIDS.lock().unwrap();
    guard.as_ref().map(|set| set.contains(&pid)).unwrap_or(false)
}

/// Mark a process as suspended in our tracking
fn mark_suspended(pid: u32) {
    init_suspended_pids();
    let mut guard = SUSPENDED_PIDS.lock().unwrap();
    if let Some(ref mut set) = *guard {
        set.insert(pid);
    }
}

/// Mark a process as resumed in our tracking
fn mark_resumed(pid: u32) {
    init_suspended_pids();
    let mut guard = SUSPENDED_PIDS.lock().unwrap();
    if let Some(ref mut set) = *guard {
        set.remove(&pid);
    }
}

/// Remove a PID from tracking (e.g., when process terminates)
pub fn untrack_process(pid: u32) {
    init_suspended_pids();
    let mut guard = SUSPENDED_PIDS.lock().unwrap();
    if let Some(ref mut set) = *guard {
        set.remove(&pid);
    }
}

/// Get the NtSuspendProcess function from ntdll
fn get_nt_suspend_process() -> Option<NtSuspendResumeProcess> {
    unsafe {
        let module = GetModuleHandleA(PCSTR(b"ntdll.dll\0".as_ptr())).ok()?;
        let proc = GetProcAddress(module, PCSTR(b"NtSuspendProcess\0".as_ptr()))?;
        Some(std::mem::transmute(proc))
    }
}

/// Get the NtResumeProcess function from ntdll
fn get_nt_resume_process() -> Option<NtSuspendResumeProcess> {
    unsafe {
        let module = GetModuleHandleA(PCSTR(b"ntdll.dll\0".as_ptr())).ok()?;
        let proc = GetProcAddress(module, PCSTR(b"NtResumeProcess\0".as_ptr()))?;
        Some(std::mem::transmute(proc))
    }
}

/// Suspend a process by PID
/// 
/// Returns Ok(()) on success, Err with message on failure
pub fn suspend_process(pid: u32) -> Result<(), String> {
    // Don't suspend system processes
    if pid == 0 || pid == 4 {
        return Err("Cannot suspend system processes".to_string());
    }

    // Check if already suspended
    if is_process_suspended(pid) {
        return Err("Process is already suspended".to_string());
    }

    let nt_suspend = get_nt_suspend_process()
        .ok_or_else(|| "Failed to load NtSuspendProcess".to_string())?;

    unsafe {
        let handle = OpenProcess(PROCESS_SUSPEND_RESUME, false, pid)
            .map_err(|e| format!("Failed to open process: {}", e))?;

        if handle.is_invalid() {
            return Err("Failed to open process: invalid handle".to_string());
        }

        let status = nt_suspend(handle);
        let _ = CloseHandle(handle);

        if status == STATUS_SUCCESS {
            mark_suspended(pid);
            Ok(())
        } else {
            Err(format!("NtSuspendProcess failed with status: 0x{:08X}", status.0))
        }
    }
}

/// Resume a suspended process by PID
/// 
/// Returns Ok(()) on success, Err with message on failure
pub fn resume_process(pid: u32) -> Result<(), String> {
    // Check if we think it's suspended
    if !is_process_suspended(pid) {
        return Err("Process is not suspended".to_string());
    }

    let nt_resume = get_nt_resume_process()
        .ok_or_else(|| "Failed to load NtResumeProcess".to_string())?;

    unsafe {
        let handle = OpenProcess(PROCESS_SUSPEND_RESUME, false, pid)
            .map_err(|e| format!("Failed to open process: {}", e))?;

        if handle.is_invalid() {
            return Err("Failed to open process: invalid handle".to_string());
        }

        let status = nt_resume(handle);
        let _ = CloseHandle(handle);

        if status == STATUS_SUCCESS {
            mark_resumed(pid);
            Ok(())
        } else {
            Err(format!("NtResumeProcess failed with status: 0x{:08X}", status.0))
        }
    }
}

/// Toggle suspend/resume state for a process
pub fn toggle_suspend(pid: u32) -> Result<bool, String> {
    if is_process_suspended(pid) {
        resume_process(pid)?;
        Ok(false) // Now running
    } else {
        suspend_process(pid)?;
        Ok(true) // Now suspended
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suspended_tracking() {
        init_suspended_pids();
        assert!(!is_process_suspended(99999));
        mark_suspended(99999);
        assert!(is_process_suspended(99999));
        mark_resumed(99999);
        assert!(!is_process_suspended(99999));
    }
}
