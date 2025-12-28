//! Application state management

use common::ipc::AppState as IpcAppState;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Current application state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppState {
    /// Currently selected profile ID
    pub selected_profile: Option<String>,
    /// Set of enabled bundle IDs
    pub enabled_bundles: HashSet<String>,
    /// Custom hostname (if changed from current)
    pub hostname: Option<String>,
    /// Whether there are unsaved changes
    pub has_changes: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Select a profile (replaces any existing selection)
    pub fn select_profile(&mut self, profile_id: impl Into<String>) {
        self.selected_profile = Some(profile_id.into());
        self.has_changes = true;
    }

    /// Clear profile selection
    pub fn clear_profile(&mut self) {
        self.selected_profile = None;
        self.has_changes = true;
    }

    /// Toggle a bundle on/off
    pub fn toggle_bundle(&mut self, bundle_id: impl Into<String>) {
        let id = bundle_id.into();
        if self.enabled_bundles.contains(&id) {
            self.enabled_bundles.remove(&id);
        } else {
            self.enabled_bundles.insert(id);
        }
        self.has_changes = true;
    }

    /// Enable a bundle
    pub fn enable_bundle(&mut self, bundle_id: impl Into<String>) {
        self.enabled_bundles.insert(bundle_id.into());
        self.has_changes = true;
    }

    /// Disable a bundle
    pub fn disable_bundle(&mut self, bundle_id: &str) {
        self.enabled_bundles.remove(bundle_id);
        self.has_changes = true;
    }

    /// Check if a bundle is enabled
    pub fn is_bundle_enabled(&self, bundle_id: &str) -> bool {
        self.enabled_bundles.contains(bundle_id)
    }

    /// Set custom hostname
    pub fn set_hostname(&mut self, hostname: impl Into<String>) {
        let h = hostname.into();
        self.hostname = if h.is_empty() { None } else { Some(h) };
        self.has_changes = true;
    }

    /// Mark changes as applied
    pub fn mark_applied(&mut self) {
        self.has_changes = false;
    }

    /// Convert to IPC state format
    pub fn to_ipc_state(&self) -> IpcAppState {
        IpcAppState {
            selected_profile: self.selected_profile.clone(),
            enabled_bundles: self.enabled_bundles.iter().cloned().collect(),
            hostname: self.hostname.clone(),
            last_applied: None,
        }
    }

    /// Create from IPC state format
    pub fn from_ipc_state(ipc: IpcAppState) -> Self {
        Self {
            selected_profile: ipc.selected_profile,
            enabled_bundles: ipc.enabled_bundles.into_iter().collect(),
            hostname: ipc.hostname,
            has_changes: false,
        }
    }

    /// Get a summary of current selections
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref profile) = self.selected_profile {
            parts.push(format!("Profile: {}", profile));
        }

        if !self.enabled_bundles.is_empty() {
            let bundles: Vec<_> = self.enabled_bundles.iter().cloned().collect();
            parts.push(format!("Bundles: {}", bundles.join(", ")));
        }

        if let Some(ref hostname) = self.hostname {
            parts.push(format!("Hostname: {}", hostname));
        }

        if parts.is_empty() {
            "No selections made".to_string()
        } else {
            parts.join("\n")
        }
    }
}
