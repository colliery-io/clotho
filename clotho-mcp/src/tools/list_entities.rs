use crate::formatting::text_result;
use crate::workspace_resolver;
use clotho_store::data::entities::EntityStore;
use clotho_store::workspace::Workspace;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "clotho_list_entities",
    description = "List entities in the Clotho workspace with optional type, status, or state filters.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListEntitiesTool {
    /// Filter by entity type (e.g., Task, Program, Note)
    pub entity_type: Option<String>,
    /// Filter by status (active, inactive)
    pub status: Option<String>,
    /// Filter by task state (todo, doing, blocked, done)
    pub state: Option<String>,
}

impl ListEntitiesTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws_path = workspace_resolver::require_workspace()
            .map_err(|e| CallToolError::new(std::io::Error::other(e)))?;
        let ws = Workspace::open(Path::new(&ws_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let store = EntityStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let rows = if let Some(ref t) = self.entity_type {
            store.list_by_type(t)
        } else if let Some(ref s) = self.status {
            store.list_by_status(s)
        } else if let Some(ref s) = self.state {
            store.list_by_state(s)
        } else {
            store.list_all()
        }
        .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        if rows.is_empty() {
            return Ok(text_result("No entities found."));
        }

        let mut output = format!(
            "## Entities ({})\n\n| ID | Type | Title | Status |\n|---|---|---|---|\n",
            rows.len()
        );

        for row in &rows {
            let id_short = if row.id.len() > 8 { &row.id[..8] } else { &row.id };
            let status = row
                .task_state
                .as_deref()
                .or(row.status.as_deref())
                .unwrap_or("-");
            output.push_str(&format!(
                "| `{}...` | {} | {} | {} |\n",
                id_short, row.entity_type, row.title, status,
            ));
        }

        Ok(text_result(output))
    }
}
