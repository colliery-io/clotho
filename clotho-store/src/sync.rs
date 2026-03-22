use chrono::Utc;

use clotho_core::domain::traits::*;
use clotho_core::domain::types::*;
use clotho_core::graph::GraphStore;

use crate::content::ContentStore;
use crate::data::entities::{EntityRow, EntityStore};
use crate::data::extractions::ExtractionStore;
use crate::data::jsonl::{Event, EventStore, EventType};
use crate::error::StoreError;
use crate::index::SearchIndex;

/// Coordinated write layer across all storage backends.
///
/// All mutations should go through StoreSync to ensure consistency
/// across content files, entities.db, graph, search index, and event log.
pub struct StoreSync<'a> {
    pub content: &'a ContentStore,
    pub entities: &'a EntityStore,
    pub extractions: &'a ExtractionStore,
    pub events: &'a EventStore,
    pub search: &'a SearchIndex,
    pub graph: &'a GraphStore,
}

impl<'a> StoreSync<'a> {
    /// Save an entity across all backends.
    ///
    /// 1. Write content to markdown file (if content provided)
    /// 2. Insert/update in entities.db
    /// 3. Register node in graph
    /// 4. Index in FTS5
    /// 5. Log event
    pub fn save_entity(
        &self,
        row: &EntityRow,
        content: Option<&str>,
        entity_type: EntityType,
    ) -> Result<(), StoreError> {
        let id_str = &row.id;
        let id = parse_entity_id(id_str)?;

        // Check existence BEFORE insert/update to determine event type
        let is_update = self.entities.get(id_str)?.is_some();

        // 1. Write content file if provided
        if let Some(text) = content {
            self.content.write_content(entity_type, &id, text)?;
        }

        // 2. Insert or update in entities.db
        if is_update {
            self.entities.update(row)?;
        } else {
            self.entities.insert(row)?;
        }

        // 3. Register node in graph
        self.graph
            .register_node(&id, entity_type, &row.title)
            .map_err(|e| StoreError::SearchError(e.to_string()))?;

        // 4. Index in FTS5
        self.search
            .index_entity(id_str, &row.entity_type, &row.title, content.unwrap_or(""))?;

        // 5. Log event
        let event_type = if is_update {
            EventType::Updated
        } else {
            EventType::Created
        };
        self.log_event(event_type, id_str, None)?;

        Ok(())
    }

    /// Delete an entity from all backends.
    pub fn delete_entity(&self, id_str: &str, entity_type: EntityType) -> Result<(), StoreError> {
        let id = parse_entity_id(id_str)?;

        // Content
        self.content.delete_content(entity_type, &id)?;

        // entities.db
        self.entities.delete(id_str)?;

        // Graph
        self.graph
            .remove_node(&id)
            .map_err(|e| StoreError::SearchError(e.to_string()))?;

        // Search index
        self.search.remove_entity(id_str)?;

        // Event log
        self.log_event(EventType::Deleted, id_str, None)?;

        Ok(())
    }

    /// Promote a draft extraction: move from extractions.db to entities.db,
    /// register graph node, index in FTS5, and log event.
    pub fn promote_extraction(&self, id_str: &str) -> Result<EntityRow, StoreError> {
        // Promote in extractions.db (validates it's a draft)
        let extraction = self.extractions.promote(id_str)?;

        // Convert to entity row
        let entity_row = EntityRow {
            id: extraction.id.clone(),
            entity_type: extraction.entity_type.clone(),
            title: extraction.title.clone(),
            created_at: extraction.created_at.clone(),
            updated_at: Utc::now().to_rfc3339(),
            status: None,
            task_state: None,
            extraction_status: Some("promoted".to_string()),
            source_transcript_id: extraction.source_transcript_id,
            source_span_start: extraction.source_span_start,
            source_span_end: extraction.source_span_end,
            confidence: extraction.confidence,
            content_path: None,
            metadata: extraction.metadata,
        };

        // Insert into entities.db
        self.entities.insert(&entity_row)?;

        // Register in graph
        if let Ok(et) = parse_entity_type_str(&entity_row.entity_type) {
            let id = parse_entity_id(&entity_row.id)?;
            self.graph
                .register_node(&id, et, &entity_row.title)
                .map_err(|e| StoreError::SearchError(e.to_string()))?;

            // Index in FTS5
            self.search.index_entity(
                &entity_row.id,
                &entity_row.entity_type,
                &entity_row.title,
                "",
            )?;
        }

        // Log event
        self.log_event(EventType::Promoted, id_str, None)?;

        Ok(entity_row)
    }

    /// Discard a draft extraction.
    pub fn discard_extraction(&self, id_str: &str) -> Result<(), StoreError> {
        self.extractions.discard(id_str)?;
        self.log_event(EventType::Discarded, id_str, None)?;
        Ok(())
    }

    /// Materialize temporal edges for an entity in the graph.
    ///
    /// Inspects the entity's temporal fields and creates/removes
    /// corresponding HAS_CADENCE, HAS_DEADLINE, HAS_SCHEDULE graph edges.
    pub fn materialize_temporal_edges(
        &self,
        id_str: &str,
        _entity_type: EntityType,
        has_cadence: bool,
        has_deadline: bool,
        has_schedule: bool,
    ) -> Result<(), StoreError> {
        let id = parse_entity_id(id_str)?;

        // For temporal edges, we create self-referencing edges with properties.
        // In a richer implementation, these would point to temporal anchor nodes.
        // For now, we use the entity itself as both source and target with edge properties.

        if has_cadence {
            self.graph
                .add_edge(&id, &id, RelationType::HasCadence)
                .map_err(|e| StoreError::SearchError(e.to_string()))?;
        }

        if has_deadline {
            self.graph
                .add_edge(&id, &id, RelationType::HasDeadline)
                .map_err(|e| StoreError::SearchError(e.to_string()))?;
        }

        if has_schedule {
            self.graph
                .add_edge(&id, &id, RelationType::HasSchedule)
                .map_err(|e| StoreError::SearchError(e.to_string()))?;
        }

        Ok(())
    }

    fn log_event(
        &self,
        event_type: EventType,
        entity_id: &str,
        details: Option<serde_json::Value>,
    ) -> Result<(), StoreError> {
        self.events.log(&Event {
            timestamp: Utc::now(),
            event_type,
            entity_id: entity_id.to_string(),
            details,
        })
    }
}

fn parse_entity_id(s: &str) -> Result<EntityId, StoreError> {
    uuid::Uuid::parse_str(s)
        .map(EntityId::from)
        .map_err(|e| StoreError::EntityNotFound(format!("invalid entity id '{}': {}", s, e)))
}

fn parse_entity_type_str(s: &str) -> Result<EntityType, StoreError> {
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
        _ => Err(StoreError::EntityNotFound(format!(
            "unknown entity type: {}",
            s
        ))),
    }
}
