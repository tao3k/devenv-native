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
    pub(crate) task_message_ref: Option<String>,
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
    pub(crate) name: Option<String>,
    pub(crate) timer: Option<RawTimerSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawTimerSpec {
    pub(crate) kind: BpmnTimerKind,
    pub(crate) expression: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CaptureTarget {
    TimerExpression(BpmnTimerKind),
    StandardLoopCondition,
    MultiInstanceLoopCardinality,
    MultiInstanceLoopDataInputRef,
    MultiInstanceLoopDataOutputRef,
    MultiInstanceCompletionCondition,
    SequenceFlowConditionExpression,
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
