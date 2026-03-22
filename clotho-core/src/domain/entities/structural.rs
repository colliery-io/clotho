use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::domain::traits::*;
use crate::domain::types::*;

// ---------------------------------------------------------------------------
// Program
// ---------------------------------------------------------------------------

/// Strategic initiative with explicit objectives.
///
/// Examples: technical education, PMO establishment, monolith breakup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub id: EntityId,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: Status,
    pub tags: Vec<Tag>,
    pub content: String,
    pub cadence: Option<Cadence>,
}

impl Program {
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

impl Entity for Program {
    fn id(&self) -> &EntityId {
        &self.id
    }
    fn entity_type(&self) -> EntityType {
        EntityType::Program
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

impl Activatable for Program {
    fn status(&self) -> Status {
        self.status
    }
    fn set_status(&mut self, status: Status) {
        self.status = status;
        self.updated_at = Utc::now();
    }
}

impl Relatable for Program {
    fn relations(&self, graph: &GraphStore) -> Vec<Relation> {
        graph
            .get_edges_from(self.id())
            .unwrap_or_default()
            .into_iter()
            .map(Relation::from)
            .collect()
    }
    fn graph_label(&self) -> &'static str {
        "Program"
    }
}

impl Taggable for Program {
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

impl ContentBearing for Program {
    fn content(&self) -> &str {
        &self.content
    }
    fn set_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }
    fn content_path(&self) -> PathBuf {
        PathBuf::from(format!("content/programs/{}.md", self.id))
    }
}

impl HasCadence for Program {
    fn cadence(&self) -> Option<&Cadence> {
        self.cadence.as_ref()
    }
    fn set_cadence(&mut self, cadence: Option<Cadence>) {
        self.cadence = cadence;
        self.updated_at = Utc::now();
    }
}

// ---------------------------------------------------------------------------
// Responsibility
// ---------------------------------------------------------------------------

/// Ongoing role obligation that never "completes".
///
/// Examples: team mentorship, HR reporting, budget management, 1:1s.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Responsibility {
    pub id: EntityId,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: Status,
    pub tags: Vec<Tag>,
    pub content: String,
    pub cadence: Option<Cadence>,
}

impl Responsibility {
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

impl Entity for Responsibility {
    fn id(&self) -> &EntityId {
        &self.id
    }
    fn entity_type(&self) -> EntityType {
        EntityType::Responsibility
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

impl Activatable for Responsibility {
    fn status(&self) -> Status {
        self.status
    }
    fn set_status(&mut self, status: Status) {
        self.status = status;
        self.updated_at = Utc::now();
    }
}

impl Relatable for Responsibility {
    fn relations(&self, graph: &GraphStore) -> Vec<Relation> {
        graph
            .get_edges_from(self.id())
            .unwrap_or_default()
            .into_iter()
            .map(Relation::from)
            .collect()
    }
    fn graph_label(&self) -> &'static str {
        "Responsibility"
    }
}

impl Taggable for Responsibility {
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

impl ContentBearing for Responsibility {
    fn content(&self) -> &str {
        &self.content
    }
    fn set_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }
    fn content_path(&self) -> PathBuf {
        PathBuf::from(format!("content/responsibilities/{}.md", self.id))
    }
}

impl HasCadence for Responsibility {
    fn cadence(&self) -> Option<&Cadence> {
        self.cadence.as_ref()
    }
    fn set_cadence(&mut self, cadence: Option<Cadence>) {
        self.cadence = cadence;
        self.updated_at = Utc::now();
    }
}

// ---------------------------------------------------------------------------
// Objective
// ---------------------------------------------------------------------------

/// Outcome within a Program.
///
/// Belongs to exactly one Program.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Objective {
    pub id: EntityId,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: Status,
    pub tags: Vec<Tag>,
    pub content: String,
    /// The program this objective belongs to.
    pub program_id: EntityId,
    pub deadline: Option<DateTime<Utc>>,
}

impl Objective {
    pub fn new(title: impl Into<String>, program_id: EntityId) -> Self {
        let now = Utc::now();
        Self {
            id: EntityId::new(),
            title: title.into(),
            created_at: now,
            updated_at: now,
            status: Status::Active,
            tags: Vec::new(),
            content: String::new(),
            program_id,
            deadline: None,
        }
    }
}

impl Entity for Objective {
    fn id(&self) -> &EntityId {
        &self.id
    }
    fn entity_type(&self) -> EntityType {
        EntityType::Objective
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

impl Activatable for Objective {
    fn status(&self) -> Status {
        self.status
    }
    fn set_status(&mut self, status: Status) {
        self.status = status;
        self.updated_at = Utc::now();
    }
}

impl Relatable for Objective {
    fn relations(&self, graph: &GraphStore) -> Vec<Relation> {
        graph
            .get_edges_from(self.id())
            .unwrap_or_default()
            .into_iter()
            .map(Relation::from)
            .collect()
    }
    fn graph_label(&self) -> &'static str {
        "Objective"
    }
}

impl Taggable for Objective {
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

impl ContentBearing for Objective {
    fn content(&self) -> &str {
        &self.content
    }
    fn set_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }
    fn content_path(&self) -> PathBuf {
        PathBuf::from(format!("content/objectives/{}.md", self.id))
    }
}

impl HasDeadline for Objective {
    fn deadline(&self) -> Option<DateTime<Utc>> {
        self.deadline
    }
    fn set_deadline(&mut self, deadline: Option<DateTime<Utc>>) {
        self.deadline = deadline;
        self.updated_at = Utc::now();
    }
}
