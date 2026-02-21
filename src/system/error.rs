//! Custom error types for type-safe error handling
//!
//! This module provides structured error types instead of raw strings,
//! enabling better error handling, matching, and user messages.

use std::fmt;

// ============================================================================
// Process Error
// ============================================================================

/// Errors that can occur during process operations.
///
/// This enum provides structured error information for operations like
/// kill, suspend, resume, priority changes, and affinity modifications.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessError {
    /// Cannot perform operation on system processes (PID 0 or 4)
    SystemProcess,
    
    /// Process does not exist or has already terminated
    NotFound,
    
    /// Access denied - typically requires elevation
    AccessDenied,
    
    /// Process is already in the requested state
    AlreadyInState {
        /// Description of current state
        state: &'static str,
    },
    
    /// Invalid handle returned from Windows API
    InvalidHandle,
    
    /// Windows API call failed with an error code
    WinApiError {
        /// Name of the failing API function
        api: &'static str,
        /// Windows error code (HRESULT or NTSTATUS)
        code: i32,
    },
    
    /// Failed to load a required function from ntdll.dll
    NtdllLoadFailed {
        /// Name of the function that failed to load
        function: &'static str,
    },
    
    /// Invalid parameter provided to a function
    InvalidParameter {
        /// Description of what was invalid
        reason: &'static str,
    },
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessError::SystemProcess => {
                write!(f, "Cannot perform operation on system processes")
            }
            ProcessError::NotFound => {
                write!(f, "Process not found or has terminated")
            }
            ProcessError::AccessDenied => {
                write!(f, "Access denied - try running as Administrator")
            }
            ProcessError::AlreadyInState { state } => {
                write!(f, "Process is already {}", state)
            }
            ProcessError::InvalidHandle => {
                write!(f, "Invalid process handle")
            }
            ProcessError::WinApiError { api, code } => {
                write!(f, "{} failed (error code: 0x{:08X})", api, code)
            }
            ProcessError::NtdllLoadFailed { function } => {
                write!(f, "Failed to load {} from ntdll.dll", function)
            }
            ProcessError::InvalidParameter { reason } => {
                write!(f, "Invalid parameter: {}", reason)
            }
        }
    }
}

impl std::error::Error for ProcessError {}

// ============================================================================
// Affinity Error
// ============================================================================

/// Errors specific to CPU affinity operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AffinityError {
    /// No cores selected in the affinity mask
    NoCoresSelected,
    
    /// Cannot modify system process affinity
    SystemProcess,
    
    /// Access denied - needs elevation
    AccessDenied,
    
    /// Failed to read current affinity
    ReadFailed,
    
    /// Failed to set new affinity
    SetFailed,
    
    /// Selected cores are not available on this system
    InvalidCoreSelection,
}

impl fmt::Display for AffinityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AffinityError::NoCoresSelected => {
                write!(f, "At least one core must be selected")
            }
            AffinityError::SystemProcess => {
                write!(f, "Cannot modify system process affinity")
            }
            AffinityError::AccessDenied => {
                write!(f, "Access denied - try running as Administrator")
            }
            AffinityError::ReadFailed => {
                write!(f, "Cannot read process affinity")
            }
            AffinityError::SetFailed => {
                write!(f, "Failed to set affinity - access denied")
            }
            AffinityError::InvalidCoreSelection => {
                write!(f, "Invalid core selection")
            }
        }
    }
}

impl std::error::Error for AffinityError {}

// ============================================================================
// Priority Error
// ============================================================================

/// Errors specific to priority operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PriorityError {
    /// Cannot open the process for modification
    OpenFailed {
        /// Windows error message
        message: String,
    },
    
    /// Failed to set the priority
    SetFailed {
        /// Windows error message
        message: String,
    },
}

impl fmt::Display for PriorityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PriorityError::OpenFailed { message } => {
                write!(f, "Cannot open process: {}", message)
            }
            PriorityError::SetFailed { message } => {
                write!(f, "Failed to set priority: {}", message)
            }
        }
    }
}

impl std::error::Error for PriorityError {}

// ============================================================================
// Conversion helpers
// ============================================================================

impl From<ProcessError> for String {
    fn from(err: ProcessError) -> String {
        err.to_string()
    }
}

impl From<AffinityError> for String {
    fn from(err: AffinityError) -> String {
        err.to_string()
    }
}

impl From<PriorityError> for String {
    fn from(err: PriorityError) -> String {
        err.to_string()
    }
}

// ============================================================================
// Result type aliases
// ============================================================================

/// Result type for process operations
pub type ProcessResult<T> = Result<T, ProcessError>;

/// Result type for affinity operations
pub type AffinityResult<T> = Result<T, AffinityError>;

/// Result type for priority operations
pub type PriorityResult<T> = Result<T, PriorityError>;
