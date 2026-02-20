//! Process detail view rendering

use std::io::{self, Write};

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};

use crate::app::App;
use crate::system::memory::format_bytes;
use crate::system::uptime::format_uptime;

use super::utils::{format_rate, truncate_string};

/// Renders the detailed process view
pub fn render_detail_view(
    stdout: &mut io::Stdout,
    app: &mut App,
    width: usize,
    height: usize,
) -> io::Result<()> {
    let details = match &app.detail_view_data {
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
    lines.push((Color::White, format!("  CPU Affinity: {}", 
        details.cpu_affinity.as_deref().unwrap_or("Unknown"))));
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
    for (_i, (color, line)) in lines.iter()
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
        Print(format!("{:width$}", " Esc/Enter: Back to process list  |  k: Kill process  |  a: CPU affinity", width = width)),
        ResetColor,
    )?;
    
    stdout.flush()
}
