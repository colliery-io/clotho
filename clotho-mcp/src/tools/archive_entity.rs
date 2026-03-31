use crate::formatting::text_result;
use crate::resolve;
use crate::workspace_resolver;
use chrono::Utc;
use clotho_store::data::entities::EntityStore;
use clotho_store::workspace::Workspace;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "clotho_archive_entity",
    description = "Archive an entity by setting its status to inactive. Archived entities are hidden from the TUI navigator by default but remain searchable and queryable. Use for completed work, resolved risks, answered questions, etc.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ArchiveEntityTool {
    /// Entity ID (full UUID or prefix)
    pub entity_id: String,
}

impl ArchiveEntityTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws_path = workspace_resolver::require_workspace()
            .map_err(|e| CallToolError::new(std::io::Error::other(e)))?;
        let ws = Workspace::open(Path::new(&ws_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let store = EntityStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let row = match resolve::resolve_for_write(&store, &self.entity_id) {
            Ok(row) => row,
            Err(result) => return Ok(result),
        };

        let mut updated = row.clone();
        updated.status = Some("inactive".to_string());
        updated.updated_at = Utc::now().to_rfc3339();

        store
            .update(&updated)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        Ok(text_result(format!(
            "## Entity Archived\n\n`{}` ({}) — **{}** is now inactive.\nIt will be hidden from the TUI navigator but remains searchable.",
            &row.id[..8], row.entity_type, row.title,
        )))
    }
}
