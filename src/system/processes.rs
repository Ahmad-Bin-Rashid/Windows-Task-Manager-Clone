//! Process enumeration using the ToolHelp32 API
//!
//! This module provides functions to enumerate all running processes
//! on the system using CreateToolhelp32Snapshot and Process32First/Next.

use std::mem;
use windows::Win32::System::Diagnostics::ToolHelp::{
    Process32FirstW, Process32NextW, PROCESSENTRY32W,
};

use crate::ffi::SnapshotHandle;

/// Information about a single process
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Parent process ID
    pub parent_pid: u32,
    /// Number of threads
    pub thread_count: u32,
    /// Base priority of the process
    pub base_priority: i32,
    /// Executable name (e.g., "notepad.exe")
    pub name: String,
}

impl ProcessInfo {
    /// Creates a ProcessInfo from a PROCESSENTRY32W struct
    fn from_entry(entry: &PROCESSENTRY32W) -> Self {
        // Convert the wide string (null-terminated u16 array) to a Rust String
        let name = wide_to_string(&entry.szExeFile);
        
        Self {
            pid: entry.th32ProcessID,
            parent_pid: entry.th32ParentProcessID,
            thread_count: entry.cntThreads,
            base_priority: entry.pcPriClassBase,
            name,
        }
    }
}

/// Converts a null-terminated wide string (u16 slice) to a Rust String
fn wide_to_string(wide: &[u16]) -> String {
    // Find the null terminator
    let len = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
    String::from_utf16_lossy(&wide[..len])
}

/// Enumerates all running processes on the system.
///
/// Uses the ToolHelp32 API to create a snapshot of all processes
/// and iterates through them.
///
/// # Returns
/// * `Ok(Vec<ProcessInfo>)` - A vector of all running processes
/// * `Err` - If the snapshot cannot be created
///
/// # Example
/// ```no_run
/// let processes = enumerate_processes().expect("Failed to enumerate processes");
/// for proc in processes {
///     println!("{}: {}", proc.pid, proc.name);
/// }
/// ```
#[must_use]
pub fn enumerate_processes() -> windows::core::Result<Vec<ProcessInfo>> {
    let snapshot = SnapshotHandle::create_process_snapshot()?;
    let mut processes = Vec::new();
    
    // Initialize the entry structure - CRITICAL: dwSize must be set!
    let mut entry = PROCESSENTRY32W {
        dwSize: mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    
    // Get the first process
    // SAFETY: We have a valid snapshot handle and properly initialized entry.
    let mut success = unsafe { Process32FirstW(snapshot.as_raw(), &mut entry) };
    
    while success.is_ok() {
        processes.push(ProcessInfo::from_entry(&entry));
        
        // Get the next process
        // SAFETY: Same as above - valid handles and initialized struct.
        success = unsafe { Process32NextW(snapshot.as_raw(), &mut entry) };
    }
    
    Ok(processes)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_enumerate_processes() {
        let processes = enumerate_processes().expect("Should enumerate processes");
        assert!(!processes.is_empty(), "Should find at least one process");
        
        // We should find our own process
        let current_pid = std::process::id();
        let found = processes.iter().any(|p| p.pid == current_pid);
        assert!(found, "Should find our own process");
    }
}
