//! Command-line argument parsing (manual implementation)

use std::env;
use std::process;

use crate::constants::{APP_NAME, APP_VERSION, DEFAULT_REFRESH_MS, MAX_REFRESH_MS, MIN_REFRESH_MS};

use super::SortColumn;

/// Parsed command-line arguments
#[derive(Debug)]
pub struct Args {
    /// Refresh interval in milliseconds
    pub refresh: u64,
    /// Initial filter string to match process names
    pub filter: Option<String>,
    /// Initial sort column
    pub sort: SortColumn,
    /// Sort in ascending order (default is descending)
    pub ascending: bool,
    /// Start in tree view mode
    pub tree: bool,
    /// Export to CSV and exit (non-interactive mode)
    pub export: bool,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            refresh: DEFAULT_REFRESH_MS,
            filter: None,
            sort: SortColumn::Cpu,
            ascending: false,
            tree: false,
            export: false,
        }
    }
}

/// Print help message and exit
fn print_help() {
    println!(
        "{} {}
A command-line Windows Task Manager built with Rust and raw Win32 API calls.

USAGE:
    {} [OPTIONS]

OPTIONS:
    -r, --refresh <MS>     Refresh interval in milliseconds [default: 2000]
                           Range: 250-10000
    -f, --filter <NAME>    Initial filter string to match process names
    -s, --sort <COLUMN>    Initial sort column [default: cpu]
                           Values: cpu, memory, name, pid, priority,
                                   threads, handles, uptime, read, write
    -a, --ascending        Sort in ascending order (default is descending)
    -t, --tree             Start in tree view mode
    -x, --export           Export to CSV and exit (non-interactive)
    -h, --help             Print help information
    -V, --version          Print version information

EXAMPLES:
    {}                          Start with default settings
    {} -r 500                   Start with 500ms refresh rate  
    {} -f chrome -s memory      Filter to chrome, sort by memory
    {} --tree                   Start in tree view mode
    {} --export                 Export all processes to CSV
    {} -f svchost --export      Export filtered processes to CSV

CONTROLS:
    q         Quit
    Enter     View process details
    k         Kill selected process
    p         Suspend/Resume process
    t         Toggle tree view
    +/-       Raise/lower priority
    s         Cycle sort column
    r         Reverse sort order
    /         Filter by name
    [/]       Slow down/speed up refresh
    ?         Show help overlay",
        APP_NAME, APP_VERSION, APP_NAME, APP_NAME, APP_NAME, APP_NAME, APP_NAME, APP_NAME, APP_NAME
    );
    process::exit(0);
}

/// Print version and exit
fn print_version() {
    println!("{} {}", APP_NAME, APP_VERSION);
    process::exit(0);
}

/// Print error message and exit
fn print_error(msg: &str) -> ! {
    eprintln!("error: {}", msg);
    eprintln!("For more information, try '--help'");
    process::exit(1);
}

/// Parse sort column from string
fn parse_sort(s: &str) -> SortColumn {
    match s.to_lowercase().as_str() {
        "cpu" => SortColumn::Cpu,
        "memory" | "mem" => SortColumn::Memory,
        "name" => SortColumn::Name,
        "pid" => SortColumn::Pid,
        "priority" | "prio" => SortColumn::Priority,
        "threads" => SortColumn::Threads,
        "handles" => SortColumn::Handles,
        "uptime" => SortColumn::Uptime,
        "read" | "disk-read" => SortColumn::DiskReadRate,
        "write" | "disk-write" => SortColumn::DiskWriteRate,
        _ => print_error(&format!(
            "invalid sort column '{}'. Valid values: cpu, memory, name, pid, priority, threads, handles, uptime, read, write",
            s
        )),
    }
}

/// Parse refresh interval from string
fn parse_refresh(s: &str) -> u64 {
    match s.parse::<u64>() {
        Ok(ms) if ms >= MIN_REFRESH_MS && ms <= MAX_REFRESH_MS => ms,
        Ok(ms) => print_error(&format!(
            "refresh interval {} is out of range. Must be between {} and {} ms",
            ms, MIN_REFRESH_MS, MAX_REFRESH_MS
        )),
        Err(_) => print_error(&format!("invalid refresh interval '{}'. Must be a number", s)),
    }
}

/// Parse command-line arguments
pub fn parse_args() -> Args {
    let mut args = Args::default();
    let mut argv: Vec<String> = env::args().skip(1).collect();
    
    while !argv.is_empty() {
        let arg = argv.remove(0);
        
        match arg.as_str() {
            "-h" | "--help" => print_help(),
            "-V" | "--version" => print_version(),
            "-a" | "--ascending" => args.ascending = true,
            "-t" | "--tree" => args.tree = true,
            "-x" | "--export" => args.export = true,
            
            "-r" | "--refresh" => {
                if argv.is_empty() {
                    print_error("--refresh requires a value");
                }
                args.refresh = parse_refresh(&argv.remove(0));
            }
            
            "-f" | "--filter" => {
                if argv.is_empty() {
                    print_error("--filter requires a value");
                }
                args.filter = Some(argv.remove(0));
            }
            
            "-s" | "--sort" => {
                if argv.is_empty() {
                    print_error("--sort requires a value");
                }
                args.sort = parse_sort(&argv.remove(0));
            }
            
            // Handle combined short flags like -at or -ta
            s if s.starts_with('-') && !s.starts_with("--") && s.len() > 2 => {
                // Split into individual flags and re-queue
                for c in s[1..].chars() {
                    argv.insert(0, format!("-{}", c));
                }
            }
            
            // Handle --key=value syntax
            s if s.starts_with("--") && s.contains('=') => {
                let parts: Vec<&str> = s.splitn(2, '=').collect();
                let key = parts[0];
                let value = parts[1];
                
                match key {
                    "--refresh" => args.refresh = parse_refresh(value),
                    "--filter" => args.filter = Some(value.to_string()),
                    "--sort" => args.sort = parse_sort(value),
                    _ => print_error(&format!("unknown option '{}'", key)),
                }
            }
            
            s if s.starts_with('-') => {
                print_error(&format!("unknown option '{}'", s));
            }
            
            s => {
                print_error(&format!("unexpected argument '{}'", s));
            }
        }
    }
    
    args
}
