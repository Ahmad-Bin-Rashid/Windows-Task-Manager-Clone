//! Header, stats bar, filter bar, column headers, and footer components

use std::io;

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};

use crate::app::App;
use crate::constants::DISPLAY_NAME;
use crate::system::{format_bytes, is_elevated, SystemMemoryInfo};

use super::utils::truncate_string;

/// Renders the application header with admin status indicator.
///
/// Displays the application title and whether it's running with
/// elevated (Administrator) privileges.
pub fn render_header(stdout: &mut io::Stdout, width: usize) -> io::Result<()> {
    let admin_indicator = if is_elevated() {
        ("[Administrator]", Color::Green)
    } else {
        ("[User]", Color::Yellow)
    };

    let title = format!(" {}", DISPLAY_NAME);
    let spacing = width.saturating_sub(title.len() + admin_indicator.0.len() + 2);

    execute!(
        stdout,
        SetBackgroundColor(Color::DarkBlue),
        SetForegroundColor(Color::White),
        Print(title),
        Print(format!("{:spacing$}", "", spacing = spacing)),
        SetForegroundColor(admin_indicator.1),
        Print(admin_indicator.0),
        Print(" "),
        ResetColor,
        Print("\r\n")
    )
}

/// Renders the system statistics line.
///
/// Shows CPU usage, memory usage, process count, current sort column,
/// and refresh interval.
pub fn render_system_stats(
    stdout: &mut io::Stdout,
    app: &App,
    mem_info: &Option<SystemMemoryInfo>,
    width: usize,
) -> io::Result<()> {
    let cpu_str = format!("CPU: {:5.1}%", app.system_cpu);
    let mem_str = if let Some(ref info) = mem_info {
        format!(
            "Memory: {} / {} ({:.0}%)",
            format_bytes(info.used_physical()),
            format_bytes(info.total_physical),
            info.memory_load_percent
        )
    } else {
        "Memory: N/A".to_string()
    };
    let proc_count = if app.filter.is_empty() {
        format!("Processes: {}", app.processes.len())
    } else {
        format!("Showing: {}/{}", app.filtered_processes.len(), app.processes.len())
    };
    let sort_arrow = if app.sort_ascending { "↑" } else { "↓" };
    let sort_str = if app.tree_view_mode {
        "View: Tree".to_string()
    } else {
        format!("Sort: {} {}", app.sort_column.name(), sort_arrow)
    };
    let refresh_str = format!("Refresh: {}", app.format_refresh_interval());

    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print(format!(
            " {}  |  {}  |  {}  |  {}  |  {}",
            cpu_str, mem_str, proc_count, sort_str, refresh_str
        )),
        ResetColor,
        Print(format!("{:width$}\r\n", "", width = width.saturating_sub(100)))
    )
}

/// Renders the filter bar when active or showing current filter.
///
/// In filter mode, displays an input field with cursor.
/// Otherwise, shows the current filter value if set.
pub fn render_filter_bar(stdout: &mut io::Stdout, app: &App, width: usize) -> io::Result<()> {
    if app.view_mode.is_filter_input() {
        execute!(
            stdout,
            SetBackgroundColor(Color::DarkYellow),
            SetForegroundColor(Color::Black),
            Print(format!(
                " Filter: {}█{:width$}",
                app.filter,
                "",
                width = width.saturating_sub(app.filter.len() + 10)
            )),
            ResetColor,
            Print("\r\n")
        )
    } else if !app.filter.is_empty() {
        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print(format!(
                " Filter: \"{}\" (press / to edit, Esc to clear)",
                app.filter
            )),
            ResetColor,
            Print("\r\n")
        )
    } else {
        execute!(stdout, Print("\r\n"))
    }
}

/// Renders the column headers for the process list.
///
/// Displays headers for: PID, Priority, Threads, Handles, Uptime,
/// Memory, CPU%, Read/s, Write/s, and Name.
pub fn render_column_headers(stdout: &mut io::Stdout, width: usize) -> io::Result<()> {
    let header = format!(
        " {:>7}  {:>8}  {:>5}  {:>6}  {:>9}  {:>10}  {:>6}  {:>9}  {:>9}  {}",
        "PID", "Priority", "Thrd", "Hndls", "Uptime", "Memory", "CPU%", "Read/s", "Write/s", "Name"
    );
    execute!(
        stdout,
        SetBackgroundColor(Color::DarkGrey),
        SetForegroundColor(Color::White),
        Print(format!("{:width$}", header, width = width)),
        ResetColor,
        Print("\r\n")
    )
}

/// Renders the footer with status/error messages and help hints.
///
/// Shows kill confirmation dialog when in confirm mode,
/// error messages when present, or keyboard shortcuts otherwise.
pub fn render_footer(stdout: &mut io::Stdout, app: &App, width: usize) -> io::Result<()> {
    // Error/status message or confirmation dialog
    if app.view_mode.is_confirm_kill() {
        if let (Some(pid), Some(ref name)) = (app.pending_kill_pid, &app.pending_kill_name) {
            execute!(
                stdout,
                SetBackgroundColor(Color::DarkRed),
                SetForegroundColor(Color::White),
                Print(format!(
                    " Kill process '{}' (PID {})? [Y/N] {:width$}",
                    truncate_string(name, 30),
                    pid,
                    "",
                    width = width.saturating_sub(60)
                )),
                ResetColor,
                Print("\r\n")
            )?;
        } else {
            execute!(stdout, Print("\r\n"))?;
        }
    } else if let Some(ref msg) = app.error_message {
        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print(format!(" {}", truncate_string(msg, width - 2))),
            ResetColor,
            Print("\r\n")
        )?;
    } else {
        // Show selected process path
        let path_display = app
            .filtered_processes
            .get(app.selected_index)
            .and_then(|p| p.path.as_ref())
            .map(|p| format!(" Path: {}", truncate_string(p, width.saturating_sub(10))))
            .unwrap_or_else(|| " Path: <access denied>".to_string());
        execute!(
            stdout,
            SetForegroundColor(Color::DarkGrey),
            Print(format!("{:width$}", path_display, width = width)),
            ResetColor,
            Print("\r\n")
        )?;
    }

    // Help line
    if app.view_mode.is_confirm_kill() {
        execute!(
            stdout,
            SetBackgroundColor(Color::DarkRed),
            SetForegroundColor(Color::White),
            Print(format!("{:width$}", " Kill process? Y:Confirm | N/Esc:Cancel", width = width)),
            ResetColor,
        )?;
    } else if app.view_mode.is_filter_input() {
        execute!(
            stdout,
            SetBackgroundColor(Color::DarkYellow),
            SetForegroundColor(Color::Black),
            Print(format!("{:width$}", " Type to filter | Enter:Apply | Esc:Cancel", width = width)),
            ResetColor,
        )?;
    } else {
        let help_line = " ?:Help | q:Quit | Enter:Details | k:Kill | p:Suspend | t:Tree | s:Sort | /:Filter | +/-:Priority";
        execute!(
            stdout,
            SetBackgroundColor(Color::DarkBlue),
            SetForegroundColor(Color::White),
            Print(format!("{:width$}", truncate_string(help_line, width), width = width)),
            ResetColor,
        )?;
    }
    
    Ok(())
}
