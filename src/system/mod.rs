//! System information module - Process, memory, and CPU metrics
//!
//! This module provides safe abstractions over Win32 system APIs
//! for gathering task manager-style information.

pub mod processes;
pub mod memory;
pub mod cpu;
pub mod disk;
pub mod priority;
