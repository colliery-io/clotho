use crate::formatting::text_result;
use chrono::Utc;
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
    name = "clotho_ingest",
    description = "Ingest a file into the Clotho workspace as content (note, meeting, transcript, or artifact).",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct IngestTool {
    /// Path to the directory containing .workspace/
    pub workspace_path: String,
    /// Path to the file to ingest
    pub file_path: String,
    /// Entity type: note, meeting, transcript, artifact (default: note)
    pub entity_type: Option<String>,
    /// Title for the entity (defaults to filename)
    pub title: Option<String>,
}

impl IngestTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws = Workspace::open(Path::new(&self.workspace_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let file = Path::new(&self.file_path);
        if !file.exists() {
            return Err(CallToolError::new(std::io::Error::other(format!(
                "File not found: {}", self.file_path
            ))));
        }

        let entity_type = parse_entity_type(self.entity_type.as_deref().unwrap_or("note"))?;
        let content = std::fs::read_to_string(file)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let title = self.title.clone().unwrap_or_else(|| {
            file.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("untitled")
                .to_string()
        });

        let id = EntityId::new();
        let now = Utc::now();

        let content_store = ContentStore::new(&ws.path);
        let content_path = content_store
            .write_content(entity_type, &id, &content)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let entity_store = EntityStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let row = EntityRow {
            id: id.to_string(),
            entity_type: format!("{}", entity_type),
            title: title.clone(),
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
        };
        entity_store
            .insert(&row)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let graph = GraphStore::open(&ws.graph_path().join("relations.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        graph
            .register_node(&id, entity_type, &title)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let index = SearchIndex::open(&ws.index_path().join("search.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        index
            .index_entity(&id.to_string(), &format!("{}", entity_type), &title, &content)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let events = EventStore::new(&ws.data_path());
        let _ = events.log(&Event {
            timestamp: now,
            event_type: EventType::Created,
            entity_id: id.to_string(),
            details: Some(serde_json::json!({"source_file": self.file_path})),
        });

        Ok(text_result(format!(
            "## Ingested\n\n| Field | Value |\n|---|---|\n| ID | `{}` |\n| Type | {} |\n| Title | {} |\n| Content | `{}` |",
            id, entity_type, title, content_path.display()
        )))
    }
}

fn parse_entity_type(s: &str) -> Result<EntityType, CallToolError> {
    match s.to_lowercase().as_str() {
        "note" => Ok(EntityType::Note),
        "meeting" => Ok(EntityType::Meeting),
        "transcript" => Ok(EntityType::Transcript),
        "artifact" => Ok(EntityType::Artifact),
        _ => Err(CallToolError::new(std::io::Error::other(format!(
            "Unknown type '{}'. Valid: note, meeting, transcript, artifact", s
        )))),
    }
}
