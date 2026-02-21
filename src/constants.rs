//! Centralized constants for the application
//!
//! This module contains all magic numbers and configuration constants
//! used throughout the application, making them easy to find and modify.

// Allow unused constants - these are reserved for future use
// #![allow(dead_code)]

// ============================================================================
// Application Info
// ============================================================================

/// Application name displayed in header
pub const DISPLAY_NAME: &str = "Windows Task Manager CLI";

/// Application name from Cargo.toml
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");

/// Application version from Cargo.toml
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

// ============================================================================
// Refresh Rate (milliseconds)
// ============================================================================

/// Default refresh interval in milliseconds
pub const DEFAULT_REFRESH_MS: u64 = 2000;

/// Minimum allowed refresh interval
pub const MIN_REFRESH_MS: u64 = 250;

/// Maximum allowed refresh interval
pub const MAX_REFRESH_MS: u64 = 10000;


// ============================================================================
// Navigation
// ============================================================================

/// Number of lines to scroll on PageUp/PageDown in help overlay
pub const HELP_PAGE_SCROLL_LINES: usize = 5;

/// Lines subtracted from terminal height to calculate visible rows
/// (accounts for header, footer, etc.)
pub const VISIBLE_ROWS_OVERHEAD: usize = 6;

// ============================================================================
// Process Tree
// ============================================================================

/// Maximum indentation depth for tree view display
pub const MAX_TREE_INDENT_DEPTH: usize = 5;

// ============================================================================
// Byte Size Conversions
// ============================================================================

/// Bytes in a kilobyte
pub const BYTES_PER_KB: f64 = 1024.0;

/// Bytes in a megabyte
pub const BYTES_PER_MB: f64 = 1_048_576.0;

/// Bytes in a gigabyte
pub const BYTES_PER_GB: f64 = 1_073_741_824.0;

// ============================================================================
// UI Dialog Dimensions
// ============================================================================

/// Width of the help dialog box
pub const HELP_DIALOG_WIDTH: usize = 52;

/// Width of the CPU affinity dialog box
pub const AFFINITY_DIALOG_WIDTH: usize = 60;

/// Minimum margin from screen edge for dialogs
pub const DIALOG_MARGIN: usize = 4;

// ============================================================================
// Help Dialog Formatting
// ============================================================================

/// Width of the key column in help dialog
pub const HELP_KEY_COL_WIDTH: usize = 14;

// ============================================================================
// CPU Usage Thresholds (for coloring)
// ============================================================================

/// CPU usage threshold for red color (critical)
pub const CPU_THRESHOLD_CRITICAL: f64 = 80.0;

/// CPU usage threshold for yellow color (warning)
pub const CPU_THRESHOLD_WARNING: f64 = 50.0;

/// CPU usage threshold for cyan color (moderate)
pub const CPU_THRESHOLD_MODERATE: f64 = 20.0;
