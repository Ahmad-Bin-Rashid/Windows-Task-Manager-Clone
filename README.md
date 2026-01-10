# Windows Task Manager Clone

A command-line Windows Task Manager clone built in Rust using raw Win32 API calls. This project demonstrates OS-level process management without relying on high-level abstractions.

## Features

- **Process Enumeration**: Lists all running processes with detailed information
- **Real-time CPU Monitoring**: Per-process and system-wide CPU usage
- **Memory Statistics**: Working set memory usage per process and system totals
- **Disk I/O Tracking**: Read/write bytes for each process
- **Thread Count**: Number of threads per process
- **Priority Management**: View and modify process priority levels
- **Interactive Controls**: Navigate, sort, filter, and kill processes
- **Color-Coded Display**: CPU usage is color-coded for quick identification
- **Kill Confirmation**: Safety dialog before terminating processes

## Requirements

- **Operating System**: Windows 10/11
- **Rust**: 1.70+ (2021 edition)
- **Build Tools**: MSYS2 with MinGW-w64 or Visual Studio Build Tools

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `windows` | 0.58 | Win32 API bindings |
| `crossterm` | 0.28 | Terminal UI rendering |

## Building

### Option 1: Using MSYS2 (MinGW-w64)

```powershell
# Set up PATH for MSYS2
$env:PATH = "C:\msys64\ucrt64\bin;C:\msys64\usr\bin;$env:PATH"

# Build release version
cargo build --release
```

### Option 2: Using Visual Studio Build Tools

```powershell
# Ensure MSVC toolchain is installed
rustup default stable-msvc

# Build release version
cargo build --release
```

### Output

The compiled binary will be at:
```
target\release\task_manager_clone.exe
```

## Usage

```powershell
.\target\release\task_manager_clone.exe
```

### Keyboard Controls

| Key | Action |
|-----|--------|
| `q` | Quit application |
| `Ctrl+C` | Quit application |
| `↑` / `↓` | Navigate process list |
| `PgUp` / `PgDown` | Scroll by page |
| `Home` / `End` | Jump to start/end |
| `k` | Kill selected process (with confirmation) |
| `Y` / `N` | Confirm/cancel kill |
| `+` / `=` | Raise process priority |
| `-` / `_` | Lower process priority |
| `s` | Cycle sort column |
| `r` | Reverse sort order |
| `/` | Enter filter mode |
| `Esc` | Clear filter / Cancel |

### Sort Columns

Press `s` to cycle through:
1. **CPU%** - CPU usage percentage
2. **Memory** - Working set memory
3. **Name** - Process name (alphabetical)
4. **PID** - Process ID
5. **Priority** - Process priority class
6. **Threads** - Thread count
7. **Disk Read** - Total bytes read
8. **Disk Write** - Total bytes written

Press `r` to toggle ascending/descending order.

### Filtering

1. Press `/` to enter filter mode
2. Type process name (case-insensitive)
3. Press `Enter` to apply or `Esc` to cancel
4. Press `Esc` again to clear the filter

## Project Structure

```
task-manager/
├── Cargo.toml              # Project dependencies
├── README.md               # This file
└── src/
    ├── main.rs             # Entry point & event loop
    ├── app/
    │   ├── mod.rs          # Application state & logic
    │   ├── process_entry.rs # Process data structure
    │   └── sort.rs         # Sorting options
    ├── ui/
    │   ├── mod.rs          # UI module exports
    │   └── render.rs       # Terminal rendering
    ├── ffi/
    │   ├── mod.rs          # FFI module exports
    │   └── handles.rs      # RAII handle wrappers
    └── system/
        ├── mod.rs          # System module exports
        ├── processes.rs    # Process enumeration
        ├── memory.rs       # Memory metrics
        ├── cpu.rs          # CPU tracking
        ├── disk.rs         # Disk I/O stats
        └── priority.rs     # Priority management
```

## Technical Details

### Win32 APIs Used

| Module | API | Purpose |
|--------|-----|---------|
| **Process Enumeration** | `CreateToolhelp32Snapshot` | Create snapshot of processes |
| | `Process32FirstW` / `Process32NextW` | Iterate through processes |
| **CPU Metrics** | `GetSystemTimes` | System-wide CPU times |
| | `GetProcessTimes` | Per-process CPU times |
| **Memory Metrics** | `GlobalMemoryStatusEx` | System memory info |
| | `GetProcessMemoryInfo` | Per-process memory |
| **Disk I/O** | `GetProcessIoCounters` | Read/write byte counts |
| **Priority** | `GetPriorityClass` | Get process priority |
| | `SetPriorityClass` | Modify process priority |
| **Process Control** | `OpenProcess` | Get process handle |
| | `TerminateProcess` | Kill a process |
| | `CloseHandle` | Release handles |

### CPU Calculation Method

CPU percentage is calculated using **time deltas** between refresh intervals:

```
                    Process Time Delta
CPU % = ────────────────────────────────── × 100
                    System Time Delta
```

Where:
- **Process Time Delta** = (Kernel + User time) change for the process
- **System Time Delta** = (Kernel + User time) change for the system

This matches Windows Task Manager's calculation method, showing 0-100% regardless of core count.

### Memory Metrics

| Metric | Description |
|--------|-------------|
| **Working Set** | Physical memory currently in use by the process |
| **System Memory** | Total and available physical RAM |
| **Memory Load %** | Percentage of physical memory in use |

### Priority Levels

| Priority | Value | Description |
|----------|-------|-------------|
| Idle | 4 | Runs only when CPU is idle |
| Below Normal | 6 | Lower than normal priority |
| Normal | 8 | Default priority |
| Above Normal | 10 | Higher than normal priority |
| High | 13 | Significantly elevated priority |
| Realtime | 24 | Highest priority (dangerous!) |

⚠️ **Warning**: Setting Realtime priority can freeze the system by starving critical OS processes.

### Disk I/O

- **Disk Read**: Cumulative bytes read since process start
- **Disk Write**: Cumulative bytes written since process start

These are total values, not rates. Long-running processes will show higher values.

### Thread Count

Shows the number of execution threads in each process. Limits depend on:
- Stack size (default 1 MB per thread)
- Available memory
- 32-bit processes: ~2000 threads max
- 64-bit processes: ~100,000+ threads max

## Display

### Column Layout

```
PID      Priority  Threads     Memory   CPU%   Disk Read  Disk Write  Name
```

### Color Coding (CPU)

| Color | CPU Usage |
|-------|-----------|
| 🟢 Green | < 20% |
| 🔵 Cyan | 20-50% |
| 🟡 Yellow | 50-80% |
| 🔴 Red | ≥ 80% |

### Status Bar

- Shows error messages and status updates
- Kill confirmation dialog appears here
- Help text with available commands

## Safe Rust Wrappers

All Win32 API calls use `unsafe` blocks wrapped in safe Rust abstractions:

```rust
// RAII handle wrapper - automatically calls CloseHandle on drop
pub struct SnapshotHandle(HANDLE);

impl Drop for SnapshotHandle {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0); }
    }
}
```

This ensures:
- Handles are always properly closed
- Memory safety is maintained
- No resource leaks

## Error Handling

- Failed API calls display error messages in the status bar
- Inaccessible processes (system processes) show 0% CPU and 0 bytes
- The application continues running even if individual process queries fail

## Refresh Rate

- Default: 5 seconds
- Configurable via `REFRESH_INTERVAL_MS` constant in `main.rs`

## Bookmarks
- Win32 API Index: https://learn.microsoft.com/en-us/windows/win32/apiindex/windows-api-list
- Process & Thread Reference: https://learn.microsoft.com/en-us/windows/win32/procthread/process-and-thread-reference
- windows-rs Docs: https://microsoft.github.io/windows-docs-rs/
- windows-rs GitHub: https://github.com/microsoft/windows-rs

## Limitations

1. **Elevated Processes**: Cannot query CPU/memory for some system processes without admin rights
2. **Disk I/O**: Shows cumulative totals, not real-time rates
3. **32-bit Processes**: Cannot query 64-bit process details from 32-bit build

## License

MIT License - See LICENSE file for details.

## Author

Created for OS Lab Project - Operating Systems Course

---

*Built with ❤️ using Rust and raw Win32 APIs*
