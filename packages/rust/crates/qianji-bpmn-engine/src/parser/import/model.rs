use crate::dmn_model_api::DmnDecisionRef;
use crate::ir_event_api::{BpmnEventKind, BpmnTimerKind};
use crate::ir_node_api::{BpmnGatewayKind, BpmnNodeKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawPackageDocument {
    pub(crate) source_id: String,
    pub(crate) package_id: String,
    pub(crate) processes: Vec<RawProcess>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawProcess {
    pub(crate) process_id: String,
    pub(crate) scope: RawProcessScope,
    pub(crate) nodes: Vec<RawNode>,
    pub(crate) flows: Vec<RawSequenceFlow>,
    pub(crate) associations: Vec<RawAssociation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RawProcessScope {
    TopLevel,
    NestedShell {
        owner_process_id: String,
        owner_node_id: String,
        kind: NestedShellKind,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NestedShellKind {
    EmbeddedSubProcess,
    Transaction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RawSubProcessKind {
    CallActivity,
    EmbeddedSubProcess,
    Transaction,
}

impl RawProcess {
    pub(super) fn new_top_level(process_id: String) -> Self {
        Self {
            process_id,
            scope: RawProcessScope::TopLevel,
            nodes: Vec::new(),
            flows: Vec::new(),
            associations: Vec::new(),
        }
    }

    pub(super) fn new_nested_shell(
        process_id: String,
        owner_process_id: String,
        owner_node_id: String,
        kind: NestedShellKind,
    ) -> Self {
        Self {
            process_id,
            scope: RawProcessScope::NestedShell {
                owner_process_id,
                owner_node_id,
                kind,
            },
            nodes: Vec::new(),
            flows: Vec::new(),
            associations: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawNode {
    pub(crate) bpmn_id: String,
    pub(crate) kind: BpmnNodeKind,
    pub(crate) gateway_kind: Option<BpmnGatewayKind>,
    pub(crate) decision: Option<DmnDecisionRef>,
    pub(crate) lane: Option<RawLaneMembershipSpec>,
    pub(crate) task_message_ref: Option<String>,
    pub(crate) script_task: Option<RawScriptTaskSpec>,
    pub(crate) human_task_form: Option<RawHumanTaskFormSpec>,
    pub(crate) native_human_task_io: Option<RawHumanTaskNativeIoSpec>,
    pub(crate) human_task_assignment: Option<RawHumanTaskAssignmentSpec>,
    pub(crate) task_io: Option<RawTaskIoSpec>,
    pub(crate) called_process_ref: Option<String>,
    pub(crate) subprocess_kind: Option<RawSubProcessKind>,
    pub(crate) repeat: Option<RawRepeatSpec>,
    pub(crate) attached_to_ref: Option<String>,
    pub(crate) default_flow_ref: Option<String>,
    pub(crate) cancel_activity: bool,
    pub(crate) is_for_compensation: bool,
    pub(crate) event: Option<RawEventSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawLaneMembershipSpec {
    pub(crate) set_id: Option<String>,
    pub(crate) set_name: Option<String>,
    pub(crate) id: Option<String>,
    pub(crate) name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawScriptTaskSpec {
    pub(crate) script_format: Option<String>,
    pub(crate) script_body: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawHumanTaskFormSpec {
    pub(crate) interaction_type: String,
    pub(crate) question_ref: Option<String>,
    pub(crate) question_text: Option<String>,
    pub(crate) choices_ref: Option<String>,
    pub(crate) choices: Vec<RawHumanTaskChoiceSpec>,
    pub(crate) free_text_fields: Vec<RawHumanTaskFreeTextSpec>,
    pub(crate) result_output: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawHumanTaskChoiceSpec {
    pub(crate) value: String,
    pub(crate) label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawHumanTaskFreeTextSpec {
    pub(crate) name: String,
    pub(crate) optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct RawTaskIoSpec {
    pub(crate) declarations: Vec<RawTaskIoDeclaration>,
    pub(crate) inputs: Vec<RawTaskInputBinding>,
    pub(crate) outputs: Vec<RawTaskOutputBinding>,
    pub(crate) active_association: Option<RawTaskIoAssociation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawTaskIoDeclaration {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) kind: RawTaskIoDeclarationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RawTaskIoDeclarationKind {
    DataInput,
    DataOutput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawTaskInputBinding {
    pub(crate) name: String,
    pub(crate) source: RawTaskInputSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RawTaskInputSource {
    Variable { source_ref: String },
    Literal { value: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawTaskOutputBinding {
    pub(crate) name: String,
    pub(crate) target_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawTaskIoAssociation {
    pub(crate) kind: RawTaskIoAssociationKind,
    pub(crate) source_refs: Vec<String>,
    pub(crate) target_ref: Option<String>,
    pub(crate) assignment_from: Option<String>,
    pub(crate) assignment_to: Option<String>,
}

impl RawTaskIoAssociation {
    pub(crate) fn new(kind: RawTaskIoAssociationKind) -> Self {
        Self {
            kind,
            source_refs: Vec::new(),
            target_ref: None,
            assignment_from: None,
            assignment_to: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RawTaskIoAssociationKind {
    DataInput,
    DataOutput,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct RawHumanTaskNativeIoSpec {
    pub(crate) documentation_text: Option<String>,
    pub(crate) declarations: Vec<RawHumanTaskIoDeclaration>,
    pub(crate) interaction_type: Option<String>,
    pub(crate) question_ref: Option<String>,
    pub(crate) question_text: Option<String>,
    pub(crate) choices_ref: Option<String>,
    pub(crate) choices: Vec<RawHumanTaskChoiceSpec>,
    pub(crate) free_text_fields: Vec<RawHumanTaskFreeTextSpec>,
    pub(crate) result_output: Option<String>,
    pub(crate) active_association: Option<RawHumanTaskIoAssociation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawHumanTaskIoDeclaration {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) kind: RawHumanTaskIoDeclarationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RawHumanTaskIoDeclarationKind {
    DataInput,
    DataOutput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawHumanTaskIoAssociation {
    pub(crate) kind: RawHumanTaskIoAssociationKind,
    pub(crate) source_refs: Vec<String>,
    pub(crate) target_ref: Option<String>,
    pub(crate) assignment_from: Option<String>,
    pub(crate) assignment_to: Option<String>,
}

impl RawHumanTaskIoAssociation {
    pub(crate) fn new(kind: RawHumanTaskIoAssociationKind) -> Self {
        Self {
            kind,
            source_refs: Vec::new(),
            target_ref: None,
            assignment_from: None,
            assignment_to: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RawHumanTaskIoAssociationKind {
    DataInput,
    DataOutput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawHumanTaskAssignmentSpec {
    pub(crate) human_performers: Vec<RawHumanTaskResourceRoleSpec>,
    pub(crate) potential_owners: Vec<RawHumanTaskResourceRoleSpec>,
    pub(crate) last_role_kind: Option<RawHumanTaskResourceRoleKind>,
}

impl RawHumanTaskAssignmentSpec {
    pub(crate) fn new() -> Self {
        Self {
            human_performers: Vec::new(),
            potential_owners: Vec::new(),
            last_role_kind: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RawHumanTaskResourceRoleKind {
    HumanPerformer,
    PotentialOwner,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawHumanTaskResourceRoleSpec {
    pub(crate) name: Option<String>,
    pub(crate) resource_ref: Option<String>,
    pub(crate) assignment_expression: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RawRepeatSpec {
    StandardLoop(RawStandardLoopSpec),
    SequentialMultiInstance(RawSequentialMultiInstanceSpec),
    ParallelMultiInstance(RawParallelMultiInstanceSpec),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawStandardLoopSpec {
    pub(crate) test_before: bool,
    pub(crate) loop_maximum: Option<u32>,
    pub(crate) loop_condition: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawSequentialMultiInstanceSpec {
    pub(crate) loop_cardinality: Option<u32>,
    pub(crate) loop_data_input_ref: Option<String>,
    pub(crate) input_data_item: Option<String>,
    pub(crate) loop_data_output_ref: Option<String>,
    pub(crate) output_data_item: Option<String>,
    pub(crate) completion_condition: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawParallelMultiInstanceSpec {
    pub(crate) loop_cardinality: Option<u32>,
    pub(crate) loop_data_input_ref: Option<String>,
    pub(crate) input_data_item: Option<String>,
    pub(crate) loop_data_output_ref: Option<String>,
    pub(crate) output_data_item: Option<String>,
    pub(crate) completion_condition: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawEventSpec {
    pub(crate) kind: BpmnEventKind,
    pub(crate) reference_id: Option<String>,
    pub(crate) wait_for_completion: bool,
    pub(crate) name: Option<String>,
    pub(crate) timer: Option<RawTimerSpec>,
    pub(crate) condition_expression: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawTimerSpec {
    pub(crate) kind: BpmnTimerKind,
    pub(crate) expression: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CaptureTarget {
    TimerExpression(BpmnTimerKind),
    ConditionalExpression,
    StandardLoopCondition,
    MultiInstanceLoopCardinality,
    MultiInstanceLoopDataInputRef,
    MultiInstanceLoopDataOutputRef,
    MultiInstanceCompletionCondition,
    SequenceFlowConditionExpression,
    TaskScriptBody,
    HumanTaskDocumentationText,
    HumanTaskIoSourceRef,
    HumanTaskIoTargetRef,
    HumanTaskIoAssignmentFrom,
    HumanTaskIoAssignmentTo,
    TaskIoSourceRef,
    TaskIoTargetRef,
    TaskIoAssignmentFrom,
    TaskIoAssignmentTo,
    HumanTaskResourceRef(RawHumanTaskResourceRoleKind),
    HumanTaskAssignmentExpression(RawHumanTaskResourceRoleKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ProcessChildStartOutcome {
    NotHandled,
    Handled,
    OpenedNestedShell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawSequenceFlow {
    pub(crate) flow_id: String,
    pub(crate) source_ref: String,
    pub(crate) target_ref: String,
    pub(crate) label: Option<String>,
    pub(crate) condition_expression: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawAssociation {
    pub(crate) association_id: String,
    pub(crate) source_ref: String,
    pub(crate) target_ref: String,
}
