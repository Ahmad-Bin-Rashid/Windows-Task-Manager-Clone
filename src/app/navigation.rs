//! Navigation methods for the application

use super::state::App;

impl App {
    /// Moves selection up
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Moves selection down
    pub fn move_down(&mut self) {
        if self.selected_index < self.filtered_processes.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    /// Page up
    pub fn page_up(&mut self, visible_rows: usize) {
        self.selected_index = self.selected_index.saturating_sub(visible_rows);
    }

    /// Page down
    pub fn page_down(&mut self, visible_rows: usize) {
        self.selected_index = (self.selected_index + visible_rows)
            .min(self.filtered_processes.len().saturating_sub(1));
    }

    /// Jump to start
    pub fn jump_to_start(&mut self) {
        self.selected_index = 0;
    }

    /// Jump to end
    pub fn jump_to_end(&mut self) {
        self.selected_index = self.filtered_processes.len().saturating_sub(1);
    }
}
