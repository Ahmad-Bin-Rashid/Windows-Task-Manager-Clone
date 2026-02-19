//! FFI module - Safe wrappers around Win32 handles and error handling
//!
//! This module provides RAII wrappers for Windows handles to ensure
//! proper cleanup via CloseHandle when handles go out of scope.

mod handles;

pub use handles::SnapshotHandle;

// ProcessHandle is available for future use
#[allow(unused_imports)]
pub use handles::ProcessHandle;
