//! User interface rendering
//!
//! This module provides all terminal UI rendering functionality:
//! - `render` - Main rendering entry point
//! - `components` - Header, stats bar, filter bar, column headers, footer
//! - `process_list` - Process list rendering
//! - `detail_view` - Detailed process information view
//! - `help` - Help overlay
//! - `affinity` - CPU affinity dialog
//! - `utils` - Shared utilities

mod affinity;
mod components;
mod detail_view;
mod help;
mod process_list;
mod render;
mod utils;

pub use render::render;
