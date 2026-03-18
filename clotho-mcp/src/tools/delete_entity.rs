use crate::formatting::text_result;
use crate::workspace_resolver;
use chrono::Utc;
use clotho_core::domain::types::EntityId;
use clotho_core::graph::GraphStore;
use clotho_store::data::entities::EntityStore;
use clotho_store::data::jsonl::{Event, EventStore, EventType};
use clotho_store::index::SearchIndex;
use clotho_store::workspace::Workspace;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "clotho_delete_entity",
    description = "Delete an entity from all backends (entities.db, content, graph, search index).",
    idempotent_hint = false,
    destructive_hint = true,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeleteEntityTool {
    /// Entity ID (UUID)
    pub entity_id: String,
}

impl DeleteEntityTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws_path = workspace_resolver::require_workspace()
            .map_err(|e| CallToolError::new(std::io::Error::other(e)))?;
        let ws = Workspace::open(Path::new(&ws_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        let store = EntityStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let row = store.get(&self.entity_id)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?
            .ok_or_else(|| CallToolError::new(std::io::Error::other(format!("Entity not found: {}", self.entity_id))))?;

        let title = row.title.clone();

        // Delete content file
        if let Some(ref path) = row.content_path {
            let p = Path::new(path);
            if p.exists() { let _ = std::fs::remove_file(p); }
        }

        // Delete from entities.db
        store.delete(&self.entity_id)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        // Delete from graph
        if let Ok(graph) = GraphStore::open(&ws.graph_path().join("relations.db")) {
            if let Ok(id) = uuid::Uuid::parse_str(&self.entity_id).map(EntityId::from) {
                let _ = graph.remove_node(&id);
            }
        }

        // Delete from search index
        if let Ok(idx) = SearchIndex::open(&ws.index_path().join("search.db")) {
            let _ = idx.remove_entity(&self.entity_id);
        }

        // Event
        let events = EventStore::new(&ws.data_path());
        let _ = events.log(&Event { timestamp: Utc::now(), event_type: EventType::Deleted, entity_id: self.entity_id.clone(), details: None });

        Ok(text_result(format!("## Entity Deleted\n\nDeleted '{}' (`{}`)", title, self.entity_id)))
    }
}
