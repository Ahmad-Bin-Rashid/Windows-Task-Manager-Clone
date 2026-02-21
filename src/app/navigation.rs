//! Navigation methods for the application

use super::state::App;

impl App {
    /// Moves selection up by one row.
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Moves selection down by one row.
    pub fn move_down(&mut self) {
        if self.selected_index < self.filtered_processes.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    /// Moves selection up by one page.
    ///
    /// # Arguments
    /// * `visible_rows` - Number of rows visible in the current view
    pub fn page_up(&mut self, visible_rows: usize) {
        self.selected_index = self.selected_index.saturating_sub(visible_rows);
    }

    /// Moves selection down by one page.
    ///
    /// # Arguments
    /// * `visible_rows` - Number of rows visible in the current view
    pub fn page_down(&mut self, visible_rows: usize) {
        self.selected_index = (self.selected_index + visible_rows)
            .min(self.filtered_processes.len().saturating_sub(1));
    }

    /// Jumps selection to the first process.
    pub fn jump_to_start(&mut self) {
        self.selected_index = 0;
    }

    /// Jumps selection to the last process.
    pub fn jump_to_end(&mut self) {
        self.selected_index = self.filtered_processes.len().saturating_sub(1);
    }
}
