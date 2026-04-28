use crate::bpmn::backend::QianjiBpmnCheckpointStore;
use crate::bpmn::session::QianjiBpmnSession;
use qianji_bpmn_engine::{BpmnAdvanceOutcome, BpmnPackage, PendingHostWorkResult};
use serde_json::Value;
use std::sync::Arc;

/// Host-owned BPMN execution driver built on the session/checkpoint facade.
#[derive(Debug, Clone)]
pub struct QianjiBpmnExecutionDriver {
    pub(super) package: Arc<BpmnPackage>,
    pub(super) checkpoint_store: Option<QianjiBpmnCheckpointStore>,
}

/// Typed input for one host-owned BPMN execution attempt.
#[derive(Debug, Clone, PartialEq)]
pub struct QianjiBpmnExecutionRequest {
    /// BPMN process identifier to create when no checkpoint exists.
    pub process_id: String,
    /// Workflow instance identifier used for checkpoint lookup and fresh runs.
    pub instance_id: String,
    /// Optional initial variables for a fresh run.
    pub initial_variables: Option<Value>,
    /// Optional BPMN node id for a fresh synthetic start-at run.
    pub start_at_node_id: Option<String>,
    /// Millisecond timestamp used for fresh instance creation.
    pub started_at_ms: u64,
}

/// Explicit pending host-work completion target.
#[derive(Debug, Clone, PartialEq)]
pub struct QianjiBpmnPendingHostCompletion {
    /// Runtime token identifier for the pending host work.
    pub token_id: u64,
    /// BPMN process identifier expected for the pending host work.
    pub process_id: String,
    /// BPMN activity identifier expected for the pending host work.
    pub activity_id: String,
    /// Host-work result payload to apply.
    pub result: PendingHostWorkResult,
}

impl QianjiBpmnPendingHostCompletion {
    /// Creates one explicit pending host-work completion target.
    #[must_use]
    pub fn new(
        token_id: u64,
        process_id: impl Into<String>,
        activity_id: impl Into<String>,
        result: PendingHostWorkResult,
    ) -> Self {
        Self {
            token_id,
            process_id: process_id.into(),
            activity_id: activity_id.into(),
            result,
        }
    }
}

/// Execution result for one host-owned BPMN run attempt.
#[derive(Debug, Clone)]
pub struct QianjiBpmnExecutionReport {
    /// Session state after the bounded execution attempt.
    pub session: QianjiBpmnSession,
    /// Stable engine outcome reached by the driver.
    pub outcome: BpmnAdvanceOutcome,
    /// Whether the run resumed from a stored checkpoint.
    pub resumed_from_checkpoint: bool,
    /// Whether the driver saved a new checkpoint after the run.
    pub checkpoint_saved: bool,
    /// Whether the driver deleted stored checkpoint state after a terminal run.
    pub checkpoint_deleted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::bpmn) enum QianjiBpmnCheckpointLifecycle {
    Retain,
    DeleteOnTerminalOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::bpmn) enum QianjiBpmnHostCompletionAdvance {
    Stable,
    HostBoundary,
    HumanBoundary,
}

impl QianjiBpmnExecutionDriver {
    /// Creates one execution driver from a loaded package plus optional
    /// checkpoint storage.
    #[must_use]
    pub fn new(
        package: Arc<BpmnPackage>,
        checkpoint_store: Option<QianjiBpmnCheckpointStore>,
    ) -> Self {
        Self {
            package,
            checkpoint_store,
        }
    }
}

impl QianjiBpmnExecutionRequest {
    /// Creates one execution request for a BPMN run attempt.
    #[must_use]
    pub fn new(
        process_id: impl Into<String>,
        instance_id: impl Into<String>,
        initial_variables: Option<Value>,
        started_at_ms: u64,
    ) -> Self {
        Self {
            process_id: process_id.into(),
            instance_id: instance_id.into(),
            initial_variables,
            start_at_node_id: None,
            started_at_ms,
        }
    }

    /// Records a target BPMN node for a fresh synthetic start-at run.
    #[must_use]
    pub fn with_start_at_node_id(mut self, start_at_node_id: Option<String>) -> Self {
        self.start_at_node_id = start_at_node_id;
        self
    }
}
