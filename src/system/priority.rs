//! Process priority management using Win32 APIs
//!
//! This module provides functions to get and set process priority
//! using GetPriorityClass and SetPriorityClass.

use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{
    GetPriorityClass, SetPriorityClass, OpenProcess,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_INFORMATION,
    ABOVE_NORMAL_PRIORITY_CLASS, BELOW_NORMAL_PRIORITY_CLASS,
    HIGH_PRIORITY_CLASS, IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS,
    REALTIME_PRIORITY_CLASS, PROCESS_CREATION_FLAGS,
};

use super::error::{PriorityError, PriorityResult};

/// Windows process priority levels.
///
/// Priority determines how the OS scheduler allocates CPU time to a process.
/// Higher priority processes receive more CPU time when competing for resources.
///
/// # Levels (lowest to highest)
/// * `Idle` - Runs only when system is idle
/// * `BelowNormal` - Lower than normal priority
/// * `Normal` - Default priority for most applications
/// * `AboveNormal` - Higher than normal priority
/// * `High` - Significantly more CPU time (use with caution)
/// * `Realtime` - Highest priority, can affect system stability
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Idle = 0,
    BelowNormal = 1,
    Normal = 2,
    AboveNormal = 3,
    High = 4,
    Realtime = 5,
    Unknown = 99,
}

impl Priority {
    /// Convert from Win32 priority class value
    pub fn from_win32(value: PROCESS_CREATION_FLAGS) -> Self {
        match value {
            IDLE_PRIORITY_CLASS => Priority::Idle,
            BELOW_NORMAL_PRIORITY_CLASS => Priority::BelowNormal,
            NORMAL_PRIORITY_CLASS => Priority::Normal,
            ABOVE_NORMAL_PRIORITY_CLASS => Priority::AboveNormal,
            HIGH_PRIORITY_CLASS => Priority::High,
            REALTIME_PRIORITY_CLASS => Priority::Realtime,
            _ => Priority::Unknown,
        }
    }

    /// Convert to Win32 priority class value
    pub fn to_win32(self) -> PROCESS_CREATION_FLAGS {
        match self {
            Priority::Idle => IDLE_PRIORITY_CLASS,
            Priority::BelowNormal => BELOW_NORMAL_PRIORITY_CLASS,
            Priority::Normal => NORMAL_PRIORITY_CLASS,
            Priority::AboveNormal => ABOVE_NORMAL_PRIORITY_CLASS,
            Priority::High => HIGH_PRIORITY_CLASS,
            Priority::Realtime => REALTIME_PRIORITY_CLASS,
            Priority::Unknown => NORMAL_PRIORITY_CLASS,
        }
    }

    /// Get the next higher priority level
    pub fn raise(self) -> Self {
        match self {
            Priority::Idle => Priority::BelowNormal,
            Priority::BelowNormal => Priority::Normal,
            Priority::Normal => Priority::AboveNormal,
            Priority::AboveNormal => Priority::High,
            Priority::High => Priority::Realtime,
            Priority::Realtime => Priority::Realtime, // Can't go higher
            Priority::Unknown => Priority::Normal,
        }
    }

    /// Get the next lower priority level
    pub fn lower(self) -> Self {
        match self {
            Priority::Idle => Priority::Idle, // Can't go lower
            Priority::BelowNormal => Priority::Idle,
            Priority::Normal => Priority::BelowNormal,
            Priority::AboveNormal => Priority::Normal,
            Priority::High => Priority::AboveNormal,
            Priority::Realtime => Priority::High,
            Priority::Unknown => Priority::Normal,
        }
    }

    /// Get a short display name
    pub fn short_name(&self) -> &'static str {
        match self {
            Priority::Idle => "Idle",
            Priority::BelowNormal => "BelowN",
            Priority::Normal => "Normal",
            Priority::AboveNormal => "AboveN",
            Priority::High => "High",
            Priority::Realtime => "RT",
            Priority::Unknown => "??",
        }
    }

    /// Get full display name
    pub fn name(&self) -> &'static str {
        match self {
            Priority::Idle => "Idle",
            Priority::BelowNormal => "Below Normal",
            Priority::Normal => "Normal",
            Priority::AboveNormal => "Above Normal",
            Priority::High => "High",
            Priority::Realtime => "Realtime",
            Priority::Unknown => "Unknown",
        }
    }
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Unknown
    }
}

/// Gets the priority class of a process.
///
/// Uses GetPriorityClass to query the process priority.
///
/// # Arguments
/// * `pid` - The process ID to query
///
/// # Returns
/// * `Priority` - The process priority (Unknown if access denied)
pub fn get_process_priority(pid: u32) -> Priority {
    // Try to open the process with query rights
    // SAFETY: OpenProcess is safe to call with valid parameters.
    let handle = unsafe {
        OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
    };

    let handle = match handle {
        Ok(h) => h,
        Err(_) => return Priority::Unknown,
    };

    // SAFETY: GetPriorityClass is safe with a valid handle.
    let priority_class = unsafe { GetPriorityClass(handle) };

    // Always close the handle
    // SAFETY: We own this handle.
    unsafe {
        let _ = CloseHandle(handle);
    }

    if priority_class == 0 {
        Priority::Unknown
    } else {
        Priority::from_win32(PROCESS_CREATION_FLAGS(priority_class))
    }
}

/// Sets the priority class of a process.
///
/// Uses SetPriorityClass to change the process priority.
/// Requires elevated privileges for some priority changes.
///
/// # Arguments
/// * `pid` - The process ID to modify
/// * `priority` - The new priority level
///
/// # Returns
/// * `Ok(())` - Priority was set successfully
/// * `Err(PriorityError)` - Error if failed
pub fn set_process_priority(pid: u32, priority: Priority) -> PriorityResult<()> {
    // Need PROCESS_SET_INFORMATION to change priority
    // SAFETY: OpenProcess is safe to call with valid parameters.
    let handle = unsafe {
        OpenProcess(PROCESS_SET_INFORMATION, false, pid)
    };

    let handle = match handle {
        Ok(h) => h,
        Err(e) => return Err(PriorityError::OpenFailed { message: e.to_string() }),
    };

    // SAFETY: SetPriorityClass is safe with a valid handle.
    let result = unsafe { SetPriorityClass(handle, priority.to_win32()) };

    // Always close the handle
    // SAFETY: We own this handle.
    unsafe {
        let _ = CloseHandle(handle);
    }

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(PriorityError::SetFailed { message: e.to_string() }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_own_priority() {
        let pid = std::process::id();
        let priority = get_process_priority(pid);
        println!("Our process priority: {:?}", priority);
        assert_ne!(priority, Priority::Unknown);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Idle < Priority::Normal);
        assert!(Priority::Normal < Priority::High);
        assert!(Priority::High < Priority::Realtime);
    }
}
