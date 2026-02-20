//! Input/keyboard event handling
//!
//! This module handles all keyboard input for different application modes:
//! - Normal mode (process list navigation)
//! - Filter mode (text input for filtering)
//! - Confirm kill mode (Y/N confirmation)
//! - Detail view mode (scrolling process details)

use std::io;

use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
use crossterm::terminal;

use super::App;

/// Result of handling a key event
pub enum KeyAction {
    /// Continue running the application
    Continue,
    /// Exit the application
    Exit,
}

impl App {
    /// Handles key events when help overlay is shown
    pub fn handle_help_key(&mut self, code: KeyCode) -> KeyAction {
        match code {
            // Any key closes help
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('?') | KeyCode::Char('q') => {
                self.show_help = false;
            }
            _ => {
                self.show_help = false;
            }
        }
        KeyAction::Continue
    }

    /// Handles key events in confirm kill mode
    pub fn handle_confirm_kill_key(&mut self, code: KeyCode) -> KeyAction {
        match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.confirm_kill();
                self.refresh();
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.cancel_kill();
            }
            _ => {}
        }
        KeyAction::Continue
    }

    /// Handles key events in filter mode
    pub fn handle_filter_key(&mut self, code: KeyCode) -> KeyAction {
        match code {
            KeyCode::Esc => {
                self.filter_mode = false;
            }
            KeyCode::Enter => {
                self.filter_mode = false;
                self.apply_filter();
            }
            KeyCode::Backspace => {
                self.filter.pop();
                self.apply_filter();
            }
            KeyCode::Char(c) => {
                self.filter.push(c);
                self.apply_filter();
            }
            _ => {}
        }
        KeyAction::Continue
    }

    /// Handles key events in detail view mode
    pub fn handle_detail_view_key(&mut self, code: KeyCode) -> io::Result<KeyAction> {
        match code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                self.close_detail_view();
            }
            KeyCode::Char('k') | KeyCode::Char('K') => {
                // Allow killing from detail view
                self.close_detail_view();
                self.request_kill();
            }
            KeyCode::Up => self.detail_scroll_up(),
            KeyCode::Down => self.detail_scroll_down(),
            KeyCode::PageUp => {
                let (_, h) = terminal::size()?;
                self.detail_page_up((h as usize).saturating_sub(6));
            }
            KeyCode::PageDown => {
                let (_, h) = terminal::size()?;
                self.detail_page_down((h as usize).saturating_sub(6));
            }
            KeyCode::Home => {
                self.detail_scroll_offset = 0;
            }
            KeyCode::End => {
                self.detail_scroll_offset = usize::MAX; // Will be clamped during render
            }
            _ => {}
        }
        Ok(KeyAction::Continue)
    }

    /// Handles key events in normal mode (process list).
    /// Returns `KeyAction::Exit` if the application should quit.
    pub fn handle_normal_key(
        &mut self,
        code: KeyCode,
        modifiers: KeyModifiers,
    ) -> io::Result<KeyAction> {
        match code {
            KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(KeyAction::Exit),
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                return Ok(KeyAction::Exit)
            }
            KeyCode::Char('k') | KeyCode::Char('K') => {
                self.request_kill();
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.toggle_suspend();
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.raise_priority();
                self.refresh();
            }
            KeyCode::Char('-') | KeyCode::Char('_') => {
                self.lower_priority();
                self.refresh();
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.cycle_sort();
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.toggle_sort_order();
            }
            KeyCode::Char('t') | KeyCode::Char('T') => {
                self.toggle_tree_view();
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                self.export_processes();
            }
            KeyCode::Char('[') => {
                self.increase_refresh_interval();
            }
            KeyCode::Char(']') => {
                self.decrease_refresh_interval();
            }
            KeyCode::Char('/') => {
                self.filter_mode = true;
            }
            KeyCode::Char('?') => {
                self.show_help = true;
            }
            KeyCode::Esc => {
                self.filter.clear();
                self.apply_filter();
            }
            KeyCode::Enter => {
                self.open_detail_view();
            }
            KeyCode::Up => self.move_up(),
            KeyCode::Down => self.move_down(),
            KeyCode::PageUp => {
                let (_, h) = terminal::size()?;
                self.page_up((h as usize).saturating_sub(6));
            }
            KeyCode::PageDown => {
                let (_, h) = terminal::size()?;
                self.page_down((h as usize).saturating_sub(6));
            }
            KeyCode::Home => self.jump_to_start(),
            KeyCode::End => self.jump_to_end(),
            _ => {}
        }
        Ok(KeyAction::Continue)
    }
}
