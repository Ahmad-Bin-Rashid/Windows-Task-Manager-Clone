//! System information module - Process, memory, and CPU metrics
//!
//! This module provides safe abstractions over Win32 system APIs
//! for gathering task manager-style information.

pub mod admin;
pub mod cpu;
pub mod details;
pub mod disk;
pub mod memory;
pub mod path;
pub mod priority;
pub mod processes;
pub mod suspend;
pub mod uptime;
