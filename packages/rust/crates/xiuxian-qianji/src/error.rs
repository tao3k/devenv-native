//! Error surface for `xiuxian-qianji`.

use thiserror::Error;

/// Error type for `xiuxian-qianji` graph construction and execution.
#[derive(Error, Debug, Clone)]
pub enum QianjiError {
    /// Graph topology or manifest contract failure.
    #[error("Graph topology error: {0}")]
    Topology(String),

    /// Runtime node execution failure.
    #[error("Node execution failed: {0}")]
    Execution(String),

    /// Strategic drift or policy divergence.
    #[error("Strategic drift detected: {0}")]
    Drift(String),

    /// Capacity or resource-exhaustion failure.
    #[error("Resource exhaustion: {0}")]
    Capacity(String),

    /// Checkpoint persistence failure.
    #[error("Checkpoint persistence failed: {0}")]
    CheckpointError(String),

    /// Explicit execution abort signal.
    #[error("Execution aborted: {0}")]
    Aborted(String),
}
