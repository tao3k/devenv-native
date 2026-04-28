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
    /// Structured inclusive split / synchronization gateway.
    Inclusive,
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
    /// Intermediate throw event node.
    IntermediateThrowEvent,
    /// Intermediate catch event node.
    IntermediateCatchEvent,
    /// Boundary event attached to one host-blocking task.
    BoundaryEvent,
    /// Message-bound send task.
    SendTask,
    /// Message-bound receive task.
    ReceiveTask,
    /// Service task node.
    ServiceTask,
    /// Script task node dispatched through the host seam.
    ScriptTask,
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
    /// Optional bounded script-task metadata preserved for host dispatch.
    #[serde(default)]
    pub script_task: Option<BpmnScriptTaskSpec>,
    /// Optional human-task form metadata preserved for host dispatch.
    #[serde(default)]
    pub human_task_form: Option<BpmnHumanTaskFormSpec>,
    /// Optional standard BPMN human-task assignment metadata.
    #[serde(default)]
    pub human_task_assignment: Option<BpmnHumanTaskAssignmentSpec>,
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
            called_process_id: None,
            subprocess_kind: None,
            repeat: None,
            script_task: None,
            human_task_form: None,
            human_task_assignment: None,
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

/// Standard BPMN human-task assignment metadata preserved for host dispatch.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnHumanTaskAssignmentSpec {
    /// Human performer role declarations from the BPMN task.
    #[serde(default)]
    pub human_performers: Vec<BpmnHumanTaskResourceRoleSpec>,
    /// Potential owner role declarations from the BPMN task.
    #[serde(default)]
    pub potential_owners: Vec<BpmnHumanTaskResourceRoleSpec>,
}

impl BpmnHumanTaskAssignmentSpec {
    /// Creates an empty assignment metadata snapshot.
    #[must_use]
    pub fn new() -> Self {
        Self {
            human_performers: Vec::new(),
            potential_owners: Vec::new(),
        }
    }

    /// Adds one human performer role.
    #[must_use]
    pub fn with_human_performer(mut self, role: BpmnHumanTaskResourceRoleSpec) -> Self {
        self.human_performers.push(role);
        self
    }

    /// Adds one potential owner role.
    #[must_use]
    pub fn with_potential_owner(mut self, role: BpmnHumanTaskResourceRoleSpec) -> Self {
        self.potential_owners.push(role);
        self
    }
}

impl Default for BpmnHumanTaskAssignmentSpec {
    fn default() -> Self {
        Self::new()
    }
}

/// One standard BPMN resource role attached to a human task.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnHumanTaskResourceRoleSpec {
    /// Optional source-level role name.
    #[serde(default)]
    pub name: Option<Arc<str>>,
    /// Optional source-level `resourceRef` text.
    #[serde(default)]
    pub resource_ref: Option<Arc<str>>,
    /// Optional source-level `resourceAssignmentExpression/formalExpression` text.
    #[serde(default)]
    pub assignment_expression: Option<Arc<str>>,
}

impl BpmnHumanTaskResourceRoleSpec {
    /// Creates one standard BPMN resource role.
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: None,
            resource_ref: None,
            assignment_expression: None,
        }
    }

    /// Attaches a source-level role name.
    #[must_use]
    pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
        self.name = Some(Arc::<str>::from(name.as_ref()));
        self
    }

    /// Attaches a source-level resource reference.
    #[must_use]
    pub fn with_resource_ref(mut self, resource_ref: impl AsRef<str>) -> Self {
        self.resource_ref = Some(Arc::<str>::from(resource_ref.as_ref()));
        self
    }

    /// Attaches a source-level assignment expression.
    #[must_use]
    pub fn with_assignment_expression(mut self, assignment_expression: impl AsRef<str>) -> Self {
        self.assignment_expression = Some(Arc::<str>::from(assignment_expression.as_ref()));
        self
    }
}

impl Default for BpmnHumanTaskResourceRoleSpec {
    fn default() -> Self {
        Self::new()
    }
}

/// Bounded human-task form metadata preserved for host dispatch.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnHumanTaskFormSpec {
    /// Source-level qianji interaction type.
    pub interaction_type: Arc<str>,
    /// Optional variable reference containing the question prompt.
    #[serde(default)]
    pub question_ref: Option<Arc<str>>,
    /// Optional inline question prompt.
    #[serde(default)]
    pub question_text: Option<Arc<str>>,
    /// Optional variable reference containing choices.
    #[serde(default)]
    pub choices_ref: Option<Arc<str>>,
    /// Optional inline choices preserved from extension metadata.
    #[serde(default)]
    pub choices: Vec<BpmnHumanTaskChoiceSpec>,
    /// Optional free-text fields preserved from extension metadata.
    #[serde(default)]
    pub free_text_fields: Vec<BpmnHumanTaskFreeTextSpec>,
    /// Optional output variable that receives the primary interaction result.
    #[serde(default)]
    pub result_output: Option<Arc<str>>,
}

impl BpmnHumanTaskFormSpec {
    /// Creates one bounded human-task form metadata snapshot.
    #[must_use]
    pub fn new(interaction_type: impl AsRef<str>) -> Self {
        Self {
            interaction_type: Arc::<str>::from(interaction_type.as_ref()),
            question_ref: None,
            question_text: None,
            choices_ref: None,
            choices: Vec::new(),
            free_text_fields: Vec::new(),
            result_output: None,
        }
    }

    /// Attaches a dynamic question variable reference.
    #[must_use]
    pub fn with_question_ref(mut self, question_ref: impl AsRef<str>) -> Self {
        self.question_ref = Some(Arc::<str>::from(question_ref.as_ref()));
        self
    }

    /// Attaches an inline question prompt.
    #[must_use]
    pub fn with_question_text(mut self, question_text: impl AsRef<str>) -> Self {
        self.question_text = Some(Arc::<str>::from(question_text.as_ref()));
        self
    }

    /// Attaches a dynamic choices variable reference.
    #[must_use]
    pub fn with_choices_ref(mut self, choices_ref: impl AsRef<str>) -> Self {
        self.choices_ref = Some(Arc::<str>::from(choices_ref.as_ref()));
        self
    }

    /// Attaches inline choice metadata.
    #[must_use]
    pub fn with_choice(mut self, choice: BpmnHumanTaskChoiceSpec) -> Self {
        self.choices.push(choice);
        self
    }

    /// Attaches free-text field metadata.
    #[must_use]
    pub fn with_free_text_field(mut self, field: BpmnHumanTaskFreeTextSpec) -> Self {
        self.free_text_fields.push(field);
        self
    }

    /// Attaches the primary result output variable.
    #[must_use]
    pub fn with_result_output(mut self, result_output: impl AsRef<str>) -> Self {
        self.result_output = Some(Arc::<str>::from(result_output.as_ref()));
        self
    }
}

/// One inline choice for a bounded human-task form.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnHumanTaskChoiceSpec {
    /// Stable choice value submitted by the host.
    pub value: Arc<str>,
    /// Optional display label.
    #[serde(default)]
    pub label: Option<Arc<str>>,
}

impl BpmnHumanTaskChoiceSpec {
    /// Creates one inline choice.
    #[must_use]
    pub fn new(value: impl AsRef<str>) -> Self {
        Self {
            value: Arc::<str>::from(value.as_ref()),
            label: None,
        }
    }

    /// Attaches a display label.
    #[must_use]
    pub fn with_label(mut self, label: impl AsRef<str>) -> Self {
        self.label = Some(Arc::<str>::from(label.as_ref()));
        self
    }
}

/// One free-text field for a bounded human-task form.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnHumanTaskFreeTextSpec {
    /// Output field name.
    pub name: Arc<str>,
    /// Whether the host may omit this field.
    pub optional: bool,
}

impl BpmnHumanTaskFreeTextSpec {
    /// Creates one free-text field.
    #[must_use]
    pub fn new(name: impl AsRef<str>, optional: bool) -> Self {
        Self {
            name: Arc::<str>::from(name.as_ref()),
            optional,
        }
    }
}

/// Bounded script-task metadata preserved for host dispatch.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnScriptTaskSpec {
    /// Optional source-level `scriptFormat` attribute.
    #[serde(default)]
    pub script_format: Option<Arc<str>>,
    /// Optional nested `<bpmn:script>` body trimmed during parse-time capture.
    #[serde(default)]
    pub script_body: Option<Arc<str>>,
}

impl BpmnScriptTaskSpec {
    /// Creates one bounded script-task metadata snapshot.
    #[must_use]
    pub fn new(
        script_format: Option<impl AsRef<str>>,
        script_body: Option<impl AsRef<str>>,
    ) -> Self {
        Self {
            script_format: script_format.map(|format| Arc::<str>::from(format.as_ref())),
            script_body: script_body.map(|body| Arc::<str>::from(body.as_ref())),
        }
    }
}
