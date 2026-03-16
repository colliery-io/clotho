//! LLM backend trait definitions and request/response types.
//!
//! Per CLO-A-0001 (trait-based LLM backend abstraction) and CLO-A-0003 (VSS).
//! These are trait definitions only — concrete implementations (Claude, Ollama, etc.)
//! live in clotho-extract/backends/.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::types::{EntityId, EntityType, SourceSpan};

// ---------------------------------------------------------------------------
// Extraction
// ---------------------------------------------------------------------------

/// Speech act classifications from transcript analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpeechAct {
    Commit,
    Decide,
    Risk,
    Block,
    Question,
    Insight,
    Delegate,
    Request,
    Update,
}

/// A single extraction from a transcript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extraction {
    pub speech_act: SpeechAct,
    pub text: String,
    pub source_span: SourceSpan,
    pub confidence: f32,
    /// Resolved or unresolved entity mentions.
    pub mentions: Vec<EntityMention>,
}

/// An entity mention found during extraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMention {
    pub raw_text: String,
    pub resolved_id: Option<EntityId>,
    pub resolved_type: Option<EntityType>,
    pub confidence: f32,
}

/// Request to extract speech acts and entity mentions from transcript content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionRequest {
    /// The transcript content to analyze.
    pub content: String,
    /// Known entities for resolution context.
    pub known_entities: Vec<KnownEntity>,
    /// Meeting metadata for context.
    pub meeting_title: Option<String>,
    pub attendees: Vec<String>,
}

/// A known entity provided for extraction resolution context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownEntity {
    pub id: EntityId,
    pub entity_type: EntityType,
    pub title: String,
    pub aliases: Vec<String>,
}

/// Result of extraction from a transcript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub extractions: Vec<Extraction>,
}

#[derive(Debug, Error)]
pub enum ExtractorError {
    #[error("extraction failed: {0}")]
    Failed(String),
    #[error("backend unavailable: {0}")]
    Unavailable(String),
    #[error("rate limited")]
    RateLimited,
}

/// Core trait for LLM-powered extraction from transcripts.
#[async_trait]
pub trait Extractor: Send + Sync {
    async fn extract(&self, request: ExtractionRequest) -> Result<ExtractionResult, ExtractorError>;
}

// ---------------------------------------------------------------------------
// Summarization
// ---------------------------------------------------------------------------

/// Request to generate a summary from content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryRequest {
    /// The content to summarize.
    pub content: String,
    /// Optional context about what kind of summary is needed.
    pub context: Option<String>,
    /// Maximum length hint (in tokens or characters, backend-dependent).
    pub max_length: Option<usize>,
}

/// Result of summarization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryResult {
    pub summary: String,
}

#[derive(Debug, Error)]
pub enum SummarizerError {
    #[error("summarization failed: {0}")]
    Failed(String),
    #[error("backend unavailable: {0}")]
    Unavailable(String),
    #[error("rate limited")]
    RateLimited,
}

/// Core trait for LLM-powered summarization.
#[async_trait]
pub trait Summarizer: Send + Sync {
    async fn summarize(
        &self,
        request: SummaryRequest,
    ) -> Result<SummaryResult, SummarizerError>;
}

// ---------------------------------------------------------------------------
// Entity Resolution
// ---------------------------------------------------------------------------

/// Request to resolve ambiguous entity mentions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionRequest {
    /// The ambiguous mentions to resolve.
    pub mentions: Vec<EntityMention>,
    /// Known entities to match against.
    pub known_entities: Vec<KnownEntity>,
    /// Surrounding context for disambiguation.
    pub context: String,
}

/// A single resolution result for one mention.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedMention {
    pub original: EntityMention,
    pub resolved_id: Option<EntityId>,
    pub confidence: f32,
}

/// Result of entity resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionResult {
    pub resolutions: Vec<ResolvedMention>,
}

#[derive(Debug, Error)]
pub enum ResolverError {
    #[error("resolution failed: {0}")]
    Failed(String),
    #[error("backend unavailable: {0}")]
    Unavailable(String),
    #[error("rate limited")]
    RateLimited,
}

/// Core trait for LLM-assisted entity resolution.
#[async_trait]
pub trait Resolver: Send + Sync {
    async fn resolve(
        &self,
        request: ResolutionRequest,
    ) -> Result<ResolutionResult, ResolverError>;
}

// ---------------------------------------------------------------------------
// Embedding
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum EmbedderError {
    #[error("embedding failed: {0}")]
    Failed(String),
    #[error("backend unavailable: {0}")]
    Unavailable(String),
    #[error("rate limited")]
    RateLimited,
    #[error("input too large: {0} chunks exceeds limit")]
    InputTooLarge(usize),
}

/// Core trait for generating vector embeddings from text.
///
/// Per CLO-A-0003: embeddings are stored in SQLite+VSS (index/vectors.db)
/// as a derived index for semantic similarity search.
#[async_trait]
pub trait Embedder: Send + Sync {
    /// Generate embeddings for a batch of text chunks.
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbedderError>;

    /// Embedding dimension for this backend (e.g., 1536 for OpenAI, varies by model).
    fn dimension(&self) -> usize;
}
