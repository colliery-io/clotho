use crate::domain::types::{ExtractionStatus, TaskState};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClothoError {
    #[error(transparent)]
    Transition(#[from] TransitionError),

    #[error(transparent)]
    Promotion(#[from] PromotionError),
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
