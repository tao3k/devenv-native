//! Host-bridge request and outcome shells.

use crate::dmn_model_api::{DmnEvaluationRequest, DmnEvaluationResult};
use crate::runtime::PendingHostWorkKind;
use crate::runtime::WaitRegistration;
use serde_json::Value;

/// Error returned by host-bridge implementations.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum HostBridgeError {
    /// Returned when the host has not implemented a bridge operation yet.
    #[error("host bridge operation '{operation}' is not implemented")]
    UnsupportedOperation {
        /// Operation name.
        operation: &'static str,
    },
    /// Returned when a host-level request fails.
    #[error("host bridge request failed: {0}")]
    RequestFailed(String),
}

/// Repeat-execution context attached to one blocked host request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RepeatExecutionContext {
    /// Sequential multi-instance cardinality iteration metadata.
    SequentialMultiInstance(SequentialMultiInstanceContext),
    /// Parallel multi-instance cardinality iteration metadata.
    ParallelMultiInstance(ParallelMultiInstanceContext),
}

/// Sequential multi-instance iteration metadata for one host request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SequentialMultiInstanceContext {
    /// Zero-based iteration index currently being executed.
    pub iteration_index: u32,
    /// Total planned iterations in this bounded multi-instance owner.
    pub total_iterations: u32,
}

/// Parallel multi-instance iteration metadata for one host request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParallelMultiInstanceContext {
    /// Zero-based iteration index currently being executed.
    pub iteration_index: u32,
    /// Total planned iterations in this bounded multi-instance owner.
    pub total_iterations: u32,
}

/// Common service-task dispatch request.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ServiceTaskRequest {
    /// Owning workflow instance identifier.
    pub instance_id: String,
    /// Owning runtime token identifier.
    pub token_id: u64,
    /// BPMN node index.
    pub node_index: u32,
    /// Current workflow variables snapshot.
    pub variables: Value,
    /// Optional repeat-execution metadata for the blocked task.
    pub repeat: Option<RepeatExecutionContext>,
}

/// Common service-task dispatch outcome.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ServiceTaskOutcome {
    /// Updated variables or task output payload.
    pub data: Value,
}

/// Common user-task dispatch request.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UserTaskRequest {
    /// Owning workflow instance identifier.
    pub instance_id: String,
    /// Owning runtime token identifier.
    pub token_id: u64,
    /// BPMN node index.
    pub node_index: u32,
    /// Current workflow variables snapshot.
    pub variables: Value,
    /// Optional repeat-execution metadata for the blocked task.
    pub repeat: Option<RepeatExecutionContext>,
}

/// Common user-task dispatch outcome.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UserTaskOutcome {
    /// Updated variables or user-supplied payload.
    pub data: Value,
}

/// Common manual-task dispatch request.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ManualTaskRequest {
    /// Owning workflow instance identifier.
    pub instance_id: String,
    /// Owning runtime token identifier.
    pub token_id: u64,
    /// BPMN node index.
    pub node_index: u32,
    /// Current workflow variables snapshot.
    pub variables: Value,
    /// Optional repeat-execution metadata for the blocked task.
    pub repeat: Option<RepeatExecutionContext>,
}

/// Common manual-task dispatch outcome.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ManualTaskOutcome {
    /// Updated variables or manual-operation payload.
    pub data: Value,
}

/// Common business-rule dispatch request.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BusinessRuleTaskRequest {
    /// Owning workflow instance identifier.
    pub instance_id: String,
    /// Owning runtime token identifier.
    pub token_id: u64,
    /// BPMN node index.
    pub node_index: u32,
    /// Current DMN evaluation request.
    pub evaluation: DmnEvaluationRequest,
    /// Optional repeat-execution metadata for the blocked task.
    pub repeat: Option<RepeatExecutionContext>,
}

/// Common business-rule dispatch outcome.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BusinessRuleTaskOutcome {
    /// Evaluated DMN result.
    pub evaluation: DmnEvaluationResult,
}

/// Request payload derived from one pending host-work boundary.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PendingHostWorkRequest {
    /// Dispatch request for a service task.
    Service(ServiceTaskRequest),
    /// Dispatch request for a user task.
    User(UserTaskRequest),
    /// Dispatch request for a manual task.
    Manual(ManualTaskRequest),
    /// Dispatch request for a business-rule task.
    BusinessRule(BusinessRuleTaskRequest),
}

impl PendingHostWorkRequest {
    /// Returns the host-work kind represented by this request.
    #[must_use]
    pub fn kind(&self) -> PendingHostWorkKind {
        match self {
            Self::Service(_) => PendingHostWorkKind::Service,
            Self::User(_) => PendingHostWorkKind::User,
            Self::Manual(_) => PendingHostWorkKind::Manual,
            Self::BusinessRule(_) => PendingHostWorkKind::BusinessRule,
        }
    }

    /// Returns a stable textual kind name for diagnostics.
    #[must_use]
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Service(_) => "service",
            Self::User(_) => "user",
            Self::Manual(_) => "manual",
            Self::BusinessRule(_) => "business_rule",
        }
    }
}

/// Result payload that resolves one pending host-work boundary.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PendingHostWorkResult {
    /// Completion payload for a service task.
    Service(ServiceTaskOutcome),
    /// Completion payload for a user task.
    User(UserTaskOutcome),
    /// Completion payload for a manual task.
    Manual(ManualTaskOutcome),
    /// Completion payload for a business-rule task.
    BusinessRule(BusinessRuleTaskOutcome),
}

impl PendingHostWorkResult {
    /// Returns the host-work kind represented by this result.
    #[must_use]
    pub fn kind(&self) -> PendingHostWorkKind {
        match self {
            Self::Service(_) => PendingHostWorkKind::Service,
            Self::User(_) => PendingHostWorkKind::User,
            Self::Manual(_) => PendingHostWorkKind::Manual,
            Self::BusinessRule(_) => PendingHostWorkKind::BusinessRule,
        }
    }

    /// Returns a stable textual kind name for diagnostics.
    #[must_use]
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Service(_) => "service",
            Self::User(_) => "user",
            Self::Manual(_) => "manual",
            Self::BusinessRule(_) => "business_rule",
        }
    }

    /// Returns the payload that should be merged into workflow variables.
    #[must_use]
    pub fn data(&self) -> &Value {
        match self {
            Self::Service(outcome) => &outcome.data,
            Self::User(outcome) => &outcome.data,
            Self::Manual(outcome) => &outcome.data,
            Self::BusinessRule(outcome) => &outcome.evaluation.output,
        }
    }
}

/// External-event poll request.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EventPollRequest {
    /// Owning workflow instance identifier.
    pub instance_id: String,
    /// Optional owning gateway node when multiple waits compete.
    pub gateway_node_index: Option<u32>,
    /// Waiting registrations currently blocking the instance.
    pub waits: Vec<WaitRegistration>,
}

/// External-event poll outcome.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EventPollOutcome {
    /// Whether the awaited event is now ready.
    pub ready: bool,
    /// Winning wait node when a competition group is active.
    pub winning_wait_node_index: Option<u32>,
    /// Optional event payload to merge into workflow variables.
    pub data: Value,
}
