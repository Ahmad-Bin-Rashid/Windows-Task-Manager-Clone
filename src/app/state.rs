//! Application state and core logic

use std::collections::HashMap;
use std::time::Instant;

use crate::system::cpu::CpuTracker;
use crate::system::details::ProcessDetails;
use crate::system::disk::get_process_disk_info;
use crate::system::memory::get_process_memory_info;
use crate::system::path::{get_process_handle_count, get_process_path};
use crate::system::priority::get_process_priority;
use crate::system::processes::enumerate_processes;
use crate::system::uptime::{calculate_uptime_seconds, get_process_start_time};

use super::{ProcessEntry, SortColumn};

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
    /// Refresh interval in milliseconds
    pub refresh_interval_ms: u64,
    /// Whether we're viewing process details
    pub detail_view_mode: bool,
    /// PID of process in detail view
    pub detail_view_pid: Option<u32>,
    /// Name of process in detail view
    pub detail_view_name: Option<String>,
    /// Cached process details for the detail view
    pub detail_view_data: Option<ProcessDetails>,
    /// Scroll offset for detail view
    pub detail_scroll_offset: usize,
    /// Whether we're in tree view mode
    pub tree_view_mode: bool,
    /// Whether to show the help overlay
    pub show_help: bool,
    /// Whether we're in affinity edit mode
    pub affinity_mode: bool,
    /// PID of process being edited for affinity
    pub affinity_pid: Option<u32>,
    /// Name of process being edited for affinity
    pub affinity_name: Option<String>,
    /// Current core selection bitmask for affinity dialog
    pub affinity_mask: usize,
    /// Total number of system cores
    pub affinity_total_cores: u32,
    /// Currently selected core index in affinity dialog
    pub affinity_selected_core: usize,
    /// Scroll offset for help overlay
    pub help_scroll_offset: usize,
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
            refresh_interval_ms: 2000,
            detail_view_mode: false,
            detail_view_pid: None,
            detail_view_name: None,
            detail_view_data: None,
            detail_scroll_offset: 0,
            tree_view_mode: false,
            show_help: false,
            affinity_mode: false,
            affinity_pid: None,
            affinity_name: None,
            affinity_mask: 0,
            affinity_total_cores: 0,
            affinity_selected_core: 0,
            help_scroll_offset: 0,
        }
    }

    /// Creates a new App instance configured with command-line arguments
    pub fn with_args(args: &super::cli::Args) -> Self {
        let mut app = Self::new();
        
        // Apply CLI configuration
        app.refresh_interval_ms = args.refresh;
        app.sort_column = args.sort;
        app.sort_ascending = args.ascending;
        app.tree_view_mode = args.tree;
        
        if let Some(ref filter) = args.filter {
            app.filter = filter.clone();
        }
        
        app
    }

    /// Increase refresh interval (slower refresh)
    pub fn increase_refresh_interval(&mut self) {
        self.refresh_interval_ms = match self.refresh_interval_ms {
            x if x >= 10000 => 10000,
            x if x >= 5000 => 10000,
            x if x >= 2000 => 5000,
            x if x >= 1000 => 2000,
            x if x >= 500 => 1000,
            _ => 500,
        };
    }

    /// Decrease refresh interval (faster refresh)
    pub fn decrease_refresh_interval(&mut self) {
        self.refresh_interval_ms = match self.refresh_interval_ms {
            x if x <= 500 => 250,
            x if x <= 1000 => 500,
            x if x <= 2000 => 1000,
            x if x <= 5000 => 2000,
            x if x <= 10000 => 5000,
            _ => 5000,
        };
    }

    /// Format refresh interval for display
    pub fn format_refresh_interval(&self) -> String {
        if self.refresh_interval_ms >= 1000 {
            format!("{:.1}s", self.refresh_interval_ms as f64 / 1000.0)
        } else {
            format!("{}ms", self.refresh_interval_ms)
        }
    }

    /// Refreshes the process list and metrics
    pub fn refresh(&mut self) {
        let now = Instant::now();
        let time_delta = now.duration_since(self.last_refresh_time).as_secs_f64();
        self.last_refresh_time = now;

        self.system_cpu = self.cpu_tracker.get_system_cpu_usage();

        let processes = match enumerate_processes() {
            Ok(procs) => procs,
            Err(e) => {
                self.error_message = Some(format!("Failed to enumerate processes: {}", e));
                return;
            }
        };

        let mut new_disk_io: HashMap<u32, DiskIoSnapshot> = HashMap::new();

        self.processes = processes
            .into_iter()
            .map(|info| {
                let pid = info.pid;
                let cpu_percent = self.cpu_tracker.get_process_cpu_usage(pid);
                let mem_info = get_process_memory_info(pid);
                let disk_info = get_process_disk_info(pid);
                let priority = get_process_priority(pid);
                let thread_count = info.thread_count;

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

                new_disk_io.insert(
                    pid,
                    DiskIoSnapshot {
                        read_bytes: disk_info.read_bytes,
                        write_bytes: disk_info.write_bytes,
                    },
                );

                let start_time = get_process_start_time(pid);
                let uptime_seconds = start_time
                    .map(|st| calculate_uptime_seconds(st))
                    .unwrap_or(0);

                let path = get_process_path(pid);
                let handle_count = get_process_handle_count(pid);

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
                    path,
                    handle_count,
                    tree_depth: 0,
                }
            })
            .collect();

        self.prev_disk_io = new_disk_io;
        
        // Apply sorting/tree structure and filtering
        if self.tree_view_mode {
            // Tree view handles its own structure and calls apply_filter
            self.build_process_tree();
        } else {
            self.sort_processes();
            self.apply_filter();
        }

        let active_pids: Vec<u32> = self.processes.iter().map(|p| p.info.pid).collect();
        self.cpu_tracker.cleanup_stale_processes(&active_pids);

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
                SortColumn::Cpu => b
                    .cpu_percent
                    .partial_cmp(&a.cpu_percent)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::Memory => b.memory_bytes.cmp(&a.memory_bytes),
                SortColumn::Name => a.info.name.to_lowercase().cmp(&b.info.name.to_lowercase()),
                SortColumn::Pid => a.info.pid.cmp(&b.info.pid),
                SortColumn::Priority => b.priority.cmp(&a.priority),
                SortColumn::Threads => b.thread_count.cmp(&a.thread_count),
                SortColumn::Handles => b.handle_count.cmp(&a.handle_count),
                SortColumn::Uptime => b.uptime_seconds.cmp(&a.uptime_seconds),
                SortColumn::DiskReadRate => b
                    .disk_read_rate
                    .partial_cmp(&a.disk_read_rate)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::DiskWriteRate => b
                    .disk_write_rate
                    .partial_cmp(&a.disk_write_rate)
                    .unwrap_or(std::cmp::Ordering::Equal),
            };
            if ascending {
                cmp.reverse()
            } else {
                cmp
            }
        });
    }

    /// Apply the current filter to the process list
    pub fn apply_filter(&mut self) {
        self.filtered_processes = if self.filter.is_empty() {
            self.processes.clone()
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.processes
                .iter()
                .filter(|p| p.info.name.to_lowercase().contains(&filter_lower))
                .cloned()
                .collect()
        };

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
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
