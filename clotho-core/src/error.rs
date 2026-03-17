use crate::domain::types::{ExtractionStatus, TaskState};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClothoError {
    #[error(transparent)]
    Transition(#[from] TransitionError),

    #[error(transparent)]
    Promotion(#[from] PromotionError),

    #[error(transparent)]
    Graph(#[from] GraphError),
}

/// Errors from graph operations.
#[derive(Debug, Error)]
pub enum GraphError {
    #[error("failed to open graph database: {0}")]
    OpenFailed(String),

    #[error("graph query failed: {0}")]
    QueryFailed(String),

    #[error("node not found: {0}")]
    NodeNotFound(String),

    #[error("edge not found: {0} -[{1}]-> {2}")]
    EdgeNotFound(String, String, String),
}

/// Error when an invalid task state transition is attempted.
#[derive(Debug, Error)]
#[error("invalid transition from {from:?} to {to:?}")]
pub struct TransitionError {
    pub from: TaskState,
    pub to: TaskState,
}

/// Error when promoting a non-draft extraction.
#[derive(Debug, Error)]
#[error("cannot promote entity with status {status:?} (only Draft can be promoted)")]
pub struct PromotionError {
    pub status: ExtractionStatus,
}
