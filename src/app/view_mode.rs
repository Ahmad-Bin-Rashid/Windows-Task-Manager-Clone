//! View mode enum for application state
//!
//! Defines mutually exclusive application modes, ensuring only one
//! mode can be active at a time. This prevents bugs from having
//! multiple conflicting modes active simultaneously.

/// The current view/input mode of the application.
///
/// Only one mode can be active at a time. The mode determines
/// how keyboard input is handled and what UI is displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    /// Normal process list view (default mode)
    #[default]
    ProcessList,
    
    /// Filter input mode - typing a process name filter
    FilterInput,
    
    /// Kill confirmation dialog - waiting for Y/N
    ConfirmKill,
    
    /// Detailed process information view
    DetailView,
    
    /// Help overlay showing keyboard shortcuts
    Help,
    
    /// CPU affinity editing dialog
    Affinity,
}

#[allow(dead_code)]
impl ViewMode {
    /// Returns true if the current mode is the default process list
    #[inline]
    pub fn is_process_list(&self) -> bool {
        matches!(self, ViewMode::ProcessList)
    }

    /// Returns true if in detail view mode
    #[inline]
    pub fn is_detail_view(&self) -> bool {
        matches!(self, ViewMode::DetailView)
    }

    /// Returns true if showing help overlay
    #[inline]
    pub fn is_help(&self) -> bool {
        matches!(self, ViewMode::Help)
    }

    /// Returns true if in affinity edit mode
    #[inline]
    pub fn is_affinity(&self) -> bool {
        matches!(self, ViewMode::Affinity)
    }

    /// Returns true if in filter input mode
    #[inline]
    pub fn is_filter_input(&self) -> bool {
        matches!(self, ViewMode::FilterInput)
    }

    /// Returns true if in kill confirmation mode
    #[inline]
    pub fn is_confirm_kill(&self) -> bool {
        matches!(self, ViewMode::ConfirmKill)
    }
}
