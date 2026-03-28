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
    name = "clotho_list_surfaces",
    description = "List surfaces, optionally filtered by status (active/closed) and type. Supports keyword search across title and content.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListSurfacesTool {
    /// Filter by status: "active" or "closed". Defaults to "active" if not specified.
    pub status: Option<String>,
    /// Filter by surface type: briefing, meeting-notes, checklist, freeform
    pub surface_type: Option<String>,
    /// Search keyword — filters surfaces whose title or content contains this text
    pub search: Option<String>,
}

impl ListSurfacesTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws_path = workspace_resolver::require_workspace()
            .map_err(|e| CallToolError::new(std::io::Error::other(e)))?;
        let ws = Workspace::open(Path::new(&ws_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let store = SurfaceStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let surfaces = if let Some(ref query) = self.search {
            store
                .search(query)
                .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?
        } else {
            let status = self.status.as_deref().or(Some("active"));
            store
                .list(status, self.surface_type.as_deref())
                .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?
        };

        if surfaces.is_empty() {
            return Ok(text_result("No surfaces found.".to_string()));
        }

        let mut output = format!("## Surfaces ({})\n\n| ID | Title | Type | Status | Updated |\n|---|---|---|---|---|\n", surfaces.len());
        for s in &surfaces {
            output.push_str(&format!(
                "| `{}` | {} | {} | {} | {} |\n",
                &s.id[..8],
                s.title,
                s.surface_type.as_deref().unwrap_or("-"),
                s.status,
                s.updated_at,
            ));
        }

        Ok(text_result(output))
    }
}
