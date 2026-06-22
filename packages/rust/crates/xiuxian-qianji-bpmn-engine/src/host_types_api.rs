//! Host-bridge request and outcome shells.

use crate::dmn_model_api::{DmnEvaluationRequest, DmnEvaluationResult};
use crate::ir_node_api::{
    BpmnHumanTaskAssignmentSpec, BpmnHumanTaskFormSpec, BpmnLaneMembershipSpec,
    BpmnTaskOutputBinding,
};
use crate::runtime::WaitRegistration;
use crate::runtime::{PendingHostWorkClaim, PendingHostWorkKind};
use serde_json::Value;

macro_rules! string_id_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Borrows the serialized identifier.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::ops::Deref for $name {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                self.as_str()
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }

        impl From<std::sync::Arc<str>> for $name {
            fn from(value: std::sync::Arc<str>) -> Self {
                Self(value.to_string())
            }
        }

        impl PartialEq<&str> for $name {
            fn eq(&self, other: &&str) -> bool {
                self.as_str() == *other
            }
        }
    };
}

string_id_type!(
    /// Workflow instance identifier serialized through host-bridge requests.
    BpmnHostInstanceId
);
string_id_type!(
    /// BPMN process identifier serialized through host-bridge requests.
    BpmnHostProcessId
);
string_id_type!(
    /// BPMN activity identifier serialized through host-bridge requests.
    BpmnHostActivityId
);
string_id_type!(
    /// Host-generated pending work identifier serialized through runtime state.
    BpmnHostWorkId
);

/// Runtime token identifier serialized through host-bridge requests.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct BpmnHostTokenId(u64);

impl BpmnHostTokenId {
    /// Returns the serialized runtime token identifier.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for BpmnHostTokenId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<BpmnHostTokenId> for u64 {
    fn from(value: BpmnHostTokenId) -> Self {
        value.get()
    }
}

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

/// Common send-task dispatch request.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SendTaskRequest {
    /// Owning workflow instance identifier.
    pub instance_id: String,
    /// Owning runtime token identifier.
    pub token_id: u64,
    /// BPMN node index.
    pub node_index: u32,
    /// Required source-level message reference.
    pub message_reference: String,
    /// Optional resolved event name or fallback label.
    pub message_name: Option<String>,
    /// Current workflow variables snapshot.
    pub variables: Value,
    /// Resolved standard BPMN task inputs for this dispatch.
    #[serde(default)]
    pub inputs: Value,
    /// Declared standard BPMN task outputs for strict completion mapping.
    #[serde(default)]
    pub output_bindings: Vec<BpmnTaskOutputBinding>,
}

/// Common send-task dispatch outcome.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SendTaskOutcome {
    /// Updated variables or send-operation payload.
    pub data: Value,
}

/// Common generic BPMN task dispatch request.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TaskRequest {
    /// Owning workflow instance identifier.
    pub instance_id: BpmnHostInstanceId,
    /// Owning BPMN process identifier.
    pub process_id: BpmnHostProcessId,
    /// Owning runtime token identifier.
    pub token_id: BpmnHostTokenId,
    /// BPMN node index.
    pub node_index: u32,
    /// Stable BPMN activity identifier.
    pub activity_id: BpmnHostActivityId,
    /// Current workflow variables snapshot.
    pub variables: Value,
    /// Resolved standard BPMN task inputs for this dispatch.
    #[serde(default)]
    pub inputs: Value,
    /// Declared standard BPMN task outputs for strict completion mapping.
    #[serde(default)]
    pub output_bindings: Vec<BpmnTaskOutputBinding>,
    /// Optional repeat-execution metadata for the blocked task.
    pub repeat: Option<RepeatExecutionContext>,
    /// Optional BPMN lane membership metadata for passive routing/display.
    pub lane: Option<BpmnLaneMembershipSpec>,
}

/// Common generic BPMN task dispatch outcome.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TaskOutcome {
    /// Updated variables or task output payload.
    pub data: Value,
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
    /// Resolved standard BPMN task inputs for this dispatch.
    #[serde(default)]
    pub inputs: Value,
    /// Declared standard BPMN task outputs for strict completion mapping.
    #[serde(default)]
    pub output_bindings: Vec<BpmnTaskOutputBinding>,
    /// Optional repeat-execution metadata for the blocked task.
    pub repeat: Option<RepeatExecutionContext>,
}

/// Common service-task dispatch outcome.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ServiceTaskOutcome {
    /// Updated variables or task output payload.
    pub data: Value,
}

/// Common script-task dispatch request.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScriptTaskRequest {
    /// Owning workflow instance identifier.
    pub instance_id: String,
    /// Owning runtime token identifier.
    pub token_id: u64,
    /// BPMN node index.
    pub node_index: u32,
    /// Optional source-level `scriptFormat` attribute.
    pub script_format: Option<String>,
    /// Optional nested `<bpmn:script>` body.
    pub script_body: Option<String>,
    /// Current workflow variables snapshot.
    pub variables: Value,
    /// Resolved standard BPMN task inputs for this dispatch.
    #[serde(default)]
    pub inputs: Value,
    /// Declared standard BPMN task outputs for strict completion mapping.
    #[serde(default)]
    pub output_bindings: Vec<BpmnTaskOutputBinding>,
    /// Optional repeat-execution metadata for the blocked task.
    pub repeat: Option<RepeatExecutionContext>,
}

/// Common script-task dispatch outcome.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScriptTaskOutcome {
    /// Updated variables or script output payload.
    pub data: Value,
}

/// Common user-task dispatch request.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UserTaskRequest {
    /// Owning workflow instance identifier.
    pub instance_id: BpmnHostInstanceId,
    /// Owning BPMN process identifier.
    pub process_id: BpmnHostProcessId,
    /// Owning runtime token identifier.
    pub token_id: BpmnHostTokenId,
    /// BPMN node index.
    pub node_index: u32,
    /// Stable BPMN activity identifier.
    pub activity_id: BpmnHostActivityId,
    /// Current workflow variables snapshot.
    pub variables: Value,
    /// Resolved standard BPMN task inputs for this dispatch.
    #[serde(default)]
    pub inputs: Value,
    /// Declared standard BPMN task outputs for strict completion mapping.
    #[serde(default)]
    pub output_bindings: Vec<BpmnTaskOutputBinding>,
    /// Optional repeat-execution metadata for the blocked task.
    pub repeat: Option<RepeatExecutionContext>,
    /// Optional BPMN lane membership metadata for passive routing/display.
    pub lane: Option<BpmnLaneMembershipSpec>,
    /// Optional human-task form metadata preserved for host rendering.
    pub form: Option<BpmnHumanTaskFormSpec>,
    /// Optional standard BPMN assignment metadata preserved for host routing.
    pub assignment: Option<BpmnHumanTaskAssignmentSpec>,
    /// Optional checkpointed claim metadata for this pending human task.
    pub claim: Option<PendingHostWorkClaim>,
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
    pub instance_id: BpmnHostInstanceId,
    /// Owning BPMN process identifier.
    pub process_id: BpmnHostProcessId,
    /// Owning runtime token identifier.
    pub token_id: BpmnHostTokenId,
    /// BPMN node index.
    pub node_index: u32,
    /// Stable BPMN activity identifier.
    pub activity_id: BpmnHostActivityId,
    /// Current workflow variables snapshot.
    pub variables: Value,
    /// Resolved standard BPMN task inputs for this dispatch.
    #[serde(default)]
    pub inputs: Value,
    /// Declared standard BPMN task outputs for strict completion mapping.
    #[serde(default)]
    pub output_bindings: Vec<BpmnTaskOutputBinding>,
    /// Optional repeat-execution metadata for the blocked task.
    pub repeat: Option<RepeatExecutionContext>,
    /// Optional BPMN lane membership metadata for passive routing/display.
    pub lane: Option<BpmnLaneMembershipSpec>,
    /// Optional human-task form metadata preserved for host rendering.
    pub form: Option<BpmnHumanTaskFormSpec>,
    /// Optional standard BPMN assignment metadata preserved for host routing.
    pub assignment: Option<BpmnHumanTaskAssignmentSpec>,
    /// Optional checkpointed claim metadata for this pending human task.
    pub claim: Option<PendingHostWorkClaim>,
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
    /// Resolved standard BPMN task inputs used for host-visible dispatch.
    #[serde(default)]
    pub inputs: Value,
    /// Declared standard BPMN task outputs for strict completion mapping.
    #[serde(default)]
    pub output_bindings: Vec<BpmnTaskOutputBinding>,
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
    /// Dispatch request for a generic BPMN task.
    Task(TaskRequest),
    /// Dispatch request for a send task.
    Send(SendTaskRequest),
    /// Dispatch request for a service task.
    Service(ServiceTaskRequest),
    /// Dispatch request for a script task.
    Script(ScriptTaskRequest),
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
            Self::Task(_) => PendingHostWorkKind::Task,
            Self::Send(_) => PendingHostWorkKind::Send,
            Self::Service(_) => PendingHostWorkKind::Service,
            Self::Script(_) => PendingHostWorkKind::Script,
            Self::User(_) => PendingHostWorkKind::User,
            Self::Manual(_) => PendingHostWorkKind::Manual,
            Self::BusinessRule(_) => PendingHostWorkKind::BusinessRule,
        }
    }

    /// Returns a stable textual kind name for diagnostics.
    #[must_use]
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Task(_) => "task",
            Self::Send(_) => "send",
            Self::Service(_) => "service",
            Self::Script(_) => "script",
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
    /// Completion payload for a generic BPMN task.
    Task(TaskOutcome),
    /// Completion payload for a send task.
    Send(SendTaskOutcome),
    /// Completion payload for a service task.
    Service(ServiceTaskOutcome),
    /// Completion payload for a script task.
    Script(ScriptTaskOutcome),
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
            Self::Task(_) => PendingHostWorkKind::Task,
            Self::Send(_) => PendingHostWorkKind::Send,
            Self::Service(_) => PendingHostWorkKind::Service,
            Self::Script(_) => PendingHostWorkKind::Script,
            Self::User(_) => PendingHostWorkKind::User,
            Self::Manual(_) => PendingHostWorkKind::Manual,
            Self::BusinessRule(_) => PendingHostWorkKind::BusinessRule,
        }
    }

    /// Returns a stable textual kind name for diagnostics.
    #[must_use]
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Task(_) => "task",
            Self::Send(_) => "send",
            Self::Service(_) => "service",
            Self::Script(_) => "script",
            Self::User(_) => "user",
            Self::Manual(_) => "manual",
            Self::BusinessRule(_) => "business_rule",
        }
    }

    /// Returns the payload that should be merged into workflow variables.
    #[must_use]
    pub fn data(&self) -> &Value {
        match self {
            Self::Task(outcome) => &outcome.data,
            Self::Send(outcome) => &outcome.data,
            Self::Service(outcome) => &outcome.data,
            Self::Script(outcome) => &outcome.data,
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
