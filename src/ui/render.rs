//! Main rendering entry point
//!
//! This module coordinates rendering by dispatching to specialized sub-modules:
//! - `components` - Header, stats, filter bar, column headers, footer
//! - `process_list` - Process list rendering
//! - `detail_view` - Detailed process information view
//! - `help` - Help overlay
//! - `utils` - Shared utilities (truncate, format_rate, etc.)

use std::io::{self, Write};

use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{self, Clear, ClearType},
};

use crate::app::App;
use crate::system::memory::get_system_memory_info;

use super::components::{
    render_column_headers, render_filter_bar, render_footer, render_header, render_system_stats,
};
use super::affinity::render_affinity_dialog;
use super::detail_view::render_detail_view;
use super::help::render_help_overlay;
use super::process_list::render_process_list;

/// Renders the UI to the terminal
///
/// This is the main entry point for all rendering. It determines the current
/// view mode and dispatches to the appropriate rendering function.
pub fn render(stdout: &mut io::Stdout, app: &mut App) -> io::Result<()> {
    let (width, height) = terminal::size()?;
    let width = width as usize;
    let height = height as usize;

    // Clear and move to top
    execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;

    // Dispatch based on current view mode
    if app.show_help {
        return render_help_overlay(stdout, width, height);
    }

    if app.affinity_mode {
        return render_affinity_dialog(stdout, app, width, height);
    }

    if app.detail_view_mode {
        return render_detail_view(stdout, app, width, height);
    }

    // Main process list view
    render_main_view(stdout, app, width, height)
}

/// Renders the main process list view
fn render_main_view(
    stdout: &mut io::Stdout,
    app: &mut App,
    width: usize,
    height: usize,
) -> io::Result<()> {
    // Get system memory info
    let mem_info = get_system_memory_info().ok();

    // === HEADER ===
    render_header(stdout, width)?;

    // === SYSTEM STATS ===
    render_system_stats(stdout, app, &mem_info, width)?;

    // === FILTER BAR ===
    render_filter_bar(stdout, app, width)?;

    // === COLUMN HEADERS ===
    render_column_headers(stdout, width)?;

    // === PROCESS LIST ===
    let header_lines = 5;
    let footer_lines = 2;
    let visible_rows = height.saturating_sub(header_lines + footer_lines);

    render_process_list(stdout, app, visible_rows, width)?;

    // === FOOTER ===
    render_footer(stdout, app, width)?;

    stdout.flush()
}
