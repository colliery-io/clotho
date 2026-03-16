use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Unique identifier for all entities.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(Uuid);

impl EntityId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for EntityId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<EntityId> for Uuid {
    fn from(id: EntityId) -> Self {
        id.0
    }
}

/// Enum of all 15 entity types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    // Structural layer
    Program,
    Responsibility,
    Objective,
    // Execution layer
    Workstream,
    Task,
    // Capture layer
    Meeting,
    Transcript,
    Note,
    Reflection,
    Artifact,
    // Derived layer
    Decision,
    Risk,
    Blocker,
    Question,
    Insight,
    // Cross-cutting
    Person,
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Program => "Program",
            Self::Responsibility => "Responsibility",
            Self::Objective => "Objective",
            Self::Workstream => "Workstream",
            Self::Task => "Task",
            Self::Meeting => "Meeting",
            Self::Transcript => "Transcript",
            Self::Note => "Note",
            Self::Reflection => "Reflection",
            Self::Artifact => "Artifact",
            Self::Decision => "Decision",
            Self::Risk => "Risk",
            Self::Blocker => "Blocker",
            Self::Question => "Question",
            Self::Insight => "Insight",
            Self::Person => "Person",
        };
        write!(f, "{}", s)
    }
}

/// Lifecycle status for Activatable entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Active,
    Inactive,
}

/// Workflow state for Taskable entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    Todo,
    Doing,
    Blocked,
    Done,
}

/// Extraction lifecycle for Extractable entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionStatus {
    Draft,
    Promoted,
    Discarded,
}

/// Reference to a span within a transcript.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub transcript_id: EntityId,
    pub start: usize,
    pub end: usize,
}

/// Freeform tag.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tag(String);

impl Tag {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Tag {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for Tag {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Period type for Reflections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeriodType {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Adhoc,
}

/// Recurring schedule frequency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Frequency {
    Daily,
    Weekly,
    Biweekly,
    Monthly,
    Quarterly,
    Yearly,
    Custom,
}

/// Recurring schedule metadata.
///
/// Stored on entity structs via the `HasCadence` trait AND materialized
/// as `HAS_CADENCE` graph edges for cross-entity temporal queries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cadence {
    pub frequency: Frequency,
    /// Cron expression for custom schedules.
    pub cron: Option<String>,
    /// Human-readable label (e.g., "weekly sync", "quarterly review").
    pub label: Option<String>,
    /// Next computed occurrence.
    pub next_occurrence: Option<DateTime<Utc>>,
}
