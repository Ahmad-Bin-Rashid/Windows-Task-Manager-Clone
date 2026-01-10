//! Sorting options for the process list

/// Sort column options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortColumn {
    Cpu,
    Memory,
    Name,
    Pid,
    Priority,
    Threads,
    Handles,
    Uptime,
    DiskReadRate,
    DiskWriteRate,
}

impl SortColumn {
    /// Cycle to the next sort option
    pub fn next(self) -> Self {
        match self {
            SortColumn::Cpu => SortColumn::Memory,
            SortColumn::Memory => SortColumn::Name,
            SortColumn::Name => SortColumn::Pid,
            SortColumn::Pid => SortColumn::Priority,
            SortColumn::Priority => SortColumn::Threads,
            SortColumn::Threads => SortColumn::Handles,
            SortColumn::Handles => SortColumn::Uptime,
            SortColumn::Uptime => SortColumn::DiskReadRate,
            SortColumn::DiskReadRate => SortColumn::DiskWriteRate,
            SortColumn::DiskWriteRate => SortColumn::Cpu,
        }
    }

    /// Get display name for the sort column
    pub fn name(&self) -> &'static str {
        match self {
            SortColumn::Cpu => "CPU%",
            SortColumn::Memory => "Memory",
            SortColumn::Name => "Name",
            SortColumn::Pid => "PID",
            SortColumn::Priority => "Priority",
            SortColumn::Threads => "Threads",
            SortColumn::Handles => "Handles",
            SortColumn::Uptime => "Uptime",
            SortColumn::DiskReadRate => "Read/s",
            SortColumn::DiskWriteRate => "Write/s",
        }
    }
}
