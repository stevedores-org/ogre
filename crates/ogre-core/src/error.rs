use crate::agent_lifecycle::AgentState;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OgreCoreError {
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidStateTransition { from: AgentState, to: AgentState },

    #[error("Workflow execution failed: {reason}")]
    WorkflowExecutionFailed { reason: String },

    #[error("Agent {agent_id} state not found")]
    AgentNotFound { agent_id: uuid::Uuid },

    #[error("Safety gate validation failed: {reason}")]
    SafetyGateError { reason: String },

    #[error("Failed to load checkpoint for agent {agent_id}")]
    CheckpointLoadError { agent_id: uuid::Uuid },

    #[error("Failed to save checkpoint for agent {agent_id}")]
    CheckpointSaveError { agent_id: uuid::Uuid },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, OgreCoreError>;
