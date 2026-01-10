//! CLI Windows Task Manager Clone
//!
//! A command-line task manager that displays running processes, CPU usage,
//! and memory statistics using raw Win32 API calls via the `windows` crate.
//!
//! Controls:
//! - q: Quit
//! - k: Kill selected process (with confirmation)
//! - +/-: Raise/lower process priority
//! - s: Cycle sort column
//! - r: Reverse sort order
//! - /: Filter by process name
//! - [: Slow down refresh rate
//! - ]: Speed up refresh rate
//! - ↑/↓: Navigate process list
//! - PgUp/PgDown: Scroll by page
//! - Home/End: Jump to start/end

mod app;
mod ffi;
mod system;
mod ui;

use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{
        self, DisableLineWrap, EnableLineWrap,
        EnterAlternateScreen, LeaveAlternateScreen,
    },
};

use app::App;
use ui::render;

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    // Set up terminal
    terminal::enable_raw_mode()?;
    execute!(
        stdout,
        EnterAlternateScreen,
        DisableLineWrap,
        Hide
    )?;

    // Create app state
    let mut app = App::new();

    // Initial refresh
    app.refresh();

    let mut last_refresh = Instant::now();

    // Main loop
    loop {
        // Render current state
        render(&mut stdout, &mut app)?;

        // Calculate dynamic refresh interval
        let refresh_interval = Duration::from_millis(app.refresh_interval_ms);

        // Check for events (with timeout for refresh)
        let timeout = refresh_interval
            .checked_sub(last_refresh.elapsed())
            .unwrap_or(Duration::ZERO);

        if event::poll(timeout)? {
            if let Event::Key(key_event) = event::read()? {
                // Only handle key PRESS events, ignore Release and Repeat
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }

                // Clear error message on any key press
                app.error_message = None;

                // Handle confirm kill mode
                if app.confirm_kill_mode {
                    handle_confirm_kill_keys(&mut app, key_event.code);
                    continue;
                }

                // Handle filter mode
                if app.filter_mode {
                    handle_filter_keys(&mut app, key_event.code);
                    continue;
                }

                // Handle normal mode
                if handle_normal_keys(&mut app, key_event.code, key_event.modifiers)? {
                    break;
                }
            }
        }

        // Time-based refresh (recalculate interval in case it changed)
        if last_refresh.elapsed() >= Duration::from_millis(app.refresh_interval_ms) {
            app.refresh();
            last_refresh = Instant::now();
        }
    }

    // Restore terminal
    execute!(
        stdout,
        Show,
        EnableLineWrap,
        LeaveAlternateScreen
    )?;
    terminal::disable_raw_mode()?;

    println!("Task Manager closed.");
    Ok(())
}

/// Handles key events in confirm kill mode
fn handle_confirm_kill_keys(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.confirm_kill();
            app.refresh();
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.cancel_kill();
        }
        _ => {}
    }
}

/// Handles key events in filter mode
fn handle_filter_keys(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.filter_mode = false;
        }
        KeyCode::Enter => {
            app.filter_mode = false;
            app.apply_filter();
        }
        KeyCode::Backspace => {
            app.filter.pop();
            app.apply_filter();
        }
        KeyCode::Char(c) => {
            app.filter.push(c);
            app.apply_filter();
        }
        _ => {}
    }
}

/// Handles key events in normal mode. Returns true if app should exit.
fn handle_normal_keys(
    app: &mut App,
    code: KeyCode,
    modifiers: KeyModifiers,
) -> io::Result<bool> {
    match code {
        KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(true),
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
        KeyCode::Char('k') | KeyCode::Char('K') => {
            app.request_kill();
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            app.raise_priority();
            app.refresh();
        }
        KeyCode::Char('-') | KeyCode::Char('_') => {
            app.lower_priority();
            app.refresh();
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            app.cycle_sort();
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.toggle_sort_order();
        }
        KeyCode::Char('[') => {
            app.increase_refresh_interval();
        }
        KeyCode::Char(']') => {
            app.decrease_refresh_interval();
        }
        KeyCode::Char('/') => {
            app.filter_mode = true;
        }
        KeyCode::Esc => {
            app.filter.clear();
            app.apply_filter();
        }
        KeyCode::Up => app.move_up(),
        KeyCode::Down => app.move_down(),
        KeyCode::PageUp => {
            let (_, h) = terminal::size()?;
            app.page_up((h as usize).saturating_sub(6));
        }
        KeyCode::PageDown => {
            let (_, h) = terminal::size()?;
            app.page_down((h as usize).saturating_sub(6));
        }
        KeyCode::Home => app.jump_to_start(),
        KeyCode::End => app.jump_to_end(),
        _ => {}
    }
    Ok(false)
}

