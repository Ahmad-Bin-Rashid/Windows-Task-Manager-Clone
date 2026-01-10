//! CPU usage tracking using Win32 APIs
//!
//! This module provides functions to calculate CPU usage for the system
//! and individual processes using GetSystemTimes and GetProcessTimes.
//! 
//! CPU usage requires delta measurements between two time points.

use std::collections::HashMap;
use windows::Win32::Foundation::{CloseHandle, FILETIME};
use windows::Win32::System::Threading::{
    GetProcessTimes, GetSystemTimes, OpenProcess,
    PROCESS_QUERY_LIMITED_INFORMATION,
};

/// Snapshot of system-wide CPU times
#[derive(Debug, Clone, Default)]
pub struct SystemCpuSnapshot {
    /// Time spent in idle
    pub idle_time: u64,
    /// Time spent in kernel mode (includes idle)
    pub kernel_time: u64,
    /// Time spent in user mode
    pub user_time: u64,
}

/// Snapshot of per-process CPU times
#[derive(Debug, Clone, Default)]
pub struct ProcessCpuSnapshot {
    /// Time spent in kernel mode
    pub kernel_time: u64,
    /// Time spent in user mode
    pub user_time: u64,
}

/// Holds CPU snapshots for calculating deltas
#[derive(Debug, Clone, Default)]
pub struct CpuTracker {
    /// Previous system CPU snapshot
    prev_system: SystemCpuSnapshot,
    /// Previous per-process CPU snapshots (keyed by PID)
    prev_processes: HashMap<u32, ProcessCpuSnapshot>,
    /// Number of logical processors
    num_cpus: u32,
}

impl CpuTracker {
    /// Creates a new CPU tracker and takes initial measurements.
    pub fn new() -> Self {
        let num_cpus = get_num_cpus();
        let mut tracker = Self {
            prev_system: SystemCpuSnapshot::default(),
            prev_processes: HashMap::new(),
            num_cpus,
        };
        
        // Take initial snapshot
        if let Ok(snapshot) = get_system_cpu_snapshot() {
            tracker.prev_system = snapshot;
        }
        
        tracker
    }
    
    /// Updates the system snapshot and returns the CPU usage percentage.
    pub fn get_system_cpu_usage(&mut self) -> f64 {
        let current = match get_system_cpu_snapshot() {
            Ok(s) => s,
            Err(_) => return 0.0,
        };
        
        let idle_delta = current.idle_time.saturating_sub(self.prev_system.idle_time);
        let kernel_delta = current.kernel_time.saturating_sub(self.prev_system.kernel_time);
        let user_delta = current.user_time.saturating_sub(self.prev_system.user_time);
        
        // Total time = kernel + user (kernel includes idle)
        let total_time = kernel_delta + user_delta;
        let busy_time = total_time.saturating_sub(idle_delta);
        
        self.prev_system = current;
        
        if total_time == 0 {
            return 0.0;
        }
        
        (busy_time as f64 / total_time as f64) * 100.0
    }
    
    /// Gets CPU usage for a specific process as a percentage.
    /// Returns 0.0 if the process cannot be accessed or on first call.
    pub fn get_process_cpu_usage(&mut self, pid: u32) -> f64 {
        let current_system = match get_system_cpu_snapshot() {
            Ok(s) => s,
            Err(_) => return 0.0,
        };
        
        let current_process = get_process_cpu_snapshot(pid);
        
        // Calculate deltas
        let prev_process = self.prev_processes.get(&pid).cloned()
            .unwrap_or_default();
        
        let system_delta = (current_system.kernel_time + current_system.user_time)
            .saturating_sub(self.prev_system.kernel_time + self.prev_system.user_time);
        
        let process_delta = (current_process.kernel_time + current_process.user_time)
            .saturating_sub(prev_process.kernel_time + prev_process.user_time);
        
        // Store current snapshot for next calculation
        self.prev_processes.insert(pid, current_process);
        
        if system_delta == 0 {
            return 0.0;
        }
        
        // Calculate CPU usage as percentage of total system CPU (0-100%)
        // This matches Windows Task Manager behavior
        let usage = (process_delta as f64 / system_delta as f64) * 100.0;
        
        // Clamp to reasonable range
        usage.min(100.0).max(0.0)
    }
    
    /// Clears tracked processes that no longer exist.
    pub fn cleanup_stale_processes(&mut self, active_pids: &[u32]) {
        self.prev_processes.retain(|pid, _| active_pids.contains(pid));
    }
}

/// Converts a FILETIME to a u64 (100-nanosecond intervals since 1601)
fn filetime_to_u64(ft: &FILETIME) -> u64 {
    ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64)
}

/// Gets the number of logical processors
fn get_num_cpus() -> u32 {
    // Use environment variable or default to 1
    std::env::var("NUMBER_OF_PROCESSORS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
}

/// Gets a snapshot of system-wide CPU times.
fn get_system_cpu_snapshot() -> windows::core::Result<SystemCpuSnapshot> {
    let mut idle_time = FILETIME::default();
    let mut kernel_time = FILETIME::default();
    let mut user_time = FILETIME::default();
    
    // SAFETY: GetSystemTimes is safe to call with valid pointers.
    unsafe {
        GetSystemTimes(
            Some(&mut idle_time),
            Some(&mut kernel_time),
            Some(&mut user_time),
        )?;
    }
    
    Ok(SystemCpuSnapshot {
        idle_time: filetime_to_u64(&idle_time),
        kernel_time: filetime_to_u64(&kernel_time),
        user_time: filetime_to_u64(&user_time),
    })
}

/// Gets CPU times for a specific process.
fn get_process_cpu_snapshot(pid: u32) -> ProcessCpuSnapshot {
    // Try to open the process
    // SAFETY: OpenProcess is safe to call with valid parameters.
    let handle = unsafe {
        OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
    };
    
    let handle = match handle {
        Ok(h) => h,
        Err(_) => return ProcessCpuSnapshot::default(),
    };
    
    let mut creation_time = FILETIME::default();
    let mut exit_time = FILETIME::default();
    let mut kernel_time = FILETIME::default();
    let mut user_time = FILETIME::default();
    
    // SAFETY: GetProcessTimes is safe with a valid handle and pointers.
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
    // SAFETY: We own this handle.
    unsafe {
        let _ = CloseHandle(handle);
    }
    
    if result.is_ok() {
        ProcessCpuSnapshot {
            kernel_time: filetime_to_u64(&kernel_time),
            user_time: filetime_to_u64(&user_time),
        }
    } else {
        ProcessCpuSnapshot::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_system_cpu_snapshot() {
        let snapshot = get_system_cpu_snapshot().expect("Should get CPU times");
        assert!(snapshot.kernel_time > 0, "Should have some kernel time");
    }
    
    #[test]
    fn test_cpu_tracker() {
        let mut tracker = CpuTracker::new();
        
        // Wait a bit to accumulate CPU time
        thread::sleep(Duration::from_millis(100));
        
        let usage = tracker.get_system_cpu_usage();
        assert!(usage >= 0.0, "CPU usage should be non-negative");
        assert!(usage <= 100.0, "CPU usage should be at most 100%");
    }
}
