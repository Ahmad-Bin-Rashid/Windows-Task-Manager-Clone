//! Process entry data structure

use crate::system::priority::Priority;
use crate::system::processes::ProcessInfo;

/// Process entry with calculated metrics
#[derive(Debug, Clone)]
pub struct ProcessEntry {
    /// Basic process information
    pub info: ProcessInfo,
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Memory usage in bytes (working set)
    pub memory_bytes: u64,
    /// Disk read bytes (total since process start)
    pub disk_read: u64,
    /// Disk write bytes (total since process start)
    pub disk_write: u64,
    /// Process priority class
    pub priority: Priority,
    /// Number of threads in the process
    pub thread_count: u32,
}
