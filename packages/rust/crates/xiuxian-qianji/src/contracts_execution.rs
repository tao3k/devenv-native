use serde::{Deserialize, Serialize};

/// Represents the execution status of a single mechanism node in the Qianji Box.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    /// Initial state before any execution attempt.
    Idle,
    /// Waiting in the scheduling queue.
    Queued,
    /// Currently performing logic.
    Executing,
    /// Waiting for multi-agent consensus agreement.
    ConsensusPending,
    /// Under adversarial audit (Synapse-Audit).
    Calibrating,
    /// Successfully finished execution.
    Completed,
    /// Terminal failure with an error message.
    Failed(String),
}

/// Control instructions emitted by nodes to manipulate the workflow execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowInstruction {
    /// Continue to next topological layer normally.
    Continue,
    /// Select a specific outgoing edge by its label (Probabilistic/Conditional).
    SelectBranch(String),
    /// Reset specific nodes to 'Idle' and restart their execution (Calibration Loop).
    RetryNodes(Vec<String>),
    /// Suspend workflow execution, save checkpoint, and yield control back to the caller.
    Suspend(String),
    /// Terminate the entire workflow immediately with a fatal error.
    Abort(String),
}

/// Structured output from a Qianji mechanism.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QianjiOutput {
    /// The resulting data to be merged into the shared context.
    pub data: serde_json::Value,
    /// The routing/flow instruction for the scheduler.
    pub instruction: FlowInstruction,
}
