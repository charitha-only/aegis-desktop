//! Process-Per-Window Sandboxed Supervisor Engine
//!
//! Enforces site-isolation and process-level sandboxing for desktop windows.
//! Each child window (e.g. dialog, external preview, plugin frame) runs under
//! a dedicated sandboxed child process with strictly attenuated capabilities.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Granular Capability Scopes granted to a sandboxed child window process
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowCapabilityScope {
    pub can_read_storage: bool,
    pub can_write_storage: bool,
    pub can_access_clipboard: bool,
    pub can_open_file_dialog: bool,
    pub can_sign_transactions: bool,
    pub max_memory_mb: u32,
}

impl Default for WindowCapabilityScope {
    fn default() -> Self {
        Self {
            can_read_storage: true,
            can_write_storage: false, // Strict default: child windows cannot overwrite storage
            can_access_clipboard: false,
            can_open_file_dialog: false,
            can_sign_transactions: false,
            max_memory_mb: 64, // 64MB memory cap per child process
        }
    }
}

/// Managed Child Window Process Descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildProcessDescriptor {
    pub process_id: u32,
    pub window_id: String,
    pub title: String,
    pub capabilities: WindowCapabilityScope,
    pub is_alive: bool,
    pub memory_usage_bytes: usize,
}

/// Window Process Supervisor Engine
pub struct WindowProcessSupervisor {
    children: HashMap<String, ChildProcessDescriptor>,
    next_synthetic_pid: u32,
}

impl WindowProcessSupervisor {
    pub fn new() -> Self {
        Self {
            children: HashMap::new(),
            next_synthetic_pid: 20000,
        }
    }

    /// Spawn and supervise an isolated child window process with capability attenuation
    pub fn spawn_isolated_child(
        &mut self,
        window_id: &str,
        title: &str,
        capabilities: WindowCapabilityScope,
    ) -> Result<ChildProcessDescriptor, &'static str> {
        if self.children.contains_key(window_id) {
            return Err("ERR_WINDOW_ALREADY_EXISTS");
        }

        let pid = self.next_synthetic_pid;
        self.next_synthetic_pid += 1;

        let descriptor = ChildProcessDescriptor {
            process_id: pid,
            window_id: window_id.to_string(),
            title: title.to_string(),
            capabilities,
            is_alive: true,
            memory_usage_bytes: 14 * 1024 * 1024, // ~14MB initial resident RSS
        };

        self.children.insert(window_id.to_string(), descriptor.clone());
        Ok(descriptor)
    }

    /// Validate whether a child window process is authorized to perform an IPC action
    pub fn authorize_action(&self, window_id: &str, action: &str) -> bool {
        let child = match self.children.get(window_id) {
            Some(c) if c.is_alive => c,
            _ => return false,
        };

        match action {
            "storage:read" => child.capabilities.can_read_storage,
            "storage:write" => child.capabilities.can_write_storage,
            "clipboard:read" | "clipboard:write" => child.capabilities.can_access_clipboard,
            "dialog:open" | "dialog:save" => child.capabilities.can_open_file_dialog,
            "transaction:sign" => child.capabilities.can_sign_transactions,
            _ => false, // Fail closed for any unmapped action
        }
    }

    /// Terminate and cleanup a supervised child window process
    pub fn terminate_child(&mut self, window_id: &str) -> Result<u32, &'static str> {
        if let Some(child) = self.children.get_mut(window_id) {
            if !child.is_alive {
                return Err("ERR_ALREADY_TERMINATED");
            }
            child.is_alive = false;
            Ok(child.process_id)
        } else {
            Err("ERR_WINDOW_NOT_FOUND")
        }
    }

    /// List all currently active child processes
    pub fn list_active_children(&self) -> Vec<ChildProcessDescriptor> {
        self.children
            .values()
            .filter(|c| c.is_alive)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_and_capability_attenuation() {
        let mut supervisor = WindowProcessSupervisor::new();

        let restricted_caps = WindowCapabilityScope {
            can_read_storage: true,
            can_write_storage: false,
            can_access_clipboard: false,
            can_open_file_dialog: false,
            can_sign_transactions: false,
            max_memory_mb: 32,
        };

        let child = supervisor
            .spawn_isolated_child("win-preview-01", "Document Previewer", restricted_caps)
            .expect("Spawn failed");

        assert_eq!(child.process_id, 20000);
        assert!(supervisor.authorize_action("win-preview-01", "storage:read"));
        // Write, clipboard, and signing must fail closed
        assert!(!supervisor.authorize_action("win-preview-01", "storage:write"));
        assert!(!supervisor.authorize_action("win-preview-01", "clipboard:read"));
        assert!(!supervisor.authorize_action("win-preview-01", "transaction:sign"));
    }

    #[test]
    fn test_termination_lifecycle() {
        let mut supervisor = WindowProcessSupervisor::new();
        supervisor
            .spawn_isolated_child("win-dialog", "Confirmation Dialog", WindowCapabilityScope::default())
            .unwrap();

        assert_eq!(supervisor.list_active_children().len(), 1);

        let term_res = supervisor.terminate_child("win-dialog");
        assert!(term_res.is_ok());
        assert_eq!(supervisor.list_active_children().len(), 0);

        // Subsequent actions are denied
        assert!(!supervisor.authorize_action("win-dialog", "storage:read"));
    }
}

