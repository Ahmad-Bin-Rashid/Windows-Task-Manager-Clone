//! Process list rendering

use std::io;

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};

use crate::app::App;
use crate::constants::MAX_TREE_INDENT_DEPTH;
use crate::system::{format_bytes, format_uptime};

use super::utils::{cpu_color, format_rate, truncate_string};

/// Renders the scrollable process list.
///
/// Displays process information including PID, priority, threads, handles,
/// uptime, memory, CPU usage, disk I/O rates, and process name.
/// Highlights the currently selected process and shows tree indentation
/// when tree view mode is enabled.
pub fn render_process_list(
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
        let cpu_col = cpu_color(entry.cpu_percent);

        // Add tree indentation if in tree view mode
        let tree_prefix = if app.tree_view_mode && entry.tree_depth > 0 {
            let indent = "  ".repeat(entry.tree_depth.min(MAX_TREE_INDENT_DEPTH));
            format!("{}└─", indent)
        } else {
            String::new()
        };
        
        // Check if process is suspended
        let is_suspended = app.is_process_suspended(entry.info.pid);
        let suspend_indicator = if is_suspended { "[S] " } else { "" };
        
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
        
        // Calculate available space for name with tree prefix and suspend indicator
        let name_space = width.saturating_sub(90 + tree_prefix.len() + suspend_indicator.len());
        let suffix = format!(
            "  {:>9}  {:>9}  {}{}{}",
            format_rate(entry.disk_read_rate),
            format_rate(entry.disk_write_rate),
            tree_prefix,
            suspend_indicator,
            truncate_string(&entry.info.name, name_space)
        );

        if is_selected {
            // Selected row - use background color, CPU still colored
            execute!(
                stdout,
                SetBackgroundColor(Color::DarkCyan),
                SetForegroundColor(Color::White),
                Print(&prefix),
                SetForegroundColor(cpu_col),
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
                SetForegroundColor(cpu_col),
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
