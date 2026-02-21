//! Help overlay rendering

use std::io::{self, Write};

use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};

use crate::app::App;
use crate::constants::{HELP_DIALOG_WIDTH, HELP_KEY_COL_WIDTH};

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
    ("  a", "Set CPU affinity (in detail view)"),
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

/// Renders the help overlay showing all keyboard shortcuts.
///
/// Displays a centered dialog with available keybindings organized
/// by category: Navigation, Process Actions, View Options, Settings,
/// and Other. Supports scrolling when content exceeds window height.
pub fn render_help_overlay(
    stdout: &mut io::Stdout,
    app: &App,
    width: usize,
    height: usize,
) -> io::Result<()> {
    // Calculate box dimensions - use fixed width for consistent borders
    let box_width = HELP_DIALOG_WIDTH;
    let inner_width = box_width - 2; // Width between left and right borders
    
    // Calculate available content height (minus borders, title, separator, footer hint)
    let max_content_lines = height.saturating_sub(8); // 8 = top border + title + separator + footer hint + margins
    let content_height = HELP_LINES.len().min(max_content_lines);
    let box_height = content_height + 5; // +5 for borders and header/footer
    
    let start_x = (width.saturating_sub(box_width)) / 2;
    let start_y = (height.saturating_sub(box_height)) / 2;
    
    // Clamp scroll offset to valid range
    let max_scroll = HELP_LINES.len().saturating_sub(content_height);
    let scroll_offset = app.help_scroll_offset.min(max_scroll);

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

    // Draw help content with scroll offset
    let visible_lines: Vec<_> = HELP_LINES.iter()
        .skip(scroll_offset)
        .take(content_height)
        .collect();
    
    for (i, (key, desc)) in visible_lines.iter().enumerate() {
        let y = start_y + 3 + i;

        if key.is_empty() && desc.is_empty() {
            // Empty line
            draw_bordered_line(stdout, y, "", Color::White)?;
        } else if desc.is_empty() {
            // Section header
            draw_bordered_line(stdout, y, &format!(" {}", key), Color::Cyan)?;
        } else {
            // Key + description: format with fixed columns
            let key_col = HELP_KEY_COL_WIDTH;
            
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
    for y in (start_y + 3 + visible_lines.len())..(start_y + box_height - 1) {
        draw_bordered_line(stdout, y, "", Color::White)?;
    }

    // Draw bottom border with scroll hint
    execute!(stdout, MoveTo(start_x as u16, (start_y + box_height - 1) as u16))?;
    let hint = if max_scroll > 0 {
        format!(" ↑/↓: Scroll | Esc: Close ({}/{}) ", scroll_offset + 1, max_scroll + 1)
    } else {
        " Esc/Enter: Close ".to_string()
    };
    // Calculate display width (Unicode arrows are 1 column each, not their byte length)
    let hint_display_width = hint.chars().count();
    let hint_padding = (inner_width.saturating_sub(hint_display_width)) / 2;
    let bottom_left = "─".repeat(hint_padding);
    let bottom_right = "─".repeat(inner_width.saturating_sub(hint_padding + hint_display_width));
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
