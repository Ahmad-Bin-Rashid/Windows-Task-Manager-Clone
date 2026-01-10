//! Application state and logic

mod process_entry;
mod sort;

pub use process_entry::ProcessEntry;
pub use sort::SortColumn;

use std::collections::HashMap;
use std::time::Instant;

use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

use crate::system::cpu::CpuTracker;
use crate::system::disk::get_process_disk_info;
use crate::system::memory::get_process_memory_info;
use crate::system::priority::{get_process_priority, set_process_priority};
use crate::system::processes::enumerate_processes;
use crate::system::uptime::{get_process_start_time, calculate_uptime_seconds};

/// Previous disk I/O snapshot for rate calculation
#[derive(Debug, Clone, Default)]
struct DiskIoSnapshot {
    read_bytes: u64,
    write_bytes: u64,
}

/// Application state
pub struct App {
    /// All tracked processes
    pub processes: Vec<ProcessEntry>,
    /// Filtered processes (after applying search filter)
    pub filtered_processes: Vec<ProcessEntry>,
    /// CPU usage tracker (stores previous measurements for delta calculation)
    pub cpu_tracker: CpuTracker,
    /// Currently selected process index
    pub selected_index: usize,
    /// Scroll offset for the process list
    pub scroll_offset: usize,
    /// System CPU usage percentage
    pub system_cpu: f64,
    /// Error message to display (if any)
    pub error_message: Option<String>,
    /// Current sort column
    pub sort_column: SortColumn,
    /// Sort in ascending order (false = descending)
    pub sort_ascending: bool,
    /// Search filter string
    pub filter: String,
    /// Whether we're in filter input mode
    pub filter_mode: bool,
    /// Whether we're waiting for kill confirmation
    pub confirm_kill_mode: bool,
    /// PID of process pending kill confirmation
    pub pending_kill_pid: Option<u32>,
    /// Name of process pending kill confirmation
    pub pending_kill_name: Option<String>,
    /// Previous disk I/O values for rate calculation
    prev_disk_io: HashMap<u32, DiskIoSnapshot>,
    /// Time of last refresh for rate calculation
    last_refresh_time: Instant,
}

impl App {
    /// Creates a new App instance
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            filtered_processes: Vec::new(),
            cpu_tracker: CpuTracker::new(),
            selected_index: 0,
            scroll_offset: 0,
            system_cpu: 0.0,
            error_message: None,
            sort_column: SortColumn::Cpu,
            sort_ascending: false,
            filter: String::new(),
            filter_mode: false,
            confirm_kill_mode: false,
            pending_kill_pid: None,
            pending_kill_name: None,
            prev_disk_io: HashMap::new(),
            last_refresh_time: Instant::now(),
        }
    }

    /// Refreshes the process list and metrics
    pub fn refresh(&mut self) {
        // Calculate time delta for rate calculations
        let now = Instant::now();
        let time_delta = now.duration_since(self.last_refresh_time).as_secs_f64();
        self.last_refresh_time = now;

        // Get system CPU usage
        self.system_cpu = self.cpu_tracker.get_system_cpu_usage();

        // Enumerate all processes
        let processes = match enumerate_processes() {
            Ok(procs) => procs,
            Err(e) => {
                self.error_message = Some(format!("Failed to enumerate processes: {}", e));
                return;
            }
        };

        // Build new disk I/O map for this refresh
        let mut new_disk_io: HashMap<u32, DiskIoSnapshot> = HashMap::new();

        // Build process entries with CPU and memory info
        self.processes = processes
            .into_iter()
            .map(|info| {
                let pid = info.pid;
                let cpu_percent = self.cpu_tracker.get_process_cpu_usage(pid);
                let mem_info = get_process_memory_info(pid);
                let disk_info = get_process_disk_info(pid);
                let priority = get_process_priority(pid);
                let thread_count = info.thread_count;

                // Calculate disk I/O rates
                let (disk_read_rate, disk_write_rate) = if time_delta > 0.0 {
                    if let Some(prev) = self.prev_disk_io.get(&pid) {
                        let read_delta = disk_info.read_bytes.saturating_sub(prev.read_bytes);
                        let write_delta = disk_info.write_bytes.saturating_sub(prev.write_bytes);
                        (
                            read_delta as f64 / time_delta,
                            write_delta as f64 / time_delta,
                        )
                    } else {
                        (0.0, 0.0)
                    }
                } else {
                    (0.0, 0.0)
                };

                // Store current disk I/O for next calculation
                new_disk_io.insert(pid, DiskIoSnapshot {
                    read_bytes: disk_info.read_bytes,
                    write_bytes: disk_info.write_bytes,
                });

                // Get process uptime
                let start_time = get_process_start_time(pid);
                let uptime_seconds = start_time
                    .map(|st| calculate_uptime_seconds(st))
                    .unwrap_or(0);

                ProcessEntry {
                    info,
                    cpu_percent,
                    memory_bytes: mem_info.working_set,
                    disk_read: disk_info.read_bytes,
                    disk_write: disk_info.write_bytes,
                    disk_read_rate,
                    disk_write_rate,
                    priority,
                    thread_count,
                    start_time,
                    uptime_seconds,
                }
            })
            .collect();

        // Update previous disk I/O map
        self.prev_disk_io = new_disk_io;

        // Sort based on selected column
        self.sort_processes();

        // Apply filter
        self.apply_filter();

        // Clean up stale process entries from CPU tracker
        let active_pids: Vec<u32> = self.processes.iter().map(|p| p.info.pid).collect();
        self.cpu_tracker.cleanup_stale_processes(&active_pids);

        // Ensure selection is within bounds
        if self.selected_index >= self.filtered_processes.len() {
            self.selected_index = self.filtered_processes.len().saturating_sub(1);
        }
    }

    /// Sorts processes based on current sort column and order
    fn sort_processes(&mut self) {
        let ascending = self.sort_ascending;
        let sort_column = self.sort_column;
        
        self.processes.sort_by(|a, b| {
            let cmp = match sort_column {
                SortColumn::Cpu => b.cpu_percent
                    .partial_cmp(&a.cpu_percent)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::Memory => b.memory_bytes.cmp(&a.memory_bytes),
                SortColumn::Name => a.info.name.to_lowercase().cmp(&b.info.name.to_lowercase()),
                SortColumn::Pid => a.info.pid.cmp(&b.info.pid),
                SortColumn::Priority => b.priority.cmp(&a.priority),
                SortColumn::Threads => b.thread_count.cmp(&a.thread_count),
                SortColumn::Uptime => b.uptime_seconds.cmp(&a.uptime_seconds),
                SortColumn::DiskReadRate => b.disk_read_rate
                    .partial_cmp(&a.disk_read_rate)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::DiskWriteRate => b.disk_write_rate
                    .partial_cmp(&a.disk_write_rate)
                    .unwrap_or(std::cmp::Ordering::Equal),
            };
            if ascending { cmp.reverse() } else { cmp }
        });
    }

    /// Apply the current filter to the process list
    pub fn apply_filter(&mut self) {
        if self.filter.is_empty() {
            self.filtered_processes = self.processes.clone();
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.filtered_processes = self.processes
                .iter()
                .filter(|p| p.info.name.to_lowercase().contains(&filter_lower))
                .cloned()
                .collect();
        }

        // Reset selection if out of bounds
        if self.selected_index >= self.filtered_processes.len() {
            self.selected_index = self.filtered_processes.len().saturating_sub(1);
        }
    }

    /// Toggle to next sort column
    pub fn cycle_sort(&mut self) {
        self.sort_column = self.sort_column.next();
        self.sort_processes();
        self.apply_filter();
    }

    /// Toggle sort order between ascending and descending
    pub fn toggle_sort_order(&mut self) {
        self.sort_ascending = !self.sort_ascending;
        self.sort_processes();
        self.apply_filter();
    }

    /// Requests to kill the currently selected process (shows confirmation)
    pub fn request_kill(&mut self) {
        if self.filtered_processes.is_empty() {
            return;
        }

        let process = &self.filtered_processes[self.selected_index];
        self.pending_kill_pid = Some(process.info.pid);
        self.pending_kill_name = Some(process.info.name.clone());
        self.confirm_kill_mode = true;
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
        self.confirm_kill_mode = false;
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

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
