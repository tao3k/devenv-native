use std::collections::{BTreeSet, HashMap};
use std::ops::Range;

#[derive(Clone, Default)]
pub(super) struct ProcessMetadata {
    pub(super) task_inputs: HashMap<String, BTreeSet<String>>,
    pub(super) task_outputs: HashMap<String, BTreeSet<String>>,
    pub(super) task_input_spans: HashMap<String, Range<usize>>,
    pub(super) task_output_spans: HashMap<String, Range<usize>>,
    pub(super) gateway_default_flows: HashMap<String, String>,
    pub(super) sequence_flows: HashMap<String, SequenceFlowMetadata>,
    pub(super) node_spans: HashMap<String, Range<usize>>,
}

#[derive(Clone)]
pub(super) struct SequenceFlowMetadata {
    pub(super) target_ref: String,
    pub(super) span: Range<usize>,
}

#[derive(Clone, serde::Serialize)]
pub(super) struct DefaultReentryFlow {
    #[serde(rename = "gateway_id")]
    pub(super) gateway_node: String,
    #[serde(rename = "flow_id")]
    pub(super) sequence_flow: String,
    #[serde(rename = "target_id")]
    pub(super) target_node: String,
    #[serde(rename = "suggested_exit_target_id")]
    pub(super) suggested_exit_target: Option<String>,
}

#[derive(Default)]
pub(super) struct ActiveTask {
    pub(super) id: String,
    pub(super) inputs: BTreeSet<String>,
    pub(super) outputs: BTreeSet<String>,
    pub(super) association_context: Option<TaskAssociationContext>,
    pub(super) association_capture: Option<TaskAssociationCapture>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TaskAssociationContext {
    Input,
    Output,
}

#[derive(Clone, Copy)]
pub(super) enum TaskAssociationCapture {
    InputSourceRef,
    OutputTargetRef,
}

pub(super) struct LoopRiskEvidence {
    pub(super) task_node_ids: Vec<String>,
    pub(super) gateway_ids: Vec<String>,
    pub(super) route_variables: BTreeSet<String>,
    pub(super) updated_variables: BTreeSet<String>,
    pub(super) user_outputs: BTreeSet<String>,
    pub(super) worker_inputs: BTreeSet<String>,
    pub(super) missing_progress_outputs: BTreeSet<String>,
    pub(super) missing_feedback_inputs: BTreeSet<String>,
    pub(super) default_reentry_flows: Vec<DefaultReentryFlow>,
    pub(super) has_exit_path: bool,
    pub(super) has_conditionless_gateway_cycle: bool,
}
