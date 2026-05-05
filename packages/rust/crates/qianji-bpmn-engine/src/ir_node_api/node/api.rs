//! Public ir node api node contracts for BPMN/DMN engine integration.

use super::human_task::{BpmnHumanTaskAssignmentSpec, BpmnHumanTaskFormSpec};
use super::kind::{BpmnGatewayKind, BpmnNodeKind, BpmnSubProcessKind};
use super::lane::BpmnLaneMembershipSpec;
use super::script::BpmnScriptTaskSpec;
use super::task_io::BpmnTaskIoSpec;
use crate::dmn_model_api::DmnDecisionRef;
use crate::ir_index_api::BpmnNodeIndex;
use crate::ir_repeat_api::BpmnRepeatSpec;
use std::sync::Arc;

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
    /// Optional BPMN lane membership metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lane: Option<BpmnLaneMembershipSpec>,
    /// Optional called-process identifier for bounded call activities.
    pub called_process_id: Option<Arc<str>>,
    /// Optional subprocess discriminator for subprocess-like nodes.
    pub subprocess_kind: Option<BpmnSubProcessKind>,
    /// Optional repeatable-task snapshot for bounded loop execution.
    pub repeat: Option<BpmnRepeatSpec>,
    /// Optional bounded script-task metadata preserved for host dispatch.
    #[serde(default)]
    pub script_task: Option<BpmnScriptTaskSpec>,
    /// Optional human-task form metadata preserved for host dispatch.
    #[serde(default)]
    pub human_task_form: Option<BpmnHumanTaskFormSpec>,
    /// Optional standard BPMN human-task assignment metadata.
    #[serde(default)]
    pub human_task_assignment: Option<BpmnHumanTaskAssignmentSpec>,
    /// Optional bounded BPMN task IO bindings for host dispatch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_io: Option<BpmnTaskIoSpec>,
    /// Optional attached host node for boundary events.
    pub attached_to: Option<BpmnNodeIndex>,
    /// Optional default outgoing edge for bounded conditional-gateway routing.
    #[serde(default)]
    pub default_outgoing_edge: Option<u32>,
    /// Optional matching structured inclusive-join node for one inclusive split.
    #[serde(default)]
    pub inclusive_join_node: Option<BpmnNodeIndex>,
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
            lane: None,
            called_process_id: None,
            subprocess_kind: None,
            repeat: None,
            script_task: None,
            human_task_form: None,
            human_task_assignment: None,
            task_io: None,
            attached_to: None,
            default_outgoing_edge: None,
            inclusive_join_node: None,
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

    /// Attaches optional BPMN lane membership metadata to the node.
    #[must_use]
    pub fn with_lane(mut self, lane: BpmnLaneMembershipSpec) -> Self {
        self.lane = Some(lane);
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

    /// Attaches bounded script-task metadata to the node.
    #[must_use]
    pub fn with_script_task(mut self, script_task: BpmnScriptTaskSpec) -> Self {
        self.script_task = Some(script_task);
        self
    }

    /// Attaches bounded human-task form metadata to the node.
    #[must_use]
    pub fn with_human_task_form(mut self, form: BpmnHumanTaskFormSpec) -> Self {
        self.human_task_form = Some(form);
        self
    }

    /// Attaches standard BPMN human-task assignment metadata to the node.
    #[must_use]
    pub fn with_human_task_assignment(mut self, assignment: BpmnHumanTaskAssignmentSpec) -> Self {
        self.human_task_assignment = Some(assignment);
        self
    }

    /// Attaches bounded standard BPMN task IO bindings to the node.
    #[must_use]
    pub fn with_task_io(mut self, task_io: BpmnTaskIoSpec) -> Self {
        self.task_io = Some(task_io);
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

    /// Declares one default outgoing edge for bounded conditional-gateway routing.
    #[must_use]
    pub fn with_default_outgoing_edge(mut self, edge_index: u32) -> Self {
        self.default_outgoing_edge = Some(edge_index);
        self
    }

    /// Declares the matching structured inclusive-join node for one inclusive split.
    #[must_use]
    pub fn with_inclusive_join_node(mut self, node_index: BpmnNodeIndex) -> Self {
        self.inclusive_join_node = Some(node_index);
        self
    }

    /// Marks this node as a bounded compensation handler activity.
    #[must_use]
    pub fn with_compensation_marker(mut self, is_for_compensation: bool) -> Self {
        self.is_for_compensation = is_for_compensation;
        self
    }
}
