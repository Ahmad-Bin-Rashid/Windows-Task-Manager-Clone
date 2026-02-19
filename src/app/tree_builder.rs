//! Process tree building logic

use std::collections::{HashMap, HashSet};

use super::state::App;
use super::ProcessEntry;

impl App {
    /// Builds a hierarchical process tree from flat list
    pub fn build_process_tree(&mut self) {
        // If not in tree mode, just apply filter normally
        if !self.tree_view_mode {
            self.apply_filter();
            return;
        }

        // Create a map of PID -> ProcessEntry
        let pid_map: HashMap<u32, &ProcessEntry> = self
            .processes
            .iter()
            .map(|p| (p.info.pid, p))
            .collect();

        // Find all PIDs in our list
        let all_pids: HashSet<u32> = self.processes.iter().map(|p| p.info.pid).collect();

        // Find root processes (parent not in our list or parent is 0)
        let mut roots: Vec<&ProcessEntry> = self
            .processes
            .iter()
            .filter(|p| p.info.parent_pid == 0 || !all_pids.contains(&p.info.parent_pid))
            .collect();

        // Sort roots by name
        roots.sort_by(|a, b| a.info.name.to_lowercase().cmp(&b.info.name.to_lowercase()));

        // Recursively build tree
        let mut result = Vec::new();
        for root in roots {
            self.add_process_with_children(&mut result, root, 0, &pid_map, &all_pids);
        }

        self.processes = result;
        self.apply_filter();
    }

    /// Recursively adds a process and its children to the result
    fn add_process_with_children(
        &self,
        result: &mut Vec<ProcessEntry>,
        process: &ProcessEntry,
        depth: usize,
        pid_map: &HashMap<u32, &ProcessEntry>,
        _all_pids: &HashSet<u32>,
    ) {
        // Add this process with its depth
        let mut entry = process.clone();
        entry.tree_depth = depth;
        result.push(entry);

        // Find and add children (limit depth to prevent infinite loops)
        if depth < 10 {
            let mut children: Vec<&ProcessEntry> = pid_map
                .values()
                .filter(|p| p.info.parent_pid == process.info.pid && p.info.pid != process.info.pid)
                .cloned()
                .collect();

            // Sort children by name
            children.sort_by(|a, b| a.info.name.to_lowercase().cmp(&b.info.name.to_lowercase()));

            // Recursively add children
            for child in children {
                self.add_process_with_children(result, child, depth + 1, pid_map, _all_pids);
            }
        }
    }

    /// Toggles tree view mode on/off
    pub fn toggle_tree_view(&mut self) {
        self.tree_view_mode = !self.tree_view_mode;
        self.selected_index = 0;
        self.scroll_offset = 0;

        if self.tree_view_mode {
            self.build_process_tree();
        } else {
            // Reset tree depth
            for proc in &mut self.processes {
                proc.tree_depth = 0;
            }
            self.apply_filter();
        }
    }
}
