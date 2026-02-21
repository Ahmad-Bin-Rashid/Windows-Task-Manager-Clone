//! CSV export functionality

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;

use windows::Win32::System::SystemInformation::GetLocalTime;

use crate::constants::BYTES_PER_MB;

use super::ProcessEntry;

/// Generates a timestamped filename for the export
fn generate_filename() -> String {
    let st = unsafe { GetLocalTime() };
    
    format!(
        "processes_{:04}-{:02}-{:02}_{:02}{:02}{:02}.csv",
        st.wYear, st.wMonth, st.wDay,
        st.wHour, st.wMinute, st.wSecond
    )
}

/// Escapes a string for CSV format
/// Wraps in quotes if contains comma, quote, or newline
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Exports the process list to a CSV file
/// Returns the path to the exported file on success
pub fn export_to_csv(processes: &[ProcessEntry]) -> io::Result<PathBuf> {
    let filename = generate_filename();
    let path = PathBuf::from(&filename);
    
    let file = File::create(&path)?;
    let mut writer = BufWriter::new(file);
    
    // Write CSV header
    writeln!(
        writer,
        "PID,Name,CPU%,Memory(MB),Threads,Priority,Handles,Uptime(s),DiskRead/s,DiskWrite/s,Path"
    )?;
    
    // Write each process
    for proc in processes {
        let name = escape_csv(&proc.info.name);
        let path_str = proc.path.as_deref().unwrap_or("");
        let path_escaped = escape_csv(path_str);
        
        writeln!(
            writer,
            "{},{},{:.2},{:.2},{},{},{},{},{:.0},{:.0},{}",
            proc.info.pid,
            name,
            proc.cpu_percent,
            proc.memory_bytes as f64 / BYTES_PER_MB,
            proc.thread_count,
            proc.priority.name(),
            proc.handle_count,
            proc.uptime_seconds,
            proc.disk_read_rate,
            proc.disk_write_rate,
            path_escaped,
        )?;
    }
    
    writer.flush()?;
    
    Ok(path)
}

impl super::App {
    /// Exports the current (filtered) process list to CSV
    pub fn export_processes(&mut self) {
        let processes = if self.filtered_processes.is_empty() && self.filter.is_empty() {
            &self.processes
        } else {
            &self.filtered_processes
        };
        
        match export_to_csv(processes) {
            Ok(path) => {
                self.error_message = Some(format!(
                    "Exported {} processes to {}",
                    processes.len(),
                    path.display()
                ));
            }
            Err(e) => {
                self.error_message = Some(format!("Export failed: {}", e));
            }
        }
    }
}
