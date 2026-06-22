use super::event::RawEventSpec;
use super::human_task::{
    RawHumanTaskAssignmentSpec, RawHumanTaskFormSpec, RawHumanTaskNativeIoSpec,
};
use super::lane::RawLaneMembershipSpec;
use super::repeat::RawRepeatSpec;
use super::script::RawScriptTaskSpec;
use super::task_io::RawTaskIoSpec;
use crate::dmn_model_api::DmnDecisionRef;
use crate::ir_node_api::{BpmnGatewayKind, BpmnNodeKind};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RawSubProcessKind {
    CallActivity,
    EmbeddedSubProcess,
    Transaction,
    EventSubProcess,
}
