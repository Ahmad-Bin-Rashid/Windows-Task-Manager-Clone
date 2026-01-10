//! Terminal rendering logic

use std::io::{self, Write};

use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};

use crate::app::App;
use crate::system::memory::{format_bytes, get_system_memory_info};
use crate::system::uptime::format_uptime;

/// Renders the UI to the terminal
pub fn render(stdout: &mut io::Stdout, app: &mut App) -> io::Result<()> {
    let (width, height) = terminal::size()?;
    let width = width as usize;
    let height = height as usize;

    // Get system memory info
    let mem_info = get_system_memory_info().ok();

    // Clear and move to top
    execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;

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

/// Renders the application header
fn render_header(stdout: &mut io::Stdout, width: usize) -> io::Result<()> {
    execute!(
        stdout,
        SetBackgroundColor(Color::DarkBlue),
        SetForegroundColor(Color::White),
        Print(format!("{:width$}", " Windows Task Manager Clone", width = width)),
        ResetColor,
        Print("\r\n")
    )
}

/// Renders system statistics line
fn render_system_stats(
    stdout: &mut io::Stdout,
    app: &App,
    mem_info: &Option<crate::system::memory::SystemMemoryInfo>,
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
    let sort_str = format!("Sort: {} {}", app.sort_column.name(), sort_arrow);

    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print(format!(
            " {}  |  {}  |  {}  |  {}",
            cpu_str, mem_str, proc_count, sort_str
        )),
        ResetColor,
        Print(format!("{:width$}\r\n", "", width = width.saturating_sub(80)))
    )
}

/// Renders the filter bar
fn render_filter_bar(stdout: &mut io::Stdout, app: &App, width: usize) -> io::Result<()> {
    if app.filter_mode {
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

/// Renders column headers
fn render_column_headers(stdout: &mut io::Stdout, width: usize) -> io::Result<()> {
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

/// Renders the process list
fn render_process_list(
    stdout: &mut io::Stdout,
    app: &mut App,
    visible_rows: usize,
    width: usize,
) -> io::Result<()> {
    // Adjust scroll offset to keep selection visible
    if app.selected_index < app.scroll_offset {
        app.scroll_offset = app.selected_index;
    } else if app.selected_index >= app.scroll_offset + visible_rows {
        app.scroll_offset = app.selected_index - visible_rows + 1;
    }

    // Display processes
    for (i, entry) in app
        .filtered_processes
        .iter()
        .skip(app.scroll_offset)
        .take(visible_rows)
        .enumerate()
    {
        let actual_index = app.scroll_offset + i;
        let is_selected = actual_index == app.selected_index;

        // Color-code CPU usage
        let cpu_color = if entry.cpu_percent >= 80.0 {
            Color::Red
        } else if entry.cpu_percent >= 50.0 {
            Color::Yellow
        } else if entry.cpu_percent >= 20.0 {
            Color::Cyan
        } else {
            Color::Green
        };

        // Build the line parts
        let prefix = format!(
            " {:>7}  {:>8}  {:>5}  {:>6}  {:>9}  {:>10}  ",
            entry.info.pid,
            entry.priority.short_name(),
            entry.thread_count,
            entry.handle_count,
            format_uptime(entry.uptime_seconds),
            format_bytes(entry.memory_bytes),
        );
        let cpu_str = format!("{:>5.1}%", entry.cpu_percent);
        let suffix = format!(
            "  {:>9}  {:>9}  {}",
            format_rate(entry.disk_read_rate),
            format_rate(entry.disk_write_rate),
            truncate_string(&entry.info.name, width.saturating_sub(90))
        );

        if is_selected {
            // Selected row - use background color, CPU still colored
            execute!(
                stdout,
                SetBackgroundColor(Color::DarkCyan),
                SetForegroundColor(Color::White),
                Print(&prefix),
                SetForegroundColor(cpu_color),
                Print(&cpu_str),
                SetForegroundColor(Color::White),
                Print(format!(
                    "{:width$}",
                    suffix,
                    width = width.saturating_sub(prefix.len() + cpu_str.len())
                )),
                ResetColor,
            )?;
        } else {
            // Normal row - color only CPU
            execute!(
                stdout,
                Print(&prefix),
                SetForegroundColor(cpu_color),
                Print(&cpu_str),
                ResetColor,
                Print(format!(
                    "{:width$}",
                    suffix,
                    width = width.saturating_sub(prefix.len() + cpu_str.len())
                )),
            )?;
        }
        execute!(stdout, Print("\r\n"))?;
    }

    // Fill remaining space
    for _ in app.filtered_processes.len().min(visible_rows)..visible_rows {
        execute!(stdout, Print(format!("{:width$}\r\n", "", width = width)))?;
    }

    Ok(())
}

/// Renders the footer (status/error message and help line)
fn render_footer(stdout: &mut io::Stdout, app: &App, width: usize) -> io::Result<()> {
    // Error/status message or confirmation dialog
    if app.confirm_kill_mode {
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
    let help_text = if app.confirm_kill_mode {
        " Y:Confirm Kill | N/Esc:Cancel"
    } else if app.filter_mode {
        " Type to filter | Enter:Apply | Esc:Cancel"
    } else {
        " q:Quit | k:Kill | +/-:Priority | s:Sort | r:Reverse | /:Filter"
    };
    execute!(
        stdout,
        SetBackgroundColor(Color::DarkBlue),
        SetForegroundColor(Color::White),
        Print(format!("{:width$}", help_text, width = width)),
        ResetColor,
    )
}

/// Truncates a string to fit within a given width
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len > 3 {
        format!("{}...", &s[..max_len - 3])
    } else {
        s[..max_len].to_string()
    }
}

/// Formats a byte rate (bytes/sec) as a human-readable string
fn format_rate(bytes_per_sec: f64) -> String {
    if bytes_per_sec < 1.0 {
        "0 B/s".to_string()
    } else if bytes_per_sec < 1024.0 {
        format!("{:.0} B/s", bytes_per_sec)
    } else if bytes_per_sec < 1024.0 * 1024.0 {
        format!("{:.1} KB/s", bytes_per_sec / 1024.0)
    } else if bytes_per_sec < 1024.0 * 1024.0 * 1024.0 {
        format!("{:.1} MB/s", bytes_per_sec / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB/s", bytes_per_sec / (1024.0 * 1024.0 * 1024.0))
    }
}
