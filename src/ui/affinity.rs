//! CPU affinity dialog rendering

use std::io::{self, Write};

use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};

use crate::app::App;

/// Renders the CPU affinity dialog overlay
pub fn render_affinity_dialog(
    stdout: &mut io::Stdout,
    app: &App,
    width: usize,
    height: usize,
) -> io::Result<()> {
    let name = app.affinity_name.as_deref().unwrap_or("Unknown");
    let pid = app.affinity_pid.unwrap_or(0);
    let total_cores = app.affinity_total_cores as usize;

    // Calculate dialog dimensions
    // Each core takes about 12 chars: "[X] Core N  "
    // Display 4 cores per row
    let cores_per_row = 4.min(total_cores);
    let num_rows = (total_cores + cores_per_row - 1) / cores_per_row;
    
    let box_width = 60.min(width.saturating_sub(4));
    let box_height = (num_rows + 8).min(height.saturating_sub(4)); // +8 for header, footer, padding
    let start_x = (width.saturating_sub(box_width)) / 2;
    let start_y = (height.saturating_sub(box_height)) / 2;

    // Draw dimmed background
    for y in 0..height {
        execute!(stdout, MoveTo(0, y as u16))?;
        execute!(
            stdout,
            SetBackgroundColor(Color::Black),
            SetForegroundColor(Color::DarkGrey),
            Print(format!("{:w$}", "", w = width)),
            ResetColor
        )?;
    }

    let inner_width = box_width - 2;

    // Helper to draw a bordered line
    let draw_line = |stdout: &mut io::Stdout, y: usize, content: &str, fg: Color, bg: Color| -> io::Result<()> {
        execute!(stdout, MoveTo(start_x as u16, y as u16))?;
        let truncated = if content.len() > inner_width {
            &content[..inner_width]
        } else {
            content
        };
        let padded = format!("{:<w$}", truncated, w = inner_width);
        execute!(
            stdout,
            SetBackgroundColor(bg),
            SetForegroundColor(Color::White),
            Print("│"),
            SetForegroundColor(fg),
            Print(&padded),
            SetForegroundColor(Color::White),
            Print("│"),
            ResetColor
        )
    };

    let mut y = start_y;

    // Top border
    execute!(stdout, MoveTo(start_x as u16, y as u16))?;
    execute!(
        stdout,
        SetBackgroundColor(Color::DarkBlue),
        SetForegroundColor(Color::White),
        Print("┌"),
        Print("─".repeat(inner_width)),
        Print("┐"),
        ResetColor
    )?;
    y += 1;

    // Title
    let title = format!(" Set CPU Affinity: {} (PID: {}) ", name, pid);
    draw_line(stdout, y, &title, Color::Yellow, Color::DarkBlue)?;
    y += 1;

    // Separator
    execute!(stdout, MoveTo(start_x as u16, y as u16))?;
    execute!(
        stdout,
        SetBackgroundColor(Color::DarkBlue),
        SetForegroundColor(Color::White),
        Print("├"),
        Print("─".repeat(inner_width)),
        Print("┤"),
        ResetColor
    )?;
    y += 1;

    // Current selection info
    let selected_count = app.affinity_mask.count_ones();
    let info = format!(" Selected: {}/{} cores", selected_count, total_cores);
    draw_line(stdout, y, &info, Color::Cyan, Color::DarkBlue)?;
    y += 1;

    // Empty line
    draw_line(stdout, y, "", Color::White, Color::DarkBlue)?;
    y += 1;

    // Render cores in a grid
    for row in 0..num_rows {
        let mut line = String::from(" ");
        
        for col in 0..cores_per_row {
            let core_idx = row * cores_per_row + col;
            if core_idx >= total_cores {
                break;
            }

            let is_selected = app.is_core_selected(core_idx);
            let is_cursor = core_idx == app.affinity_selected_core;

            let checkbox = if is_selected { "[X]" } else { "[ ]" };
            
            // Build the core label
            let core_label = if is_cursor {
                format!(">{}Core {:<2}", checkbox, core_idx)
            } else {
                format!(" {}Core {:<2}", checkbox, core_idx)
            };

            line.push_str(&format!("{:<14}", core_label));
        }

        // Render the line with proper coloring
        execute!(stdout, MoveTo(start_x as u16, y as u16))?;
        execute!(
            stdout,
            SetBackgroundColor(Color::DarkBlue),
            SetForegroundColor(Color::White),
            Print("│"),
        )?;

        // Render each core with appropriate color
        let mut char_pos = 0;
        for col in 0..cores_per_row {
            let core_idx = row * cores_per_row + col;
            if core_idx >= total_cores {
                break;
            }

            let is_selected = app.is_core_selected(core_idx);
            let is_cursor = core_idx == app.affinity_selected_core;

            let checkbox = if is_selected { "[X]" } else { "[ ]" };
            let prefix = if is_cursor { ">" } else { " " };
            
            // Cursor highlight
            if is_cursor {
                execute!(stdout, SetBackgroundColor(Color::DarkCyan))?;
            } else {
                execute!(stdout, SetBackgroundColor(Color::DarkBlue))?;
            }

            // Checkbox color
            let checkbox_color = if is_selected { Color::Green } else { Color::DarkGrey };
            
            execute!(
                stdout,
                SetForegroundColor(Color::Yellow),
                Print(prefix),
                SetForegroundColor(checkbox_color),
                Print(checkbox),
                SetForegroundColor(Color::White),
                Print(format!("Core {:<2}  ", core_idx)),
            )?;
            
            char_pos += 14;
        }

        // Fill remaining space to align right border
        // char_pos is total chars rendered, add 5 for leading space, subtract from inner_width
        let remaining = inner_width.saturating_sub(char_pos + 1) + 5;
        execute!(
            stdout,
            SetBackgroundColor(Color::DarkBlue),
            Print(format!("{:w$}", "", w = remaining)),
            SetForegroundColor(Color::White),
            Print("│"),
            ResetColor
        )?;
        y += 1;
    }

    // Empty line
    draw_line(stdout, y, "", Color::White, Color::DarkBlue)?;
    y += 1;

    // Separator
    execute!(stdout, MoveTo(start_x as u16, y as u16))?;
    execute!(
        stdout,
        SetBackgroundColor(Color::DarkBlue),
        SetForegroundColor(Color::White),
        Print("├"),
        Print("─".repeat(inner_width)),
        Print("┤"),
        ResetColor
    )?;
    y += 1;

    // Help line 1
    let help1 = " ←/→: Select   Space: Toggle   A: All   N: None";
    draw_line(stdout, y, help1, Color::DarkGrey, Color::DarkBlue)?;
    y += 1;

    // Help line 2
    let help2 = " Enter: Apply   Esc: Cancel";
    draw_line(stdout, y, help2, Color::DarkGrey, Color::DarkBlue)?;
    y += 1;

    // Bottom border
    execute!(stdout, MoveTo(start_x as u16, y as u16))?;
    execute!(
        stdout,
        SetBackgroundColor(Color::DarkBlue),
        SetForegroundColor(Color::White),
        Print("└"),
        Print("─".repeat(inner_width)),
        Print("┘"),
        ResetColor
    )?;

    stdout.flush()
}
