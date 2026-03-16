use chrono::{DateTime, Utc};
use std::path::PathBuf;

use crate::domain::types::*;
use crate::error::{PromotionError, TransitionError};

// ---------------------------------------------------------------------------
// Graph placeholder types — will be fleshed out in clotho-graph work
// ---------------------------------------------------------------------------

/// Placeholder for the graph database handle.
/// Will be replaced with the real graphqlite wrapper.
pub struct Graph;

/// A typed relation between two entities.
#[derive(Debug, Clone, PartialEq)]
pub struct Relation {
    pub source_id: EntityId,
    pub target_id: EntityId,
    pub relation_type: RelationType,
}

/// Typed relation kinds per CLO-S-0005.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RelationType {
    BelongsTo,
    RelatesTo,
    Delivers,
    SpawnedFrom,
    ExtractedFrom,
    HasDecision,
    HasRisk,
    BlockedBy,
    Mentions,
    HasCadence,
    HasDeadline,
    HasSchedule,
}

// ---------------------------------------------------------------------------
// Core entity traits
// ---------------------------------------------------------------------------

/// Core identity for all entities.
pub trait Entity {
    fn id(&self) -> &EntityId;
    fn entity_type(&self) -> EntityType;
    fn title(&self) -> &str;
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;
}

/// Entities with active/inactive lifecycle.
pub trait Activatable: Entity {
    fn status(&self) -> Status;
    fn set_status(&mut self, status: Status);
}

/// Entities with task-like workflow state machine.
///
/// Valid transitions:
/// - Todo → Doing
/// - Doing → Blocked, Done
/// - Blocked → Doing
/// - Done is terminal
pub trait Taskable: Entity {
    fn state(&self) -> TaskState;
    fn transition(&mut self, to: TaskState) -> Result<(), TransitionError>;
    fn valid_transitions(&self) -> Vec<TaskState>;
}

/// Entities produced by AI extraction with a draft lifecycle.
///
/// Lifecycle: Draft → Promoted | Discarded (both terminal).
pub trait Extractable: Entity {
    fn extraction_status(&self) -> ExtractionStatus;
    fn source_span(&self) -> Option<&SourceSpan>;
    fn confidence(&self) -> f32;
    fn promote(&mut self) -> Result<(), PromotionError>;
    fn discard(&mut self);
}

/// Entities that participate in the relation graph.
pub trait Relatable: Entity {
    fn relations(&self, graph: &Graph) -> Vec<Relation>;
    fn graph_label(&self) -> &'static str;
}

/// Entities with freeform tags.
pub trait Taggable: Entity {
    fn tags(&self) -> &[Tag];
    fn add_tag(&mut self, tag: Tag);
    fn remove_tag(&mut self, tag: &str);
}

/// Entities with markdown content stored as files.
pub trait ContentBearing: Entity {
    fn content(&self) -> &str;
    fn set_content(&mut self, content: String);
    fn content_path(&self) -> PathBuf;
}

// ---------------------------------------------------------------------------
// Temporal traits
//
// Temporal scheduling concerns attach to entities via these traits.
// Data is stored on entity structs AND materialized as graph edges
// (HAS_CADENCE, HAS_DEADLINE, HAS_SCHEDULE) for cross-entity temporal
// queries like "what's due this week?" or "what fires on Mondays?"
// ---------------------------------------------------------------------------

/// Entities with a recurring schedule (e.g., "weekly sync every Monday").
pub trait HasCadence: Entity {
    fn cadence(&self) -> Option<&Cadence>;
    fn set_cadence(&mut self, cadence: Option<Cadence>);
}

/// Entities with a hard due date.
pub trait HasDeadline: Entity {
    fn deadline(&self) -> Option<DateTime<Utc>>;
    fn set_deadline(&mut self, deadline: Option<DateTime<Utc>>);
}

/// Entities scheduled at a specific date/time.
pub trait HasSchedule: Entity {
    fn scheduled_at(&self) -> Option<DateTime<Utc>>;
    fn set_scheduled_at(&mut self, at: Option<DateTime<Utc>>);
}
