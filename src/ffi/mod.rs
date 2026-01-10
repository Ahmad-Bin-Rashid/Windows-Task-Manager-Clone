//! FFI module - Safe wrappers around Win32 handles and error handling
//!
//! This module provides RAII wrappers for Windows handles to ensure
//! proper cleanup via CloseHandle when handles go out of scope.

mod handles;

pub use handles::{ProcessHandle, SnapshotHandle};
