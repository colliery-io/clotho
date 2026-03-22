use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::domain::traits::*;
use crate::domain::types::*;

/// Lightweight rolodex entry for people.
///
/// Used for entity resolution during AI extraction (fuzzy matching on name/email).
/// Notes are stored as markdown content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: EntityId,
    pub name: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<Tag>,
    /// Freeform notes about this person, stored as markdown.
    pub content: String,
}

impl Person {
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: EntityId::new(),
            name: name.into(),
            email: None,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            content: String::new(),
        }
    }

    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }
}

impl Entity for Person {
    fn id(&self) -> &EntityId {
        &self.id
    }
    fn entity_type(&self) -> EntityType {
        EntityType::Person
    }
    fn title(&self) -> &str {
        &self.name
    }
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl Relatable for Person {
    fn relations(&self, graph: &GraphStore) -> Vec<Relation> {
        graph
            .get_edges_from(self.id())
            .unwrap_or_default()
            .into_iter()
            .map(Relation::from)
            .collect()
    }
    fn graph_label(&self) -> &'static str {
        "Person"
    }
}

impl Taggable for Person {
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

impl ContentBearing for Person {
    fn content(&self) -> &str {
        &self.content
    }
    fn set_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }
    fn content_path(&self) -> PathBuf {
        PathBuf::from(format!("content/people/{}.md", self.id))
    }
}
