//! Utility functions for UI rendering

use crossterm::style::Color;

use crate::constants::{
    BYTES_PER_KB, BYTES_PER_MB, BYTES_PER_GB,
    CPU_THRESHOLD_CRITICAL, CPU_THRESHOLD_WARNING, CPU_THRESHOLD_MODERATE,
};

/// Truncates a string to fit within a given width.
///
/// If the string exceeds `max_len`, it is truncated and "..." is appended.
///
/// # Arguments
/// * `s` - The string to truncate
/// * `max_len` - Maximum character length for the output
///
/// # Returns
/// The original string if it fits, or a truncated version with "..." suffix
#[must_use]
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len > 3 {
        format!("{}...", &s[..max_len - 3])
    } else {
        s[..max_len].to_string()
    }
}

/// Formats a byte rate (bytes/sec) as a human-readable string.
///
/// Automatically selects appropriate unit (B/s, KB/s, MB/s, GB/s).
///
/// # Arguments
/// * `bytes_per_sec` - Transfer rate in bytes per second
///
/// # Returns
/// Formatted string with appropriate unit suffix
#[must_use]
pub fn format_rate(bytes_per_sec: f64) -> String {
    if bytes_per_sec < 1.0 {
        "0 B/s".to_string()
    } else if bytes_per_sec < BYTES_PER_KB {
        format!("{:.0} B/s", bytes_per_sec)
    } else if bytes_per_sec < BYTES_PER_MB {
        format!("{:.1} KB/s", bytes_per_sec / BYTES_PER_KB)
    } else if bytes_per_sec < BYTES_PER_GB {
        format!("{:.1} MB/s", bytes_per_sec / BYTES_PER_MB)
    } else {
        format!("{:.1} GB/s", bytes_per_sec / BYTES_PER_GB)
    }
}

/// Returns a color based on CPU usage percentage for visual indication.
///
/// # Color Thresholds
/// * Red - Critical usage (≥80%)
/// * Yellow - Warning level (≥50%)
/// * Cyan - Moderate usage (≥20%)
/// * Green - Low usage (<20%)
///
/// # Arguments
/// * `percent` - CPU usage percentage (0-100)
#[must_use]
pub fn cpu_color(percent: f64) -> Color {
    if percent >= CPU_THRESHOLD_CRITICAL {
        Color::Red
    } else if percent >= CPU_THRESHOLD_WARNING {
        Color::Yellow
    } else if percent >= CPU_THRESHOLD_MODERATE {
        Color::Cyan
    } else {
        Color::Green
    }
}
