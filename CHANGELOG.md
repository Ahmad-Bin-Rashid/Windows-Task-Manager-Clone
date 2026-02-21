# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-02-21

### Added

- **Process Management**
  - Process enumeration using ToolHelp32 API
  - Kill process with confirmation dialog
  - Suspend/Resume processes using NtSuspendProcess/NtResumeProcess
  - Priority adjustment (Idle, Below Normal, Normal, Above Normal, High, Realtime)
  - CPU affinity viewing and modification

- **Monitoring**
  - Real-time CPU usage (per-process and system-wide)
  - Memory statistics (working set, system totals)
  - Disk I/O rates (read/write bytes per second)
  - Thread and handle counts
  - Process uptime tracking

- **Views**
  - Tree view mode (parent-child hierarchy)
  - Detail view (modules, TCP/UDP connections, command line)
  - Sortable columns (10 sort options)
  - Filter/search by process name
  - Help overlay with keyboard shortcuts

- **UI**
  - Color-coded CPU usage indicators
  - Administrator/User privilege indicator
  - Scrollable process list
  - Status bar with feedback messages

- **CLI**
  - `--refresh` - Set refresh interval
  - `--filter` - Initial process filter
  - `--sort` - Initial sort column
  - `--ascending` - Sort order
  - `--tree` - Start in tree view mode
  - `--export` - Export to CSV and exit
  - `--help` and `--version`

- **Export**
  - CSV export (interactive `e` key or `--export` flag)

### Technical

- Built with Rust 2021 edition
- Uses `windows` crate (0.58) for raw Win32 API calls
- Uses `crossterm` crate (0.28) for terminal rendering
- RAII wrappers for safe handle management
- Custom error types for type-safe error handling
- Modular architecture (app, system, ui, ffi modules)
