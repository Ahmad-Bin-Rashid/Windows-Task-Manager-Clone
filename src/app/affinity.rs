//! CPU affinity dialog management

use crate::system::affinity::{get_process_affinity, get_system_core_count, set_process_affinity};

use super::state::App;

impl App {
    /// Opens the affinity dialog for the currently viewed process
    pub fn open_affinity_dialog(&mut self) {
        let pid = match self.detail_view_pid {
            Some(pid) => pid,
            None => return,
        };

        let name = self.detail_view_name.clone().unwrap_or_default();

        // Get current affinity
        let current_affinity = get_process_affinity(pid);
        let total_cores = get_system_core_count();

        let current_mask = match current_affinity {
            Some(aff) => aff.process_mask,
            None => {
                self.error_message = Some("Cannot read process affinity".to_string());
                return;
            }
        };

        self.affinity_mode = true;
        self.affinity_pid = Some(pid);
        self.affinity_name = Some(name);
        self.affinity_mask = current_mask;
        self.affinity_total_cores = total_cores;
        self.affinity_selected_core = 0;
    }

    /// Closes the affinity dialog without applying changes
    pub fn close_affinity_dialog(&mut self) {
        self.affinity_mode = false;
        self.affinity_pid = None;
        self.affinity_name = None;
        self.affinity_mask = 0;
        self.affinity_total_cores = 0;
        self.affinity_selected_core = 0;
    }

    /// Toggles the selected core in the affinity mask
    pub fn toggle_affinity_core(&mut self) {
        let core = self.affinity_selected_core;
        if core < self.affinity_total_cores as usize {
            // Toggle the bit
            self.affinity_mask ^= 1 << core;
            
            // Ensure at least one core is selected
            if self.affinity_mask == 0 {
                // Re-enable this core - can't have zero cores
                self.affinity_mask |= 1 << core;
                self.error_message = Some("At least one core must be selected".to_string());
            }
        }
    }

    /// Selects all cores in affinity dialog
    pub fn select_all_cores(&mut self) {
        let all_mask = (1usize << self.affinity_total_cores) - 1;
        self.affinity_mask = all_mask;
    }

    /// Deselects all cores except the first one
    pub fn select_single_core(&mut self) {
        self.affinity_mask = 1; // Only core 0
    }

    /// Moves selection left in affinity dialog
    pub fn affinity_move_left(&mut self) {
        if self.affinity_selected_core > 0 {
            self.affinity_selected_core -= 1;
        }
    }

    /// Moves selection right in affinity dialog
    pub fn affinity_move_right(&mut self) {
        if self.affinity_selected_core < (self.affinity_total_cores as usize).saturating_sub(1) {
            self.affinity_selected_core += 1;
        }
    }

    /// Applies the affinity changes
    pub fn apply_affinity(&mut self) {
        let pid = match self.affinity_pid {
            Some(pid) => pid,
            None => {
                self.close_affinity_dialog();
                return;
            }
        };

        match set_process_affinity(pid, self.affinity_mask) {
            Ok(()) => {
                let count = self.affinity_mask.count_ones();
                self.error_message = Some(format!(
                    "Set affinity to {} core{}",
                    count,
                    if count == 1 { "" } else { "s" }
                ));
                self.close_affinity_dialog();
                // Refresh detail view to show new affinity
                self.refresh_detail_view();
            }
            Err(e) => {
                self.error_message = Some(e);
            }
        }
    }

    /// Checks if a core is selected in the current affinity mask
    pub fn is_core_selected(&self, core: usize) -> bool {
        (self.affinity_mask >> core) & 1 == 1
    }
}
