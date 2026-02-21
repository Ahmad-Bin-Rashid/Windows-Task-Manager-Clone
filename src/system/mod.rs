//! System information module - Process, memory, and CPU metrics
//!
//! This module provides safe abstractions over Win32 system APIs
//! for gathering task manager-style information.

// Allow unused re-exports - these are public API for potential use
#![allow(unused_imports)]

mod admin;
mod affinity;
pub mod cpu;
mod details;
mod disk;
mod error;
mod memory;
mod path;
mod priority;
mod processes;
mod suspend;
mod uptime;

// ============================================================================
// Re-exports for clean imports
// ============================================================================

// Admin/elevation
pub use admin::{elevation_indicator, elevation_status_string, is_elevated};

// CPU affinity
pub use affinity::{get_process_affinity, get_system_core_count, set_process_affinity, CpuAffinity};

// Process details
pub use details::{
    get_process_command_line, get_process_modules, get_process_tcp_connections,
    get_process_udp_endpoints, ModuleInfo, ProcessDetails, TcpConnectionInfo, UdpEndpointInfo,
};

// Disk I/O
pub use disk::{get_process_disk_info, ProcessDiskInfo};

// Memory
pub use memory::{
    format_bytes, get_process_memory_info, get_system_memory_info, ProcessMemoryInfo,
    SystemMemoryInfo,
};

// Path and handles
pub use path::{get_process_handle_count, get_process_path, path_to_filename};

// Priority
pub use priority::{get_process_priority, set_process_priority, Priority};

// Process enumeration
pub use processes::{enumerate_processes, ProcessInfo};

// Suspend/resume
pub use suspend::{
    is_process_suspended, resume_process, suspend_process, toggle_suspend, untrack_process,
};

// Uptime
pub use uptime::{
    calculate_uptime_seconds, format_uptime, get_current_filetime, get_process_start_time,
};

// Error types
pub use error::{
    AffinityError, AffinityResult, PriorityError, PriorityResult, ProcessError, ProcessResult,
};
