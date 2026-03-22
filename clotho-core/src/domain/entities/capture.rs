use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::domain::traits::*;
use crate::domain::types::*;

// ---------------------------------------------------------------------------
// Meeting
// ---------------------------------------------------------------------------

/// Container entity for a meeting occurrence.
///
/// Has associated Transcript and/or Notes. Carries date, attendees, and
/// related entity references.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meeting {
    pub id: EntityId,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<Tag>,
    pub content: String,
    pub date: DateTime<Utc>,
    pub attendees: Vec<EntityId>,
    pub scheduled_at: Option<DateTime<Utc>>,
}

impl Meeting {
    pub fn new(title: impl Into<String>, date: DateTime<Utc>) -> Self {
        let now = Utc::now();
        Self {
            id: EntityId::new(),
            title: title.into(),
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            content: String::new(),
            date,
            attendees: Vec::new(),
            scheduled_at: Some(date),
        }
    }
}

impl Entity for Meeting {
    fn id(&self) -> &EntityId {
        &self.id
    }
    fn entity_type(&self) -> EntityType {
        EntityType::Meeting
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

impl Relatable for Meeting {
    fn relations(&self, graph: &GraphStore) -> Vec<Relation> {
        graph
            .get_edges_from(self.id())
            .unwrap_or_default()
            .into_iter()
            .map(Relation::from)
            .collect()
    }
    fn graph_label(&self) -> &'static str {
        "Meeting"
    }
}

impl Taggable for Meeting {
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

impl ContentBearing for Meeting {
    fn content(&self) -> &str {
        &self.content
    }
    fn set_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }
    fn content_path(&self) -> PathBuf {
        let date_str = self.date.format("%Y-%m-%d");
        PathBuf::from(format!("content/meetings/{}-{}.md", date_str, self.id))
    }
}

impl HasSchedule for Meeting {
    fn scheduled_at(&self) -> Option<DateTime<Utc>> {
        self.scheduled_at
    }
    fn set_scheduled_at(&mut self, at: Option<DateTime<Utc>>) {
        self.scheduled_at = at;
        self.updated_at = Utc::now();
    }
}

// ---------------------------------------------------------------------------
// Transcript
// ---------------------------------------------------------------------------

/// Raw meeting content, typically from a transcription service.
///
/// Source for AI extraction. Belongs to exactly one Meeting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    pub id: EntityId,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<Tag>,
    pub content: String,
    /// The meeting this transcript belongs to.
    pub meeting_id: EntityId,
}

impl Transcript {
    pub fn new(title: impl Into<String>, meeting_id: EntityId) -> Self {
        let now = Utc::now();
        Self {
            id: EntityId::new(),
            title: title.into(),
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            content: String::new(),
            meeting_id,
        }
    }
}

impl Entity for Transcript {
    fn id(&self) -> &EntityId {
        &self.id
    }
    fn entity_type(&self) -> EntityType {
        EntityType::Transcript
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

impl Relatable for Transcript {
    fn relations(&self, graph: &GraphStore) -> Vec<Relation> {
        graph
            .get_edges_from(self.id())
            .unwrap_or_default()
            .into_iter()
            .map(Relation::from)
            .collect()
    }
    fn graph_label(&self) -> &'static str {
        "Transcript"
    }
}

impl Taggable for Transcript {
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

impl ContentBearing for Transcript {
    fn content(&self) -> &str {
        &self.content
    }
    fn set_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }
    fn content_path(&self) -> PathBuf {
        PathBuf::from(format!("content/meetings/{}.transcript.md", self.id))
    }
}

// ---------------------------------------------------------------------------
// Note
// ---------------------------------------------------------------------------

/// Authored content, freeform.
///
/// Can belong to a Meeting or stand alone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: EntityId,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<Tag>,
    pub content: String,
    /// Optional meeting this note is attached to.
    pub meeting_id: Option<EntityId>,
}

impl Note {
    pub fn new(title: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: EntityId::new(),
            title: title.into(),
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            content: String::new(),
            meeting_id: None,
        }
    }

    pub fn for_meeting(title: impl Into<String>, meeting_id: EntityId) -> Self {
        let mut note = Self::new(title);
        note.meeting_id = Some(meeting_id);
        note
    }
}

impl Entity for Note {
    fn id(&self) -> &EntityId {
        &self.id
    }
    fn entity_type(&self) -> EntityType {
        EntityType::Note
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

impl Relatable for Note {
    fn relations(&self, graph: &GraphStore) -> Vec<Relation> {
        graph
            .get_edges_from(self.id())
            .unwrap_or_default()
            .into_iter()
            .map(Relation::from)
            .collect()
    }
    fn graph_label(&self) -> &'static str {
        "Note"
    }
}

impl Taggable for Note {
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

impl ContentBearing for Note {
    fn content(&self) -> &str {
        &self.content
    }
    fn set_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }
    fn content_path(&self) -> PathBuf {
        PathBuf::from(format!("content/notes/{}.md", self.id))
    }
}

// ---------------------------------------------------------------------------
// Reflection
// ---------------------------------------------------------------------------

/// Time-period bound thinking.
///
/// Period types: daily, weekly, monthly, quarterly, adhoc.
/// May relate to Programs for scoped reflection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reflection {
    pub id: EntityId,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<Tag>,
    pub content: String,
    pub period_type: PeriodType,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub period_name: Option<String>,
    /// Programs this reflection is scoped to.
    pub program_ids: Vec<EntityId>,
}

impl Reflection {
    pub fn new(
        title: impl Into<String>,
        period_type: PeriodType,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: EntityId::new(),
            title: title.into(),
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            content: String::new(),
            period_type,
            period_start,
            period_end,
            period_name: None,
            program_ids: Vec::new(),
        }
    }
}

impl Entity for Reflection {
    fn id(&self) -> &EntityId {
        &self.id
    }
    fn entity_type(&self) -> EntityType {
        EntityType::Reflection
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

impl Relatable for Reflection {
    fn relations(&self, graph: &GraphStore) -> Vec<Relation> {
        graph
            .get_edges_from(self.id())
            .unwrap_or_default()
            .into_iter()
            .map(Relation::from)
            .collect()
    }
    fn graph_label(&self) -> &'static str {
        "Reflection"
    }
}

impl Taggable for Reflection {
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

impl ContentBearing for Reflection {
    fn content(&self) -> &str {
        &self.content
    }
    fn set_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }
    fn content_path(&self) -> PathBuf {
        let period = match self.period_type {
            PeriodType::Daily => "daily",
            PeriodType::Weekly => "weekly",
            PeriodType::Monthly => "monthly",
            PeriodType::Quarterly => "quarterly",
            PeriodType::Adhoc => "adhoc",
        };
        PathBuf::from(format!("content/reflections/{}-{}.md", period, self.id))
    }
}

// ---------------------------------------------------------------------------
// Artifact
// ---------------------------------------------------------------------------

/// Deliverable with external source link.
///
/// Examples: design docs, PRs, presentations, shipped features.
/// Ingested as markdown with link to original.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: EntityId,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<Tag>,
    pub content: String,
    /// Link to the original source (URL, file path, etc.).
    pub source_url: String,
    pub deadline: Option<DateTime<Utc>>,
}

impl Artifact {
    pub fn new(title: impl Into<String>, source_url: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: EntityId::new(),
            title: title.into(),
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            content: String::new(),
            source_url: source_url.into(),
            deadline: None,
        }
    }
}

impl Entity for Artifact {
    fn id(&self) -> &EntityId {
        &self.id
    }
    fn entity_type(&self) -> EntityType {
        EntityType::Artifact
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

impl Relatable for Artifact {
    fn relations(&self, graph: &GraphStore) -> Vec<Relation> {
        graph
            .get_edges_from(self.id())
            .unwrap_or_default()
            .into_iter()
            .map(Relation::from)
            .collect()
    }
    fn graph_label(&self) -> &'static str {
        "Artifact"
    }
}

impl Taggable for Artifact {
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

impl ContentBearing for Artifact {
    fn content(&self) -> &str {
        &self.content
    }
    fn set_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }
    fn content_path(&self) -> PathBuf {
        PathBuf::from(format!("content/artifacts/{}.md", self.id))
    }
}

impl HasDeadline for Artifact {
    fn deadline(&self) -> Option<DateTime<Utc>> {
        self.deadline
    }
    fn set_deadline(&mut self, deadline: Option<DateTime<Utc>>) {
        self.deadline = deadline;
        self.updated_at = Utc::now();
    }
}
