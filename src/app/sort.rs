//! Sorting options for the process list

/// Sort column options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortColumn {
    Cpu,
    Memory,
    Name,
    Pid,
    Priority,
    DiskRead,
    DiskWrite,
    Threads,
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
            SortColumn::Threads => SortColumn::DiskRead,
            SortColumn::DiskRead => SortColumn::DiskWrite,
            SortColumn::DiskWrite => SortColumn::Cpu,
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
            SortColumn::DiskRead => "Disk Read",
            SortColumn::DiskWrite => "Disk Write",
        }
    }
}
