use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::domain::traits::*;
use crate::domain::types::*;
use crate::error::TransitionError;

// ---------------------------------------------------------------------------
// Workstream
// ---------------------------------------------------------------------------

/// Long-running work thread. May relate to Programs or Responsibilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workstream {
    pub id: EntityId,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: Status,
    pub tags: Vec<Tag>,
    pub content: String,
    pub cadence: Option<Cadence>,
}

impl Workstream {
    pub fn new(title: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: EntityId::new(),
            title: title.into(),
            created_at: now,
            updated_at: now,
            status: Status::Active,
            tags: Vec::new(),
            content: String::new(),
            cadence: None,
        }
    }
}

impl Entity for Workstream {
    fn id(&self) -> &EntityId {
        &self.id
    }
    fn entity_type(&self) -> EntityType {
        EntityType::Workstream
    }
    fn title(&self) -> &str {
        &self.title
    }
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl Activatable for Workstream {
    fn status(&self) -> Status {
        self.status
    }
    fn set_status(&mut self, status: Status) {
        self.status = status;
        self.updated_at = Utc::now();
    }
}

impl Relatable for Workstream {
    fn relations(&self, graph: &GraphStore) -> Vec<Relation> {
        graph
            .get_edges_from(self.id())
            .unwrap_or_default()
            .into_iter()
            .map(Relation::from)
            .collect()
    }
    fn graph_label(&self) -> &'static str {
        "Workstream"
    }
}

impl Taggable for Workstream {
    fn tags(&self) -> &[Tag] {
        &self.tags
    }
    fn add_tag(&mut self, tag: Tag) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }
    fn remove_tag(&mut self, tag: &str) {
        let len = self.tags.len();
        self.tags.retain(|t| t.as_str() != tag);
        if self.tags.len() != len {
            self.updated_at = Utc::now();
        }
    }
}

impl ContentBearing for Workstream {
    fn content(&self) -> &str {
        &self.content
    }
    fn set_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }
    fn content_path(&self) -> PathBuf {
        PathBuf::from(format!("content/workstreams/{}.md", self.id))
    }
}

impl HasCadence for Workstream {
    fn cadence(&self) -> Option<&Cadence> {
        self.cadence.as_ref()
    }
    fn set_cadence(&mut self, cadence: Option<Cadence>) {
        self.cadence = cadence;
        self.updated_at = Utc::now();
    }
}

// ---------------------------------------------------------------------------
// Task
// ---------------------------------------------------------------------------

/// Discrete work item with a state machine lifecycle.
///
/// Valid transitions:
/// - Todo → Doing
/// - Doing → Blocked, Done
/// - Blocked → Doing
/// - Done is terminal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: EntityId,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub state: TaskState,
    pub tags: Vec<Tag>,
    pub content: String,
    pub cadence: Option<Cadence>,
    pub deadline: Option<DateTime<Utc>>,
    pub scheduled_at: Option<DateTime<Utc>>,
}

impl Task {
    pub fn new(title: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: EntityId::new(),
            title: title.into(),
            created_at: now,
            updated_at: now,
            state: TaskState::Todo,
            tags: Vec::new(),
            content: String::new(),
            cadence: None,
            deadline: None,
            scheduled_at: None,
        }
    }
}

impl Entity for Task {
    fn id(&self) -> &EntityId {
        &self.id
    }
    fn entity_type(&self) -> EntityType {
        EntityType::Task
    }
    fn title(&self) -> &str {
        &self.title
    }
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl Taskable for Task {
    fn state(&self) -> TaskState {
        self.state
    }

    fn transition(&mut self, to: TaskState) -> Result<(), TransitionError> {
        let valid = self.valid_transitions();
        if valid.contains(&to) {
            self.state = to;
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(TransitionError {
                from: self.state,
                to,
            })
        }
    }

    fn valid_transitions(&self) -> Vec<TaskState> {
        match self.state {
            TaskState::Todo => vec![TaskState::Doing],
            TaskState::Doing => vec![TaskState::Blocked, TaskState::Done],
            TaskState::Blocked => vec![TaskState::Doing],
            TaskState::Done => vec![],
        }
    }
}

impl Relatable for Task {
    fn relations(&self, graph: &GraphStore) -> Vec<Relation> {
        graph
            .get_edges_from(self.id())
            .unwrap_or_default()
            .into_iter()
            .map(Relation::from)
            .collect()
    }
    fn graph_label(&self) -> &'static str {
        "Task"
    }
}

impl Taggable for Task {
    fn tags(&self) -> &[Tag] {
        &self.tags
    }
    fn add_tag(&mut self, tag: Tag) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }
    fn remove_tag(&mut self, tag: &str) {
        let len = self.tags.len();
        self.tags.retain(|t| t.as_str() != tag);
        if self.tags.len() != len {
            self.updated_at = Utc::now();
        }
    }
}

impl ContentBearing for Task {
    fn content(&self) -> &str {
        &self.content
    }
    fn set_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }
    fn content_path(&self) -> PathBuf {
        PathBuf::from(format!("content/tasks/{}.md", self.id))
    }
}

impl HasCadence for Task {
    fn cadence(&self) -> Option<&Cadence> {
        self.cadence.as_ref()
    }
    fn set_cadence(&mut self, cadence: Option<Cadence>) {
        self.cadence = cadence;
        self.updated_at = Utc::now();
    }
}

impl HasDeadline for Task {
    fn deadline(&self) -> Option<DateTime<Utc>> {
        self.deadline
    }
    fn set_deadline(&mut self, deadline: Option<DateTime<Utc>>) {
        self.deadline = deadline;
        self.updated_at = Utc::now();
    }
}

impl HasSchedule for Task {
    fn scheduled_at(&self) -> Option<DateTime<Utc>> {
        self.scheduled_at
    }
    fn set_scheduled_at(&mut self, at: Option<DateTime<Utc>>) {
        self.scheduled_at = at;
        self.updated_at = Utc::now();
    }
}
