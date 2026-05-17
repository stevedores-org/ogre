use thiserror::Error;

#[derive(Error, Debug)]
pub enum OgreCoreError {
    #[error("Agent lifecycle error: {0}")]
    LifecycleError(String),

    #[error("Workflow execution error: {0}")]
    WorkflowError(String),

    #[error("Safety gate validation failed: {0}")]
    SafetyGateError(String),

    #[error("Checkpoint error: {0}")]
    CheckpointError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, OgreCoreError>;
