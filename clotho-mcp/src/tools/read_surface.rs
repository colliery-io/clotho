use crate::formatting::text_result;
use crate::workspace_resolver;
use clotho_store::data::surfaces::SurfaceStore;
use clotho_store::workspace::Workspace;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "clotho_read_surface",
    description = "Read a surface by ID or title. Returns the full content including any user edits made in the TUI. Use this to read back surfaces you previously pushed.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReadSurfaceTool {
    /// Surface ID (full UUID) or exact title to look up
    pub id_or_title: String,
}

impl ReadSurfaceTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws_path = workspace_resolver::require_workspace()
            .map_err(|e| CallToolError::new(std::io::Error::other(e)))?;
        let ws = Workspace::open(Path::new(&ws_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let store = SurfaceStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        // Try by ID first, then by title
        let surface = store
            .get(&self.id_or_title)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?
            .or_else(|| {
                store
                    .find_active_by_title(&self.id_or_title)
                    .ok()
                    .flatten()
            });

        match surface {
            Some(s) => Ok(text_result(format!(
                "## Surface: {}\n\n| Field | Value |\n|---|---|\n| ID | `{}` |\n| Type | {} |\n| Status | {} |\n| Updated | {} |\n\n---\n\n{}",
                s.title,
                s.id,
                s.surface_type.as_deref().unwrap_or("(none)"),
                s.status,
                s.updated_at,
                s.content,
            ))),
            None => Ok(text_result(format!(
                "Surface not found: `{}`",
                self.id_or_title
            ))),
        }
    }
}
