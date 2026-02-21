# Windows Task Manager CLI

A feature-rich command-line Windows Task Manager built in Rust using raw Win32 API calls. This project demonstrates OS-level process management without relying on high-level abstractions.

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)](https://www.rust-lang.org/)
[![Windows](https://img.shields.io/badge/Platform-Windows%2010%2F11-blue)](https://www.microsoft.com/windows)
[![License](https://img.shields.io/badge/License-MIT-green)](LICENSE)

## Features

### Process Management
- **Process Enumeration** - List all running processes with detailed information
- **Kill Process** - Terminate processes with confirmation dialog
- **Suspend/Resume** - Pause and resume process execution
- **Priority Control** - View and modify process priority levels (Idle → Realtime)
- **CPU Affinity** - View and set which CPU cores a process can use

### Monitoring
- **Real-time CPU Usage** - Per-process and system-wide CPU percentage
- **Memory Statistics** - Working set memory per process and system totals
- **Disk I/O Rates** - Read/write bytes per second for each process
- **Thread & Handle Count** - Resource usage metrics
- **Process Uptime** - How long each process has been running

### Views & Navigation
- **Tree View** - Display processes in parent-child hierarchy
- **Detail View** - In-depth process info (modules, TCP/UDP connections, command line)
- **Sortable Columns** - Sort by any column, ascending or descending
- **Filter/Search** - Filter processes by name (case-insensitive)
- **Scrollable List** - Navigate large process lists with keyboard

### UI Features
- **Color-Coded CPU** - Visual indication of CPU usage levels
- **Admin Indicator** - Shows if running with elevated privileges
- **Help Overlay** - Scrollable help screen with all keyboard shortcuts
- **Export to CSV** - Save process list for external analysis

## Requirements

- **Operating System**: Windows 10/11 (64-bit recommended)
- **Rust**: 1.70+ (2021 edition)
- **Build Tools**: Visual Studio Build Tools or MSYS2 with MinGW-w64

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `windows` | 0.58 | Raw Win32 API bindings |
| `crossterm` | 0.28 | Cross-platform terminal rendering |

## Building

### Using Visual Studio Build Tools (Recommended)

```powershell
# Ensure MSVC toolchain is installed
rustup default stable-msvc

# Build release version
cargo build --release
```

### Using MSYS2 (MinGW-w64)

```powershell
# Set up PATH for MSYS2
$env:PATH = "C:\msys64\ucrt64\bin;C:\msys64\usr\bin;$env:PATH"

# Build release version
cargo build --release
```

### Output

```
target\release\task_manager_cli.exe
```

## Usage

### Basic Usage

```powershell
.\target\release\task_manager_cli.exe
```

### Command Line Options

```
task_manager_cli [OPTIONS]

Options:
  -r, --refresh <MS>    Refresh interval in milliseconds [default: 2000]
  -f, --filter <NAME>   Initial filter string to match process names
  -s, --sort <COLUMN>   Initial sort column [default: cpu]
  -a, --ascending       Sort in ascending order (default is descending)
  -t, --tree            Start in tree view mode
  -e, --export          Export process list to CSV and exit
  -h, --help            Print help information
  -V, --version         Print version
```

### Examples

```powershell
# Start with 500ms refresh rate
.\task_manager_cli.exe -r 500

# Start filtered to chrome processes, sorted by memory
.\task_manager_cli.exe -f chrome -s memory

# Start in tree view mode
.\task_manager_cli.exe --tree

# Export current processes to CSV
.\task_manager_cli.exe --export
```

## Keyboard Controls

### Navigation

| Key | Action |
|-----|--------|
| `↑` / `k` | Move selection up |
| `↓` / `j` | Move selection down |
| `PgUp` / `PgDn` | Scroll by page |
| `Home` / `End` | Jump to first/last process |

### Process Actions

| Key | Action |
|-----|--------|
| `Enter` / `d` | Open detail view for selected process |
| `k` | Kill selected process (with confirmation) |
| `p` | Suspend/Resume selected process |
| `+` / `=` | Raise process priority |
| `-` / `_` | Lower process priority |
| `a` | Open CPU affinity editor |

### View Controls

| Key | Action |
|-----|--------|
| `s` | Cycle sort column |
| `r` | Reverse sort order |
| `t` | Toggle tree view mode |
| `/` | Enter filter mode |
| `Esc` | Exit filter/detail/dialog |
| `?` | Show help overlay |
| `e` | Export to CSV |
| `q` / `Ctrl+C` | Quit application |

### Sort Columns

Press `s` to cycle through:
1. **CPU%** - CPU usage percentage
2. **Memory** - Working set memory
3. **Name** - Process name (alphabetical)
4. **PID** - Process ID
5. **Priority** - Process priority class
6. **Threads** - Thread count
7. **Handles** - Handle count
8. **Uptime** - Process running time
9. **Read/s** - Disk read rate
10. **Write/s** - Disk write rate

## Project Structure

```
task-manager/
├── Cargo.toml              # Project configuration & dependencies
├── README.md               # This file
├── LICENSE                 # MIT License
└── src/
    ├── main.rs             # Entry point & main event loop
    ├── constants.rs        # Centralized configuration constants
    ├── app/
    │   ├── mod.rs          # Module exports
    │   ├── state.rs        # Application state & refresh logic
    │   ├── cli.rs          # Command-line argument parsing
    │   ├── input.rs        # Keyboard event handling
    │   ├── navigation.rs   # List navigation methods
    │   ├── process_entry.rs# Process data structure
    │   ├── process_ops.rs  # Kill, suspend, priority operations
    │   ├── sort.rs         # Sorting options enum
    │   ├── view_mode.rs    # View state enum
    │   ├── tree_builder.rs # Process tree hierarchy
    │   ├── detail_view.rs  # Detail view logic
    │   ├── affinity.rs     # CPU affinity dialog logic
    │   └── export.rs       # CSV export functionality
    ├── system/
    │   ├── mod.rs          # Module exports
    │   ├── processes.rs    # Process enumeration (ToolHelp32)
    │   ├── cpu.rs          # CPU usage tracking
    │   ├── memory.rs       # Memory metrics
    │   ├── disk.rs         # Disk I/O statistics
    │   ├── priority.rs     # Priority get/set
    │   ├── suspend.rs      # Suspend/resume (NtSuspendProcess)
    │   ├── affinity.rs     # CPU affinity get/set
    │   ├── uptime.rs       # Process uptime calculation
    │   ├── path.rs         # Process path & handle count
    │   ├── details.rs      # Modules, TCP/UDP connections
    │   ├── admin.rs        # Elevation status detection
    │   └── error.rs        # Custom error types
    ├── ui/
    │   ├── mod.rs          # Module exports
    │   ├── render.rs       # Main render coordinator
    │   ├── components.rs   # Header, footer, stats bar
    │   ├── process_list.rs # Process list rendering
    │   ├── detail_view.rs  # Detail view rendering
    │   ├── affinity.rs     # Affinity dialog rendering
    │   ├── help.rs         # Help overlay rendering
    │   └── utils.rs        # Color helpers, formatting
    └── ffi/
        ├── mod.rs          # Module exports
        └── handles.rs      # RAII wrappers for Win32 handles
```

## Technical Details

### Win32 APIs Used

| Category | API | Purpose |
|----------|-----|---------|
| **Process Enumeration** | `CreateToolhelp32Snapshot` | Snapshot of all processes |
| | `Process32FirstW` / `Process32NextW` | Iterate processes |
| **CPU Metrics** | `GetSystemTimes` | System-wide CPU times |
| | `GetProcessTimes` | Per-process CPU times |
| **Memory** | `GlobalMemoryStatusEx` | System memory info |
| | `GetProcessMemoryInfo` | Per-process memory |
| **Disk I/O** | `GetProcessIoCounters` | Read/write byte counts |
| **Priority** | `GetPriorityClass` / `SetPriorityClass` | Priority management |
| **Suspend/Resume** | `NtSuspendProcess` / `NtResumeProcess` | Undocumented ntdll APIs |
| **Affinity** | `GetProcessAffinityMask` / `SetProcessAffinityMask` | CPU core assignment |
| **Modules** | `EnumProcessModules` / `GetModuleFileNameExW` | Loaded DLLs |
| **Network** | `GetExtendedTcpTable` / `GetExtendedUdpTable` | TCP/UDP connections |
| **Admin** | `OpenProcessToken` / `GetTokenInformation` | Elevation detection |
| **Handles** | `OpenProcess` / `CloseHandle` | Handle management |

### CPU Calculation

CPU percentage uses time deltas between refresh intervals:

```
                    Process Time Delta
CPU % = ────────────────────────────────── × 100
                    System Time Delta
```

This matches Windows Task Manager's behavior, showing 0-100% regardless of core count.

### Priority Levels

| Priority | Class | Description |
|----------|-------|-------------|
| Idle | 4 | Runs only when system is idle |
| Below Normal | 6 | Lower than normal |
| Normal | 8 | Default priority |
| Above Normal | 10 | Higher than normal |
| High | 13 | Significantly elevated |
| Realtime | 24 | Maximum priority (⚠️ dangerous!) |

### Color Coding

| Color | CPU Usage |
|-------|-----------|
| 🟢 Green | &lt; 20% |
| 🔵 Cyan | 20-50% |
| 🟡 Yellow | 50-80% |
| 🔴 Red | ≥ 80% |

## Safe Rust Patterns

All Win32 API calls use `unsafe` blocks wrapped in safe Rust abstractions:

```rust
// RAII handle wrapper - CloseHandle called automatically on drop
pub struct ProcessHandle(HANDLE);

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        unsafe { let _ = CloseHandle(self.0); }
    }
}

// Custom error types for type-safe error handling
pub enum ProcessError {
    AccessDenied,
    SystemProcess,
    InvalidHandle,
    // ...
}
```

## Limitations

1. **System Processes** - Cannot query some protected processes without admin rights
2. **32-bit Builds** - Cannot access 64-bit process details
3. **Realtime Priority** - Setting this can freeze the system

## Resources

- [Win32 API Index](https://learn.microsoft.com/en-us/windows/win32/apiindex/windows-api-list)
- [Process & Thread Reference](https://learn.microsoft.com/en-us/windows/win32/procthread/process-and-thread-reference)
- [windows-rs Documentation](https://microsoft.github.io/windows-docs-rs/)
- [windows-rs GitHub](https://github.com/microsoft/windows-rs)

## License

MIT License - See [LICENSE](LICENSE) file for details.

## Author

Created for OS Lab Project - Operating Systems Course

---

*Built with Rust and raw Win32 APIs*
