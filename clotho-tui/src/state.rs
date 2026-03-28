use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Persisted TUI display state — survives across restarts.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TuiState {
    /// Open tabs — each is either an entity ID or a surface ID.
    pub tabs: Vec<TabState>,
    /// Index of the active tab.
    pub active_tab: usize,
    /// Which navigator groups are expanded (by entity_type name).
    pub navigator_expanded: HashMap<String, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabState {
    pub kind: TabKind,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TabKind {
    Entity,
    Surface,
}

impl TuiState {
    /// Path to the state file within a workspace.
    pub fn state_path(workspace: &Path) -> PathBuf {
        workspace.join("tui-state.json")
    }

    /// Load state from disk, or return default if not found / corrupt.
    pub fn load(workspace: &Path) -> Self {
        let path = Self::state_path(workspace);
        match std::fs::read_to_string(&path) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save state to disk.
    pub fn save(&self, workspace: &Path) {
        let path = Self::state_path(workspace);
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, json);
        }
    }
}
