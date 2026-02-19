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

mod detail_view;
mod input;
mod navigation;
mod process_entry;
mod process_ops;
mod sort;
mod state;
mod tree_builder;

// Re-export public types
pub use input::KeyAction;
pub use process_entry::ProcessEntry;
pub use sort::SortColumn;
pub use state::App;

