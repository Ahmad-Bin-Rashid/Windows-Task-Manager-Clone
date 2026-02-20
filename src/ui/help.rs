//! Help overlay rendering

use std::io::{self, Write};

use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};

/// Help content definition
const HELP_LINES: &[(&str, &str)] = &[
    ("", ""),
    ("NAVIGATION", ""),
    ("  Up/Down", "Move selection up/down"),
    ("  PgUp/PgDn", "Scroll by page"),
    ("  Home/End", "Jump to first/last process"),
    ("  Enter", "View process details"),
    ("", ""),
    ("PROCESS ACTIONS", ""),
    ("  k", "Kill selected process"),
    ("  p", "Suspend/Resume process"),
    ("  +/-", "Raise/Lower priority"),
    ("", ""),
    ("VIEW OPTIONS", ""),
    ("  s", "Cycle sort column"),
    ("  r", "Reverse sort order"),
    ("  t", "Toggle tree view"),
    ("  /", "Filter by process name"),
    ("  Esc", "Clear filter"),
    ("", ""),
    ("SETTINGS", ""),
    ("  [", "Slow down refresh"),
    ("  ]", "Speed up refresh"),
    ("", ""),
    ("OTHER", ""),
    ("  e", "Export to CSV file"),
    ("  ?", "Show/hide this help"),
    ("  q", "Quit application"),
    ("  Ctrl+C", "Quit application"),
];

/// Renders the help overlay showing all keyboard shortcuts
pub fn render_help_overlay(
    stdout: &mut io::Stdout,
    width: usize,
    height: usize,
) -> io::Result<()> {
    // Calculate box dimensions - use fixed width for consistent borders
    let box_width = 52;
    let inner_width = box_width - 2; // Width between left and right borders
    let box_height = (HELP_LINES.len() + 4).min(height.saturating_sub(2));
    let start_x = (width.saturating_sub(box_width)) / 2;
    let start_y = (height.saturating_sub(box_height)) / 2;

    // Draw background fill for the whole screen (dimmed)
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

    // Helper to draw a line with consistent borders
    let draw_bordered_line = |stdout: &mut io::Stdout, y: usize, content: &str, fg: Color| -> io::Result<()> {
        execute!(stdout, MoveTo(start_x as u16, y as u16))?;
        // Pad or truncate content to exactly inner_width
        let padded = format!("{:<w$}", content, w = inner_width);
        let truncated = if padded.len() > inner_width {
            padded[..inner_width].to_string()
        } else {
            padded
        };
        execute!(
            stdout,
            SetBackgroundColor(Color::DarkBlue),
            SetForegroundColor(fg),
            Print("│"),
            Print(&truncated),
            Print("│"),
            ResetColor
        )
    };

    // Draw top border
    execute!(stdout, MoveTo(start_x as u16, start_y as u16))?;
    let top_border = format!("┌{}┐", "─".repeat(inner_width));
    execute!(
        stdout,
        SetBackgroundColor(Color::DarkBlue),
        SetForegroundColor(Color::White),
        Print(&top_border),
        ResetColor
    )?;

    // Draw title
    let title = "Keyboard Shortcuts";
    let title_padding = (inner_width.saturating_sub(title.len())) / 2;
    let title_line = format!(
        "{:>pad$}{}{:<rpad$}",
        "", title, "",
        pad = title_padding,
        rpad = inner_width - title_padding - title.len()
    );
    draw_bordered_line(stdout, start_y + 1, &title_line, Color::Yellow)?;

    // Draw separator
    execute!(stdout, MoveTo(start_x as u16, (start_y + 2) as u16))?;
    let sep_border = format!("├{}┤", "─".repeat(inner_width));
    execute!(
        stdout,
        SetBackgroundColor(Color::DarkBlue),
        SetForegroundColor(Color::White),
        Print(&sep_border),
        ResetColor
    )?;

    // Draw help content
    for (i, (key, desc)) in HELP_LINES.iter().enumerate() {
        let y = start_y + 3 + i;
        if y >= start_y + box_height - 1 {
            break;
        }

        execute!(stdout, MoveTo(start_x as u16, y as u16))?;

        if key.is_empty() && desc.is_empty() {
            // Empty line
            draw_bordered_line(stdout, y, "", Color::White)?;
        } else if desc.is_empty() {
            // Section header
            draw_bordered_line(stdout, y, &format!(" {}", key), Color::Cyan)?;
        } else {
            // Key + description: format with fixed columns
            let key_col = 14;
            
            execute!(stdout, MoveTo(start_x as u16, y as u16))?;
            execute!(
                stdout,
                SetBackgroundColor(Color::DarkBlue),
                SetForegroundColor(Color::White),
                Print("│"),
                SetForegroundColor(Color::Green),
                Print(&format!(" {:<kw$}", key, kw = key_col)),
                SetForegroundColor(Color::White),
                Print(&format!("{:<dw$}", desc, dw = inner_width - key_col - 1)),
                Print("│"),
                ResetColor
            )?;
        }
    }

    // Fill remaining space in box
    for y in (start_y + 3 + HELP_LINES.len())..(start_y + box_height - 1) {
        draw_bordered_line(stdout, y, "", Color::White)?;
    }

    // Draw bottom border with hint
    execute!(stdout, MoveTo(start_x as u16, (start_y + box_height - 1) as u16))?;
    let hint = " Press any key to close ";
    let hint_padding = (inner_width.saturating_sub(hint.len())) / 2;
    let bottom_left = "─".repeat(hint_padding);
    let bottom_right = "─".repeat(inner_width - hint_padding - hint.len());
    execute!(
        stdout,
        SetBackgroundColor(Color::DarkBlue),
        SetForegroundColor(Color::White),
        Print("└"),
        Print(&bottom_left),
        SetForegroundColor(Color::Yellow),
        Print(hint),
        SetForegroundColor(Color::White),
        Print(&bottom_right),
        Print("┘"),
        ResetColor
    )?;

    stdout.flush()
}
