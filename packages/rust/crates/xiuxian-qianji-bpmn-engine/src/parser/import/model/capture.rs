use super::human_task::RawHumanTaskResourceRoleKind;
use crate::ir_event_api::BpmnTimerKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::parser::import) enum CaptureTarget {
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
pub(in crate::parser::import) enum ProcessChildStartOutcome {
    NotHandled,
    Handled,
    OpenedNestedShell,
}
