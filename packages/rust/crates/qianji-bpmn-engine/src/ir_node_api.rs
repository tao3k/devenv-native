//! Public BPMN node contract owner.

use crate::dmn_model_api::DmnDecisionRef;
use crate::ir_index_api::BpmnNodeIndex;
use crate::ir_repeat_api::BpmnRepeatSpec;
use std::sync::Arc;

/// Supported bounded BPMN gateway kinds.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BpmnGatewayKind {
    /// Parallel fan-out / synchronization gateway.
    Parallel,
    /// Exclusive merge / deterministic single-route gateway.
    Exclusive,
    /// Event-based winner-takes-all gateway.
    EventBased,
}

/// Supported high-level BPMN node kinds for the scaffold slice.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BpmnNodeKind {
    /// Start event node.
    StartEvent,
    /// End event node.
    EndEvent,
    /// Intermediate catch event node.
    IntermediateCatchEvent,
    /// Boundary event attached to one host-blocking task.
    BoundaryEvent,
    /// Service task node.
    ServiceTask,
    /// User task node.
    UserTask,
    /// Manual task node.
    ManualTask,
    /// Business-rule task reserved for future DMN integration.
    BusinessRuleTask,
    /// Generic gateway node.
    Gateway,
    /// Subprocess or call-activity-like node.
    SubProcess,
}

/// Supported bounded subprocess ownership kinds.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BpmnSubProcessKind {
    /// One same-package call activity.
    CallActivity,
    /// One inline embedded subprocess body.
    Embedded,
    /// One inline transaction shell.
    Transaction,
}

/// Immutable BPMN node specification.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnNodeSpec {
    /// Dense runtime node index.
    pub index: BpmnNodeIndex,
    /// Stable BPMN identifier.
    pub bpmn_id: Arc<str>,
    /// Normalized node kind.
    pub kind: BpmnNodeKind,
    /// Optional bounded gateway discriminator for gateway nodes.
    pub gateway_kind: Option<BpmnGatewayKind>,
    /// Optional future DMN decision reference placeholder.
    pub decision: Option<DmnDecisionRef>,
    /// Optional called-process identifier for bounded call activities.
    pub called_process_id: Option<Arc<str>>,
    /// Optional subprocess discriminator for subprocess-like nodes.
    pub subprocess_kind: Option<BpmnSubProcessKind>,
    /// Optional repeatable-task snapshot for bounded loop execution.
    pub repeat: Option<BpmnRepeatSpec>,
    /// Optional attached host node for boundary events.
    pub attached_to: Option<BpmnNodeIndex>,
    /// Whether a boundary event interrupts the attached host work.
    pub cancel_activity: bool,
    /// Whether this activity is reserved as a compensation handler.
    pub is_for_compensation: bool,
}

impl BpmnNodeSpec {
    /// Creates a node specification.
    #[must_use]
    pub fn new(index: BpmnNodeIndex, bpmn_id: impl AsRef<str>, kind: BpmnNodeKind) -> Self {
        Self {
            index,
            bpmn_id: Arc::<str>::from(bpmn_id.as_ref()),
            kind,
            gateway_kind: None,
            decision: None,
            called_process_id: None,
            subprocess_kind: None,
            repeat: None,
            attached_to: None,
            cancel_activity: true,
            is_for_compensation: false,
        }
    }

    /// Attaches an optional bounded gateway discriminator to the node.
    #[must_use]
    pub fn with_gateway_kind(mut self, gateway_kind: BpmnGatewayKind) -> Self {
        self.gateway_kind = Some(gateway_kind);
        self
    }

    /// Attaches an optional DMN decision placeholder to the node.
    #[must_use]
    pub fn with_decision(mut self, decision: DmnDecisionRef) -> Self {
        self.decision = Some(decision);
        self
    }

    /// Attaches a bounded call-activity target process identifier.
    #[must_use]
    pub fn with_called_process(mut self, called_process_id: impl AsRef<str>) -> Self {
        self.called_process_id = Some(Arc::<str>::from(called_process_id.as_ref()));
        self
    }

    /// Attaches one bounded subprocess discriminator to the node.
    #[must_use]
    pub fn with_subprocess_kind(mut self, subprocess_kind: BpmnSubProcessKind) -> Self {
        self.subprocess_kind = Some(subprocess_kind);
        self
    }

    /// Attaches bounded repeatable-task metadata to the node.
    #[must_use]
    pub fn with_repeat(mut self, repeat: BpmnRepeatSpec) -> Self {
        self.repeat = Some(repeat);
        self
    }

    /// Attaches boundary-event ownership metadata to the node.
    #[must_use]
    pub fn with_boundary_attachment(
        mut self,
        attached_to: BpmnNodeIndex,
        cancel_activity: bool,
    ) -> Self {
        self.attached_to = Some(attached_to);
        self.cancel_activity = cancel_activity;
        self
    }

    /// Marks this node as a bounded compensation handler activity.
    #[must_use]
    pub fn with_compensation_marker(mut self, is_for_compensation: bool) -> Self {
        self.is_for_compensation = is_for_compensation;
        self
    }
}
