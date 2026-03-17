use crate::formatting::text_result;
use chrono::Utc;
use clotho_store::data::entities::EntityStore;
use clotho_store::data::jsonl::{Event, EventStore, EventType};
use clotho_store::workspace::Workspace;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "clotho_update_entity",
    description = "Update an entity's fields (title, status, state).",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateEntityTool {
    /// Path to the directory containing .workspace/
    pub workspace_path: String,
    /// Entity ID (UUID)
    pub entity_id: String,
    /// New title
    pub title: Option<String>,
    /// New status (active, inactive)
    pub status: Option<String>,
    /// New task state (todo, doing, blocked, done)
    pub state: Option<String>,
}

impl UpdateEntityTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws = Workspace::open(Path::new(&self.workspace_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        let store = EntityStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let mut row = store.get(&self.entity_id)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?
            .ok_or_else(|| CallToolError::new(std::io::Error::other(format!("Entity not found: {}", self.entity_id))))?;

        if let Some(ref t) = self.title { row.title = t.clone(); }
        if let Some(ref s) = self.status { row.status = Some(s.clone()); }
        if let Some(ref s) = self.state { row.task_state = Some(s.clone()); }
        row.updated_at = Utc::now().to_rfc3339();

        store.update(&row)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let events = EventStore::new(&ws.data_path());
        let _ = events.log(&Event { timestamp: Utc::now(), event_type: EventType::Updated, entity_id: self.entity_id.clone(), details: None });

        Ok(text_result(format!("## Entity Updated\n\nUpdated `{}`", self.entity_id)))
    }
}
