//! Process management operations (kill, suspend, priority)

use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

use super::state::App;
use super::ViewMode;
use crate::system::{is_process_suspended, set_process_priority, toggle_suspend};

impl App {
    /// Requests to kill the currently selected process (shows confirmation)
    pub fn request_kill(&mut self) {
        if self.filtered_processes.is_empty() {
            return;
        }

        let process = &self.filtered_processes[self.selected_index];
        self.pending_kill_pid = Some(process.info.pid);
        self.pending_kill_name = Some(process.info.name.clone());
        self.view_mode = ViewMode::ConfirmKill;
        self.error_message = Some(format!(
            "Kill {} (PID {})? Press Y to confirm, N to cancel",
            process.info.name, process.info.pid
        ));
    }

    /// Confirms and executes the pending kill
    pub fn confirm_kill(&mut self) {
        let pid = match self.pending_kill_pid {
            Some(p) => p,
            None => return,
        };
        let name = self.pending_kill_name.clone().unwrap_or_default();

        // SAFETY: OpenProcess and TerminateProcess are safe to call with valid params.
        let result = unsafe {
            let handle = OpenProcess(PROCESS_TERMINATE, false, pid);
            match handle {
                Ok(h) => {
                    let term_result = TerminateProcess(h, 1);
                    let _ = CloseHandle(h);
                    term_result
                }
                Err(e) => Err(e),
            }
        };

        match result {
            Ok(_) => {
                self.error_message = Some(format!("Terminated process: {} (PID {})", name, pid));
            }
            Err(e) => {
                self.error_message = Some(format!(
                    "Failed to terminate {} (PID {}): {}",
                    name, pid, e
                ));
            }
        }

        self.cancel_kill();
    }

    /// Cancels the pending kill
    pub fn cancel_kill(&mut self) {
        self.view_mode = ViewMode::ProcessList;
        self.pending_kill_pid = None;
        self.pending_kill_name = None;
    }

    /// Raises the priority of the selected process
    pub fn raise_priority(&mut self) {
        if self.filtered_processes.is_empty() {
            return;
        }

        let process = &self.filtered_processes[self.selected_index];
        let pid = process.info.pid;
        let name = process.info.name.clone();
        let current = process.priority;
        let new_priority = current.raise();

        if current == new_priority {
            self.error_message = Some(format!("{} is already at maximum priority", name));
            return;
        }

        match set_process_priority(pid, new_priority) {
            Ok(_) => {
                self.error_message = Some(format!(
                    "{}: {} → {}",
                    name, current.name(), new_priority.name()
                ));
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to raise priority: {}", e));
            }
        }
    }

    /// Lowers the priority of the selected process
    pub fn lower_priority(&mut self) {
        if self.filtered_processes.is_empty() {
            return;
        }

        let process = &self.filtered_processes[self.selected_index];
        let pid = process.info.pid;
        let name = process.info.name.clone();
        let current = process.priority;
        let new_priority = current.lower();

        if current == new_priority {
            self.error_message = Some(format!("{} is already at minimum priority", name));
            return;
        }

        match set_process_priority(pid, new_priority) {
            Ok(_) => {
                self.error_message = Some(format!(
                    "{}: {} → {}",
                    name, current.name(), new_priority.name()
                ));
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to lower priority: {}", e));
            }
        }
    }

    /// Toggle suspend/resume for the selected process
    pub fn toggle_suspend(&mut self) {
        if self.filtered_processes.is_empty() {
            return;
        }

        let process = &self.filtered_processes[self.selected_index];
        let pid = process.info.pid;
        let name = process.info.name.clone();

        match toggle_suspend(pid) {
            Ok(is_suspended) => {
                if is_suspended {
                    self.error_message = Some(format!("Suspended: {} (PID {})", name, pid));
                } else {
                    self.error_message = Some(format!("Resumed: {} (PID {})", name, pid));
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Failed: {}", e));
            }
        }
    }

    /// Check if a process is suspended
    pub fn is_process_suspended(&self, pid: u32) -> bool {
        is_process_suspended(pid)
    }
}
