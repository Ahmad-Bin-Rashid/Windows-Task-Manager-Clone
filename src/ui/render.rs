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

    // Clear and move to top
    execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;

    // Check if we're in detail view mode
    if app.detail_view_mode {
        return render_detail_view(stdout, app, width, height);
    }

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
        // Add tree indentation if in tree view mode
        let tree_prefix = if app.tree_view_mode && entry.tree_depth > 0 {
            let indent = "  ".repeat(entry.tree_depth.min(5));
            format!("{}└─", indent)
        } else {
            String::new()
        };
        
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
        
        // Calculate available space for name with tree prefix
        let name_space = width.saturating_sub(90 + tree_prefix.len());
        let suffix = format!(
            "  {:>9}  {:>9}  {}{}",
            format_rate(entry.disk_read_rate),
            format_rate(entry.disk_write_rate),
            tree_prefix,
            truncate_string(&entry.info.name, name_space)
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
        " q:Quit | Enter:Details | k:Kill | t:Tree | s:Sort | /:Filter | [/]:Speed"
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

/// Renders the detailed process view
fn render_detail_view(
    stdout: &mut io::Stdout,
    app: &mut App,
    width: usize,
    height: usize,
) -> io::Result<()> {
    let details = match &app.process_details {
        Some(d) => d,
        None => {
            app.detail_view_mode = false;
            return Ok(());
        }
    };

    // Build content lines
    let mut lines: Vec<(Color, String)> = Vec::new();
    
    // Header section
    lines.push((Color::Yellow, format!("═══ Process Details: {} (PID: {}) ═══", details.name, details.pid)));
    lines.push((Color::Reset, String::new()));
    
    // Basic stats section
    lines.push((Color::Cyan, "── Basic Information ──".to_string()));
    lines.push((Color::White, format!("  Name:        {}", details.name)));
    lines.push((Color::White, format!("  PID:         {}", details.pid)));
    lines.push((Color::White, format!("  Path:        {}", details.path.as_deref().unwrap_or("<access denied>"))));
    lines.push((Color::White, format!("  Command:     {}", details.command_line.as_deref().unwrap_or("<access denied>"))));
    lines.push((Color::White, format!("  Priority:    {}", details.priority)));
    lines.push((Color::White, format!("  Uptime:      {}", format_uptime(details.uptime_seconds))));
    lines.push((Color::Reset, String::new()));
    
    // Resource stats
    lines.push((Color::Cyan, "── Resource Usage ──".to_string()));
    lines.push((Color::White, format!("  CPU:         {:.1}%", details.cpu_percent)));
    lines.push((Color::White, format!("  Memory:      {}", format_bytes(details.memory_bytes))));
    lines.push((Color::White, format!("  Threads:     {}", details.thread_count)));
    lines.push((Color::White, format!("  Handles:     {}", details.handle_count)));
    lines.push((Color::White, format!("  Disk Read:   {}", format_rate(details.disk_read_rate))));
    lines.push((Color::White, format!("  Disk Write:  {}", format_rate(details.disk_write_rate))));
    lines.push((Color::Reset, String::new()));
    
    // Network connections
    lines.push((Color::Cyan, format!("── Network Connections ({} TCP, {} UDP) ──", 
        details.tcp_connections.len(), details.udp_endpoints.len())));
    
    if details.tcp_connections.is_empty() && details.udp_endpoints.is_empty() {
        lines.push((Color::DarkGrey, "  No network connections".to_string()));
    } else {
        // TCP connections
        for conn in &details.tcp_connections {
            let line = format!("  TCP  {:>15}:{:<5} → {:>15}:{:<5}  [{}]",
                conn.local_addr, conn.local_port,
                conn.remote_addr, conn.remote_port,
                conn.state);
            let color = match conn.state.as_str() {
                "ESTABLISHED" => Color::Green,
                "LISTEN" => Color::Cyan,
                "TIME_WAIT" | "CLOSE_WAIT" => Color::Yellow,
                _ => Color::White,
            };
            lines.push((color, line));
        }
        // UDP endpoints
        for ep in &details.udp_endpoints {
            lines.push((Color::Magenta, format!("  UDP  {:>15}:{:<5} (listening)",
                ep.local_addr, ep.local_port)));
        }
    }
    lines.push((Color::Reset, String::new()));
    
    // Loaded modules
    lines.push((Color::Cyan, format!("── Loaded Modules ({}) ──", details.modules.len())));
    if details.modules.is_empty() {
        lines.push((Color::DarkGrey, "  No modules (access denied or system process)".to_string()));
    } else {
        for module in &details.modules {
            lines.push((Color::White, format!("  {:40} @ 0x{:016X}", 
                truncate_string(&module.name, 40), 
                module.base_address)));
            if !module.path.is_empty() && module.path != module.name {
                lines.push((Color::DarkGrey, format!("    {}", truncate_string(&module.path, width.saturating_sub(6)))));
            }
        }
    }
    
    // Calculate visible area
    let header_lines_count = 2;
    let footer_lines_count = 2;
    let visible_rows = height.saturating_sub(header_lines_count + footer_lines_count);
    
    // Clamp scroll offset
    let max_scroll = lines.len().saturating_sub(visible_rows);
    if app.detail_scroll_offset > max_scroll {
        app.detail_scroll_offset = max_scroll;
    }
    
    // Render header
    execute!(
        stdout,
        SetBackgroundColor(Color::DarkMagenta),
        SetForegroundColor(Color::White),
        Print(format!("{:width$}", " Process Details View", width = width)),
        ResetColor,
        Print("\r\n"),
        SetForegroundColor(Color::DarkGrey),
        Print(format!(" Scroll: {}/{} lines  |  ↑↓/PgUp/PgDn: Scroll  |  Esc/Enter: Close",
            app.detail_scroll_offset + 1, lines.len())),
        ResetColor,
        Print(format!("{:width$}\r\n", "", width = width.saturating_sub(70))),
    )?;
    
    // Render content lines
    for (i, (color, line)) in lines.iter()
        .skip(app.detail_scroll_offset)
        .take(visible_rows)
        .enumerate() 
    {
        let display_line = truncate_string(line, width.saturating_sub(1));
        execute!(
            stdout,
            SetForegroundColor(*color),
            Print(format!("{:width$}", display_line, width = width)),
            ResetColor,
            Print("\r\n")
        )?;
    }
    
    // Fill remaining space
    let lines_rendered = lines.len().saturating_sub(app.detail_scroll_offset).min(visible_rows);
    for _ in lines_rendered..visible_rows {
        execute!(stdout, Print(format!("{:width$}\r\n", "", width = width)))?;
    }
    
    // Footer
    execute!(
        stdout,
        Print("\r\n"),
        SetBackgroundColor(Color::DarkMagenta),
        SetForegroundColor(Color::White),
        Print(format!("{:width$}", " Esc/Enter: Back to process list  |  k: Kill process", width = width)),
        ResetColor,
    )?;
    
    stdout.flush()
}
