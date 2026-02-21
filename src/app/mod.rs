//! Application module
//!
//! This module contains the application state and all related operations:
//! - `state` - Core App struct and refresh logic
//! - `process_entry` - Process data structure
//! - `sort` - Sorting configuration
//! - `navigation` - Cursor/selection movement
//! - `process_ops` - Kill, suspend, priority operations
//! - `detail_view` - Process details panel
//! - `tree_builder` - Process hierarchy tree
//! - `input` - Keyboard event handling
//! - `cli` - Command-line argument parsing
//! - `export` - CSV export functionality
//! - `affinity` - CPU affinity dialog

mod affinity;
mod cli;
mod detail_view;
mod export;
mod input;
mod navigation;
mod process_entry;
mod process_ops;
mod sort;
mod state;
mod tree_builder;
mod view_mode;

// ============================================================================
// Re-exports for clean imports
// ============================================================================

// CLI argument parsing
pub use cli::parse_args;

// CSV export
pub use export::export_to_csv;

// Input handling
pub use input::KeyAction;

// Core types
pub use process_entry::ProcessEntry;
pub use sort::SortColumn;
pub use state::App;
pub use view_mode::ViewMode;
