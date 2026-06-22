use crate::ir_event_api::{BpmnEventKind, BpmnTimerKind};

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
