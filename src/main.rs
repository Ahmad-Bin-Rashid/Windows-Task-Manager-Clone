//! CLI Windows Task Manager Clone
//!
//! A command-line task manager that displays running processes, CPU usage,
//! and memory statistics using raw Win32 API calls via the `windows` crate.
//!
//! # Usage
//!
//! ```
//! task_manager_cli [OPTIONS]
//!
//! Options:
//!   -r, --refresh <MS>    Refresh interval in milliseconds [default: 2000]
//!   -f, --filter <NAME>   Initial filter string to match process names
//!   -s, --sort <COLUMN>   Initial sort column [default: cpu]
//!   -a, --ascending       Sort in ascending order (default is descending)
//!   -t, --tree            Start in tree view mode
//!   -h, --help            Print help
//!   -V, --version         Print version
//! ```
//!
//! # Examples
//!
//! ```
//! # Start with default settings
//! task_manager_cli
//!
//! # Start with 500ms refresh rate
//! task_manager_cli -r 500
//!
//! # Start filtered to chrome processes, sorted by memory
//! task_manager_cli -f chrome -s memory
//!
//! # Start in tree view mode
//! task_manager_cli --tree
//! ```
//!
//! # Controls
//!
//! | Key | Action |
//! |-----|--------|
//! | `q` | Quit |
//! | `Enter` | View process details |
//! | `k` | Kill selected process (with confirmation) |
//! | `p` | Suspend/Resume selected process |
//! | `t` | Toggle tree view (show parent-child hierarchy) |
//! | `+`/`-` | Raise/lower process priority |
//! | `s` | Cycle sort column |
//! | `r` | Reverse sort order |
//! | `/` | Filter by process name |
//! | `[`/`]` | Slow down/speed up refresh rate |
//! | `↑`/`↓` | Navigate process list |
//! | `PgUp`/`PgDn` | Scroll by page |
//! | `Home`/`End` | Jump to start/end |
//! | `?` | Show help overlay |

mod app;
mod constants;
mod ffi;
mod system;
mod ui;

use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{
        DisableLineWrap, EnableLineWrap,
        EnterAlternateScreen, LeaveAlternateScreen,
    },
};

use app::{export_to_csv, parse_args, App, KeyAction, ViewMode};
use ui::render;

fn main() -> io::Result<()> {
    // Parse command-line arguments
    let args = parse_args();
    let mut app = App::with_args(&args);
    
    // Handle export mode (non-interactive)
    if args.export {
        return run_export_mode(&mut app);
    }
    
    // Set up terminal and run main loop
    setup_terminal()?;
    let result = run_event_loop(&mut app);
    restore_terminal()?;
    
    println!("Task Manager closed.");
    result
}

/// Runs in export mode: loads processes, exports to CSV, and exits
fn run_export_mode(app: &mut App) -> io::Result<()> {
    // Load process data
    app.refresh();
    
    // Get the appropriate process list (filtered or all)
    let processes = if app.filtered_processes.is_empty() && app.filter.is_empty() {
        &app.processes
    } else {
        &app.filtered_processes
    };
    
    // Export to CSV
    match export_to_csv(processes) {
        Ok(path) => {
            println!("Exported {} processes to {}", processes.len(), path.display());
            Ok(())
        }
        Err(e) => {
            eprintln!("Export failed: {}", e);
            Err(e)
        }
    }
}

/// Configures the terminal for TUI mode
fn setup_terminal() -> io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    execute!(
        io::stdout(),
        EnterAlternateScreen,
        DisableLineWrap,
        Hide
    )
}

/// Restores the terminal to normal mode
fn restore_terminal() -> io::Result<()> {
    execute!(
        io::stdout(),
        Show,
        EnableLineWrap,
        LeaveAlternateScreen
    )?;
    crossterm::terminal::disable_raw_mode()
}

/// Main event loop - handles rendering and input
fn run_event_loop(app: &mut App) -> io::Result<()> {
    let mut stdout = io::stdout();
    let mut last_refresh = Instant::now();
    
    // Initial data load
    app.refresh();

    loop {
        // Render current state
        render(&mut stdout, app)?;

        // Calculate timeout until next refresh
        let refresh_interval = Duration::from_millis(app.refresh_interval_ms);
        let timeout = refresh_interval
            .checked_sub(last_refresh.elapsed())
            .unwrap_or(Duration::ZERO);

        // Poll for input events
        if event::poll(timeout)? {
            if let Event::Key(key_event) = event::read()? {
                // Only handle key PRESS events, ignore Release and Repeat
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }

                // Clear error message on any key press
                app.error_message = None;

                // Dispatch to appropriate handler based on current mode
                let action = dispatch_key_event(app, key_event.code, key_event.modifiers)?;
                
                if matches!(action, KeyAction::Exit) {
                    break;
                }
            }
        }

        // Time-based refresh
        if last_refresh.elapsed() >= Duration::from_millis(app.refresh_interval_ms) {
            app.refresh();
            
            // Also refresh detail view if active
            if app.view_mode.is_detail_view() {
                app.refresh_detail_view();
            }
            
            last_refresh = Instant::now();
        }
    }

    Ok(())
}

/// Dispatches key events to the appropriate handler based on app mode
fn dispatch_key_event(
    app: &mut App,
    code: crossterm::event::KeyCode,
    modifiers: crossterm::event::KeyModifiers,
) -> io::Result<KeyAction> {
    match app.view_mode {
        ViewMode::Help => Ok(app.handle_help_key(code)),
        ViewMode::Affinity => Ok(app.handle_affinity_key(code)),
        ViewMode::ConfirmKill => Ok(app.handle_confirm_kill_key(code)),
        ViewMode::DetailView => app.handle_detail_view_key(code),
        ViewMode::FilterInput => Ok(app.handle_filter_key(code)),
        ViewMode::ProcessList => app.handle_normal_key(code, modifiers),
    }
}

