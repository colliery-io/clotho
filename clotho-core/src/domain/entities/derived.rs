use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::traits::*;
use crate::domain::types::*;
use crate::error::PromotionError;

// ---------------------------------------------------------------------------
// Shared helpers for Extractable entities
// ---------------------------------------------------------------------------

/// Common fields for all derived (extractable) entities.
macro_rules! impl_extractable {
    ($ty:ident) => {
        impl Extractable for $ty {
            fn extraction_status(&self) -> ExtractionStatus {
                self.extraction_status
            }

            fn source_span(&self) -> Option<&SourceSpan> {
                self.source_span.as_ref()
            }

            fn confidence(&self) -> f32 {
                self.confidence
            }

            fn promote(&mut self) -> Result<(), PromotionError> {
                if self.extraction_status == ExtractionStatus::Draft {
                    self.extraction_status = ExtractionStatus::Promoted;
                    self.updated_at = Utc::now();
                    Ok(())
                } else {
                    Err(PromotionError {
                        status: self.extraction_status,
                    })
                }
            }

            fn discard(&mut self) {
                if self.extraction_status == ExtractionStatus::Draft {
                    self.extraction_status = ExtractionStatus::Discarded;
                    self.updated_at = Utc::now();
                }
            }
        }
    };
}

macro_rules! impl_relatable {
    ($ty:ident, $label:expr) => {
        impl Relatable for $ty {
            fn relations(&self, graph: &GraphStore) -> Vec<Relation> {
                graph.get_edges_from(self.id()).unwrap_or_default().into_iter().map(Relation::from).collect()
            }
            fn graph_label(&self) -> &'static str { $label }
        }
    };
}

macro_rules! impl_taggable {
    ($ty:ident) => {
        impl Taggable for $ty {
            fn tags(&self) -> &[Tag] { &self.tags }
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
    };
}

// ---------------------------------------------------------------------------
// Decision
// ---------------------------------------------------------------------------

/// Recorded decision point, extracted from transcripts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: EntityId,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub extraction_status: ExtractionStatus,
    pub source_span: Option<SourceSpan>,
    pub confidence: f32,
    pub tags: Vec<Tag>,
}

impl Decision {
    pub fn draft(title: impl Into<String>, confidence: f32, source_span: Option<SourceSpan>) -> Self {
        let now = Utc::now();
        Self {
            id: EntityId::new(),
            title: title.into(),
            created_at: now,
            updated_at: now,
            extraction_status: ExtractionStatus::Draft,
            source_span,
            confidence,
            tags: Vec::new(),
        }
    }
}

impl Entity for Decision {
    fn id(&self) -> &EntityId { &self.id }
    fn entity_type(&self) -> EntityType { EntityType::Decision }
    fn title(&self) -> &str { &self.title }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
}

impl_extractable!(Decision);
impl_relatable!(Decision, "Decision");
impl_taggable!(Decision);

// ---------------------------------------------------------------------------
// Risk
// ---------------------------------------------------------------------------

/// Identified risk, extracted from transcripts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub id: EntityId,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub extraction_status: ExtractionStatus,
    pub source_span: Option<SourceSpan>,
    pub confidence: f32,
    pub tags: Vec<Tag>,
    pub deadline: Option<DateTime<Utc>>,
}

impl Risk {
    pub fn draft(title: impl Into<String>, confidence: f32, source_span: Option<SourceSpan>) -> Self {
        let now = Utc::now();
        Self {
            id: EntityId::new(),
            title: title.into(),
            created_at: now,
            updated_at: now,
            extraction_status: ExtractionStatus::Draft,
            source_span,
            confidence,
            tags: Vec::new(),
            deadline: None,
        }
    }
}

impl Entity for Risk {
    fn id(&self) -> &EntityId { &self.id }
    fn entity_type(&self) -> EntityType { EntityType::Risk }
    fn title(&self) -> &str { &self.title }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
}

impl_extractable!(Risk);
impl_relatable!(Risk, "Risk");
impl_taggable!(Risk);

impl HasDeadline for Risk {
    fn deadline(&self) -> Option<DateTime<Utc>> { self.deadline }
    fn set_deadline(&mut self, deadline: Option<DateTime<Utc>>) {
        self.deadline = deadline;
        self.updated_at = Utc::now();
    }
}

// ---------------------------------------------------------------------------
// Blocker
// ---------------------------------------------------------------------------

/// Impediment to progress, extracted from transcripts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blocker {
    pub id: EntityId,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub extraction_status: ExtractionStatus,
    pub source_span: Option<SourceSpan>,
    pub confidence: f32,
    pub tags: Vec<Tag>,
    pub deadline: Option<DateTime<Utc>>,
}

impl Blocker {
    pub fn draft(title: impl Into<String>, confidence: f32, source_span: Option<SourceSpan>) -> Self {
        let now = Utc::now();
        Self {
            id: EntityId::new(),
            title: title.into(),
            created_at: now,
            updated_at: now,
            extraction_status: ExtractionStatus::Draft,
            source_span,
            confidence,
            tags: Vec::new(),
            deadline: None,
        }
    }
}

impl Entity for Blocker {
    fn id(&self) -> &EntityId { &self.id }
    fn entity_type(&self) -> EntityType { EntityType::Blocker }
    fn title(&self) -> &str { &self.title }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
}

impl_extractable!(Blocker);
impl_relatable!(Blocker, "Blocker");
impl_taggable!(Blocker);

impl HasDeadline for Blocker {
    fn deadline(&self) -> Option<DateTime<Utc>> { self.deadline }
    fn set_deadline(&mut self, deadline: Option<DateTime<Utc>>) {
        self.deadline = deadline;
        self.updated_at = Utc::now();
    }
}

// ---------------------------------------------------------------------------
// Question
// ---------------------------------------------------------------------------

/// Open question requiring resolution, extracted from transcripts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: EntityId,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub extraction_status: ExtractionStatus,
    pub source_span: Option<SourceSpan>,
    pub confidence: f32,
    pub tags: Vec<Tag>,
    pub deadline: Option<DateTime<Utc>>,
}

impl Question {
    pub fn draft(title: impl Into<String>, confidence: f32, source_span: Option<SourceSpan>) -> Self {
        let now = Utc::now();
        Self {
            id: EntityId::new(),
            title: title.into(),
            created_at: now,
            updated_at: now,
            extraction_status: ExtractionStatus::Draft,
            source_span,
            confidence,
            tags: Vec::new(),
            deadline: None,
        }
    }
}

impl Entity for Question {
    fn id(&self) -> &EntityId { &self.id }
    fn entity_type(&self) -> EntityType { EntityType::Question }
    fn title(&self) -> &str { &self.title }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
}

impl_extractable!(Question);
impl_relatable!(Question, "Question");
impl_taggable!(Question);

impl HasDeadline for Question {
    fn deadline(&self) -> Option<DateTime<Utc>> { self.deadline }
    fn set_deadline(&mut self, deadline: Option<DateTime<Utc>>) {
        self.deadline = deadline;
        self.updated_at = Utc::now();
    }
}

// ---------------------------------------------------------------------------
// Insight
// ---------------------------------------------------------------------------

/// Learning or observation worth preserving, extracted from transcripts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    pub id: EntityId,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub extraction_status: ExtractionStatus,
    pub source_span: Option<SourceSpan>,
    pub confidence: f32,
    pub tags: Vec<Tag>,
}

impl Insight {
    pub fn draft(title: impl Into<String>, confidence: f32, source_span: Option<SourceSpan>) -> Self {
        let now = Utc::now();
        Self {
            id: EntityId::new(),
            title: title.into(),
            created_at: now,
            updated_at: now,
            extraction_status: ExtractionStatus::Draft,
            source_span,
            confidence,
            tags: Vec::new(),
        }
    }
}

impl Entity for Insight {
    fn id(&self) -> &EntityId { &self.id }
    fn entity_type(&self) -> EntityType { EntityType::Insight }
    fn title(&self) -> &str { &self.title }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
}

impl_extractable!(Insight);
impl_relatable!(Insight, "Insight");
impl_taggable!(Insight);
