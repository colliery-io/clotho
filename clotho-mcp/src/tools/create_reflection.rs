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
    name = "clotho_create_reflection",
    description = "Create a new reflection entry in the Clotho workspace with a period-based template.",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateReflectionTool {
    /// Path to the directory containing .clotho/
    pub workspace_path: String,
    /// Period type: daily, weekly, monthly, quarterly, adhoc
    pub period: String,
    /// Optional title (defaults to period-based name)
    pub title: Option<String>,
    /// Optional program ID to scope the reflection to
    pub program_id: Option<String>,
}

impl CreateReflectionTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws = Workspace::open(Path::new(&self.workspace_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let now = Utc::now();
        let id = EntityId::new();

        let title = self.title.clone().unwrap_or_else(|| {
            format!("{} {} reflection", now.format("%Y-%m-%d"), self.period)
        });

        let template = format!(
            "# {}\n\n## Period\n\nType: {}\nDate: {}\n\n## Reflections\n\n\n\n## Key Takeaways\n\n\n\n## Action Items\n\n\n",
            title, self.period, now.format("%Y-%m-%d"),
        );

        let content_store = ContentStore::new(&ws.project_root());
        let content_path = content_store
            .write_content(EntityType::Reflection, &id, &template)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let mut metadata = serde_json::Map::new();
        metadata.insert("period_type".to_string(), serde_json::Value::String(self.period.clone()));
        if let Some(ref prog) = self.program_id {
            metadata.insert("program_id".to_string(), serde_json::Value::String(prog.clone()));
        }

        let entity_store = EntityStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        entity_store
            .insert(&EntityRow {
                id: id.to_string(),
                entity_type: "Reflection".to_string(),
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
                metadata: Some(serde_json::to_string(&metadata).unwrap_or_default()),
            })
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let graph = GraphStore::open(&ws.graph_path().join("relations.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        graph
            .register_node(&id, EntityType::Reflection, &title)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let index = SearchIndex::open(&ws.index_path().join("search.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        index
            .index_entity(&id.to_string(), "Reflection", &title, &template)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let events = EventStore::new(&ws.data_path());
        let _ = events.log(&Event {
            timestamp: now,
            event_type: EventType::Created,
            entity_id: id.to_string(),
            details: Some(serde_json::json!({"period": self.period})),
        });

        Ok(text_result(format!(
            "## Reflection Created\n\n| Field | Value |\n|---|---|\n| ID | `{}` |\n| Title | {} |\n| Period | {} |\n| File | `{}` |",
            id, title, self.period, content_path.display()
        )))
    }
}
