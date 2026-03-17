use crate::formatting::text_result;
use chrono::Utc;
use clotho_core::domain::types::{EntityId, EntityType};
use clotho_core::graph::GraphStore;
use clotho_store::content::ContentStore;
use clotho_store::data::entities::{EntityRow, EntityStore};
use clotho_store::data::jsonl::{Event, EventStore, EventType as EvtType};
use clotho_store::index::SearchIndex;
use clotho_store::workspace::Workspace;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "clotho_create_entity",
    description = "Create any entity type in the Clotho workspace (program, responsibility, objective, workstream, task, person, etc.).",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateEntityTool {
    /// Path to the directory containing .workspace/
    pub workspace_path: String,
    /// Entity type (program, responsibility, objective, workstream, task, meeting, transcript, note, reflection, artifact, decision, risk, blocker, question, insight, person)
    pub entity_type: String,
    /// Title of the entity
    pub title: String,
    /// Status (active, inactive). Defaults based on type.
    pub status: Option<String>,
    /// Task state (todo, doing, blocked, done). Only for Task.
    pub state: Option<String>,
    /// Email (only for Person)
    pub email: Option<String>,
    /// Parent entity ID. Creates a BELONGS_TO relation.
    pub parent_id: Option<String>,
    /// Inline markdown content
    pub content: Option<String>,
}

impl CreateEntityTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws = Workspace::open(Path::new(&self.workspace_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let et = parse_entity_type(&self.entity_type)?;
        let id = EntityId::new();
        let now = Utc::now();

        let (def_status, def_state, def_extraction) = defaults_for_type(et);
        let status = self.status.clone().or(def_status);
        let task_state = self.state.clone().or(def_state);

        // Metadata
        let mut meta = serde_json::Map::new();
        if let Some(ref email) = self.email {
            meta.insert("email".to_string(), serde_json::Value::String(email.clone()));
        }
        if let Some(ref pid) = self.parent_id {
            meta.insert("parent_id".to_string(), serde_json::Value::String(pid.clone()));
        }
        let metadata = if meta.is_empty() { None } else { Some(serde_json::to_string(&meta).unwrap_or_default()) };

        // Content
        let content_store = ContentStore::new(&ws.path);
        let content_text = self.content.clone().unwrap_or_default();
        let content_path = if !content_text.is_empty() || is_content_bearing(et) {
            let text = if content_text.is_empty() { format!("# {}\n", self.title) } else { content_text.clone() };
            Some(content_store.write_content(et, &id, &text)
                .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?)
        } else { None };

        // entities.db
        let store = EntityStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        store.insert(&EntityRow {
            id: id.to_string(), entity_type: format!("{}", et), title: self.title.clone(),
            created_at: now.to_rfc3339(), updated_at: now.to_rfc3339(),
            status, task_state, extraction_status: def_extraction,
            source_transcript_id: None, source_span_start: None, source_span_end: None,
            confidence: None, content_path: content_path.as_ref().map(|p| p.display().to_string()),
            metadata,
        }).map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        // Graph
        let graph = GraphStore::open(&ws.graph_path().join("relations.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        graph.register_node(&id, et, &self.title)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        if let Some(ref pid) = self.parent_id {
            if let Ok(parent_id) = uuid::Uuid::parse_str(pid).map(EntityId::from) {
                let _ = graph.add_edge(&id, &parent_id, clotho_core::domain::traits::RelationType::BelongsTo);
            }
        }

        // FTS5
        let idx = SearchIndex::open(&ws.index_path().join("search.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        let _ = idx.index_entity(&id.to_string(), &format!("{}", et), &self.title, &content_text);

        // Event
        let events = EventStore::new(&ws.data_path());
        let _ = events.log(&Event { timestamp: now, event_type: EvtType::Created, entity_id: id.to_string(), details: None });

        Ok(text_result(format!(
            "## Entity Created\n\n| Field | Value |\n|---|---|\n| ID | `{}` |\n| Type | {} |\n| Title | {} |",
            id, et, self.title
        )))
    }
}

fn parse_entity_type(s: &str) -> Result<EntityType, CallToolError> {
    match s.to_lowercase().as_str() {
        "program" => Ok(EntityType::Program), "responsibility" => Ok(EntityType::Responsibility),
        "objective" => Ok(EntityType::Objective), "workstream" => Ok(EntityType::Workstream),
        "task" => Ok(EntityType::Task), "meeting" => Ok(EntityType::Meeting),
        "transcript" => Ok(EntityType::Transcript), "note" => Ok(EntityType::Note),
        "reflection" => Ok(EntityType::Reflection), "artifact" => Ok(EntityType::Artifact),
        "decision" => Ok(EntityType::Decision), "risk" => Ok(EntityType::Risk),
        "blocker" => Ok(EntityType::Blocker), "question" => Ok(EntityType::Question),
        "insight" => Ok(EntityType::Insight), "person" => Ok(EntityType::Person),
        _ => Err(CallToolError::new(std::io::Error::other(format!("Unknown type: {}", s)))),
    }
}

fn defaults_for_type(et: EntityType) -> (Option<String>, Option<String>, Option<String>) {
    match et {
        EntityType::Program | EntityType::Responsibility | EntityType::Objective | EntityType::Workstream => (Some("active".into()), None, None),
        EntityType::Task => (None, Some("todo".into()), None),
        EntityType::Decision | EntityType::Risk | EntityType::Blocker | EntityType::Question | EntityType::Insight => (None, None, Some("draft".into())),
        _ => (None, None, None),
    }
}

fn is_content_bearing(et: EntityType) -> bool {
    !matches!(et, EntityType::Decision | EntityType::Risk | EntityType::Blocker | EntityType::Question | EntityType::Insight)
}
