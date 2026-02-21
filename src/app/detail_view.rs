//! Detail view management

use crate::system::{
    get_process_affinity, get_process_command_line, get_process_modules,
    get_process_tcp_connections, get_process_udp_endpoints, ProcessDetails,
};

use super::state::App;
use super::ViewMode;

impl App {
    /// Opens detail view for the currently selected process
    pub fn open_detail_view(&mut self) {
        if self.filtered_processes.is_empty() {
            return;
        }

        let process = &self.filtered_processes[self.selected_index];
        let pid = process.info.pid;
        let name = process.info.name.clone();

        let modules = get_process_modules(pid);
        let tcp_connections = get_process_tcp_connections(pid);
        let udp_endpoints = get_process_udp_endpoints(pid);
        let command_line = get_process_command_line(pid);
        let cpu_affinity = get_process_affinity(pid).map(|a| a.format());

        let details = ProcessDetails {
            pid,
            name: name.clone(),
            path: process.path.clone(),
            command_line,
            modules,
            tcp_connections,
            udp_endpoints,
            cpu_percent: process.cpu_percent,
            memory_bytes: process.memory_bytes,
            thread_count: process.thread_count,
            handle_count: process.handle_count,
            priority: process.priority.short_name().to_string(),
            uptime_seconds: process.uptime_seconds,
            disk_read_rate: process.disk_read_rate,
            disk_write_rate: process.disk_write_rate,
            cpu_affinity,
        };

        self.view_mode = ViewMode::DetailView;
        self.detail_view_pid = Some(pid);
        self.detail_view_name = Some(name);
        self.detail_view_data = Some(details);
        self.detail_scroll_offset = 0;
    }

    /// Closes the detail view and returns to process list
    pub fn close_detail_view(&mut self) {
        self.view_mode = ViewMode::ProcessList;
        self.detail_view_pid = None;
        self.detail_view_name = None;
        self.detail_view_data = None;
        self.detail_scroll_offset = 0;
    }

    /// Refreshes the detail view data for the currently viewed process
    pub fn refresh_detail_view(&mut self) {
        let pid = match self.detail_view_pid {
            Some(pid) => pid,
            None => return,
        };

        // Find the process in the updated list to get current metrics
        let process = self.processes.iter().find(|p| p.info.pid == pid);
        
        if let Some(process) = process {
            let modules = get_process_modules(pid);
            let tcp_connections = get_process_tcp_connections(pid);
            let udp_endpoints = get_process_udp_endpoints(pid);
            let command_line = get_process_command_line(pid);
            let cpu_affinity = get_process_affinity(pid).map(|a| a.format());

            let details = ProcessDetails {
                pid,
                name: process.info.name.clone(),
                path: process.path.clone(),
                command_line,
                modules,
                tcp_connections,
                udp_endpoints,
                cpu_percent: process.cpu_percent,
                memory_bytes: process.memory_bytes,
                thread_count: process.thread_count,
                handle_count: process.handle_count,
                priority: process.priority.short_name().to_string(),
                uptime_seconds: process.uptime_seconds,
                disk_read_rate: process.disk_read_rate,
                disk_write_rate: process.disk_write_rate,
                cpu_affinity,
            };

            self.detail_view_data = Some(details);
        } else {
            // Process no longer exists - close detail view
            self.error_message = Some("Process no longer exists".to_string());
            self.close_detail_view();
        }
    }

    /// Scrolls the detail view down
    pub fn detail_scroll_down(&mut self) {
        if let Some(ref details) = self.detail_view_data {
            let total_lines = self.count_detail_lines(details);
            if self.detail_scroll_offset < total_lines.saturating_sub(10) {
                self.detail_scroll_offset += 1;
            }
        }
    }

    /// Scrolls the detail view up
    pub fn detail_scroll_up(&mut self) {
        if self.detail_scroll_offset > 0 {
            self.detail_scroll_offset -= 1;
        }
    }

    /// Pages the detail view down
    pub fn detail_page_down(&mut self, lines: usize) {
        if let Some(ref details) = self.detail_view_data {
            let total_lines = self.count_detail_lines(details);
            let max_offset = total_lines.saturating_sub(10);
            self.detail_scroll_offset = (self.detail_scroll_offset + lines).min(max_offset);
        }
    }

    /// Pages the detail view up
    pub fn detail_page_up(&mut self, lines: usize) {
        self.detail_scroll_offset = self.detail_scroll_offset.saturating_sub(lines);
    }

    /// Counts the total number of lines in detail view
    fn count_detail_lines(&self, details: &ProcessDetails) -> usize {
        let mut count = 7; // Header + basic info lines

        // Modules section
        count += 2; // Header + separator
        count += details.modules.len();

        // TCP section
        count += 2;
        count += details.tcp_connections.len();

        // UDP section
        count += 2;
        count += details.udp_endpoints.len();

        count
    }
}
