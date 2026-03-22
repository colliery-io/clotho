use crate::formatting::text_result;
use crate::resolve;
use crate::workspace_resolver;
use chrono::Utc;
use clotho_core::domain::types::{EntityId, EntityType};
use clotho_store::content::ContentStore;
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
    name = "clotho_update_entity",
    description = "Update an entity's fields (title, status, state, content). Accepts full UUID or prefix. Content updates write to the entity's markdown file.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateEntityTool {
    /// Entity ID (full UUID or prefix)
    pub entity_id: String,
    /// New title
    pub title: Option<String>,
    /// New status (active, inactive)
    pub status: Option<String>,
    /// New task state (todo, doing, blocked, done)
    pub state: Option<String>,
    /// New markdown content (replaces the entity's content file)
    pub content: Option<String>,
    /// Email address (for Person entities)
    pub email: Option<String>,
    /// URL (for Reference and Artifact entities)
    pub url: Option<String>,
}

impl UpdateEntityTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws_path = workspace_resolver::require_workspace()
            .map_err(|e| CallToolError::new(std::io::Error::other(e)))?;
        let ws = Workspace::open(Path::new(&ws_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        let store = EntityStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let mut row = match resolve::resolve_for_write(&store, &self.entity_id) {
            Ok(row) => row,
            Err(result) => return Ok(result),
        };

        if let Some(ref t) = self.title {
            row.title = t.clone();
        }
        if let Some(ref s) = self.status {
            row.status = Some(s.clone());
        }
        if let Some(ref s) = self.state {
            row.task_state = Some(s.clone());
        }

        // Handle metadata updates (email, url — stored in metadata JSON)
        if self.email.is_some() || self.url.is_some() {
            let mut meta: serde_json::Map<String, serde_json::Value> = row
                .metadata
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default();
            if let Some(ref email) = self.email {
                meta.insert(
                    "email".to_string(),
                    serde_json::Value::String(email.clone()),
                );
            }
            if let Some(ref url) = self.url {
                meta.insert("url".to_string(), serde_json::Value::String(url.clone()));
            }
            row.metadata = Some(serde_json::to_string(&meta).unwrap_or_default());
        }

        // Handle content update
        if let Some(ref new_content) = self.content {
            let et = parse_entity_type_str(&row.entity_type)
                .map_err(|e| CallToolError::new(std::io::Error::other(e)))?;
            let eid = uuid::Uuid::parse_str(&row.id)
                .map(EntityId::from)
                .map_err(|e| {
                    CallToolError::new(std::io::Error::other(format!("invalid ID: {}", e)))
                })?;

            let content_store = ContentStore::new(&ws.project_root());
            let content_path = content_store
                .write_content(et, &eid, new_content)
                .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

            row.content_path = Some(content_path.display().to_string());

            // Update FTS5 index
            if let Ok(index) = SearchIndex::open(&ws.index_path().join("search.db")) {
                let _ = index.index_entity(&row.id, &row.entity_type, &row.title, new_content);
            }
        }

        row.updated_at = Utc::now().to_rfc3339();

        store
            .update(&row)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let resolved_id = row.id.clone();
        let events = EventStore::new(&ws.data_path());
        let _ = events.log(&Event {
            timestamp: Utc::now(),
            event_type: EventType::Updated,
            entity_id: resolved_id.clone(),
            details: None,
        });

        let mut msg = format!("## Entity Updated\n\nUpdated `{}`", resolved_id);
        if self.content.is_some() {
            msg.push_str(" (content updated)");
        }

        Ok(text_result(msg))
    }
}

fn parse_entity_type_str(s: &str) -> Result<EntityType, String> {
    match s {
        "Program" => Ok(EntityType::Program),
        "Responsibility" => Ok(EntityType::Responsibility),
        "Objective" => Ok(EntityType::Objective),
        "Workstream" => Ok(EntityType::Workstream),
        "Task" => Ok(EntityType::Task),
        "Meeting" => Ok(EntityType::Meeting),
        "Transcript" => Ok(EntityType::Transcript),
        "Note" => Ok(EntityType::Note),
        "Reflection" => Ok(EntityType::Reflection),
        "Artifact" => Ok(EntityType::Artifact),
        "Reference" => Ok(EntityType::Reference),
        "Decision" => Ok(EntityType::Decision),
        "Risk" => Ok(EntityType::Risk),
        "Blocker" => Ok(EntityType::Blocker),
        "Question" => Ok(EntityType::Question),
        "Insight" => Ok(EntityType::Insight),
        "Person" => Ok(EntityType::Person),
        _ => Err(format!("unknown entity type: {}", s)),
    }
}
