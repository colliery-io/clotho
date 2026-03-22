use crate::formatting::text_result;
use crate::resolve;
use crate::workspace_resolver;
use chrono::Utc;
use clotho_core::domain::traits::RelationType;
use clotho_core::domain::types::{EntityId, EntityType};
use clotho_core::graph::GraphStore;
use clotho_store::content::ContentStore;
use clotho_store::data::entities::{EntityRow, EntityStore};
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
    name = "clotho_create_note",
    description = "Create a new note entity in the Clotho workspace. Optionally link to a parent entity via belongs_to relation.",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateNoteTool {
    /// Title of the note
    pub title: String,
    /// Markdown content of the note
    pub content: String,
    /// Optional parent entity ID (full UUID or prefix) — auto-creates a belongs_to relation
    pub parent_id: Option<String>,
}

impl CreateNoteTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws_path = workspace_resolver::require_workspace()
            .map_err(|e| CallToolError::new(std::io::Error::other(e)))?;
        let ws = Workspace::open(Path::new(&ws_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let id = EntityId::new();
        let now = Utc::now();

        let content_store = ContentStore::new(&ws.project_root());
        let content_path = content_store
            .write_content(EntityType::Note, &id, &self.content)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let entity_store = EntityStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        entity_store
            .insert(&EntityRow {
                id: id.to_string(),
                entity_type: "Note".to_string(),
                title: self.title.clone(),
                created_at: now.to_rfc3339(),
                updated_at: now.to_rfc3339(),
                status: Some("active".to_string()),
                task_state: None,
                extraction_status: None,
                source_transcript_id: None,
                source_span_start: None,
                source_span_end: None,
                confidence: None,
                content_path: Some(content_path.display().to_string()),
                metadata: None,
            })
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let graph = GraphStore::open(&ws.graph_path().join("relations.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        graph
            .register_node(&id, EntityType::Note, &self.title)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let index = SearchIndex::open(&ws.index_path().join("search.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        index
            .index_entity(&id.to_string(), "Note", &self.title, &self.content)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let events = EventStore::new(&ws.data_path());
        let _ = events.log(&Event {
            timestamp: now,
            event_type: EventType::Created,
            entity_id: id.to_string(),
            details: None,
        });

        // Auto-create belongs_to relation if parent_id provided
        let mut parent_info = String::new();
        if let Some(ref parent_input) = self.parent_id {
            let parent_row = match resolve::resolve_for_write(&entity_store, parent_input) {
                Ok(row) => row,
                Err(result) => return Ok(result),
            };
            let parent_eid = uuid::Uuid::parse_str(&parent_row.id)
                .map(EntityId::from)
                .map_err(|e| {
                    CallToolError::new(std::io::Error::other(format!("invalid parent ID: {}", e)))
                })?;
            graph
                .add_edge(&id, &parent_eid, RelationType::BelongsTo)
                .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
            parent_info = format!(
                "\n| Parent | `{}` ({}) |",
                &parent_row.id[..8],
                parent_row.title
            );
        }

        Ok(text_result(format!(
            "## Note Created\n\n| Field | Value |\n|---|---|\n| ID | `{}` |\n| Title | {} |\n| Content | `{}` |{}",
            id, self.title, content_path.display(), parent_info
        )))
    }
}
