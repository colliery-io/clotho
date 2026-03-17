use crate::formatting::text_result;
use clotho_store::workspace::Workspace;
use clotho_sync::SyncEngine;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "clotho_sync",
    description = "Sync the Clotho workspace to git (stage, commit, push). Optionally prune history.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SyncTool {
    /// Path to the directory containing .workspace/
    pub workspace_path: String,
    /// Prune history after sync (keep last N commits, default 20)
    pub prune: Option<bool>,
    /// Number of commits to keep when pruning
    pub keep: Option<u32>,
}

impl SyncTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws = Workspace::open(Path::new(&self.workspace_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let engine = match SyncEngine::open(&ws.path) {
            Ok(e) => e,
            Err(_) => SyncEngine::init(&ws.path)
                .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?,
        };

        let result = engine.sync()
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let mut output = if result.committed {
            format!("## Synced\n\n{} files committed", result.files_changed)
        } else {
            "## No Changes\n\nNothing to sync.".to_string()
        };

        if result.pushed {
            output.push_str("\nPushed to remote.");
        }

        if self.prune.unwrap_or(false) {
            let keep = self.keep.unwrap_or(20) as usize;
            match engine.prune_history(keep) {
                Ok(pruned) if pruned > 0 => {
                    output.push_str(&format!("\nPruned {} commits (keeping {})", pruned, keep));
                }
                _ => {}
            }
        }

        Ok(text_result(output))
    }
}
