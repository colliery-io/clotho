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
    name = "clotho_push_surface",
    description = "Push a surface (text blob) to the Clotho TUI. Surfaces appear as tabs in the TUI and can be edited by the user. Use for daily briefings, meeting notes, status updates, etc. If replace=true and a surface with the same title exists, its content is updated instead of creating a new one.",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PushSurfaceTool {
    /// Title of the surface (becomes the tab name in the TUI)
    pub title: String,
    /// Text content (markdown). User can edit this in the TUI.
    pub content: String,
    /// Optional surface type hint: briefing, meeting-notes, checklist, freeform
    pub surface_type: Option<String>,
    /// If true, replaces an existing active surface with the same title instead of creating a new one
    #[serde(default)]
    pub replace: bool,
}

impl PushSurfaceTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws_path = workspace_resolver::require_workspace()
            .map_err(|e| CallToolError::new(std::io::Error::other(e)))?;
        let ws = Workspace::open(Path::new(&ws_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let store = SurfaceStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let surface = store
            .push(
                &self.title,
                &self.content,
                self.surface_type.as_deref(),
                self.replace,
            )
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let action = if self.replace { "pushed/replaced" } else { "pushed" };

        Ok(text_result(format!(
            "## Surface {}\n\n| Field | Value |\n|---|---|\n| ID | `{}` |\n| Title | {} |\n| Type | {} |\n| Status | {} |",
            action,
            &surface.id[..8],
            surface.title,
            surface.surface_type.as_deref().unwrap_or("(none)"),
            surface.status,
        )))
    }
}
