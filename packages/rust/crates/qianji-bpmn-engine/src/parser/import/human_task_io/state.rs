use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::{BpmnEngineError, Result};
use crate::ir_node_api::BpmnNodeKind;
use crate::parser::import::capture::last_process_node_mut;
use crate::parser::import::model::{
    RawHumanTaskIoAssociation, RawHumanTaskNativeIoSpec, RawProcess,
};

pub(super) fn active_association_mut<'a>(
    source: &BpmnSourceFile,
    process: &'a mut RawProcess,
) -> Result<&'a mut RawHumanTaskIoAssociation> {
    native_io_mut(source, process)?
        .active_association
        .as_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "native_human_task_io_association_child_without_association",
        })
}

pub(super) fn ensure_native_io<'a>(
    source: &BpmnSourceFile,
    process: &'a mut RawProcess,
) -> Result<&'a mut RawHumanTaskNativeIoSpec> {
    let node = last_process_node_mut(source, process)?;
    if !matches!(node.kind, BpmnNodeKind::UserTask | BpmnNodeKind::ManualTask) {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "native_human_task_io_without_human_task",
        });
    }
    if node.native_human_task_io.is_none() {
        node.native_human_task_io = Some(RawHumanTaskNativeIoSpec::default());
    }
    node.native_human_task_io
        .as_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "native_human_task_io_missing_state",
        })
}

pub(super) fn native_io_mut<'a>(
    source: &BpmnSourceFile,
    process: &'a mut RawProcess,
) -> Result<&'a mut RawHumanTaskNativeIoSpec> {
    ensure_native_io(source, process)
}

pub(super) fn last_node_is_human_task(process: &RawProcess) -> bool {
    process
        .nodes
        .last()
        .is_some_and(|node| matches!(node.kind, BpmnNodeKind::UserTask | BpmnNodeKind::ManualTask))
}

pub(super) fn is_human_task(tag: &str) -> bool {
    matches!(tag, "userTask" | "manualTask")
}

pub(super) fn is_supported_task(tag: &str) -> bool {
    matches!(
        tag,
        "serviceTask"
            | "userTask"
            | "manualTask"
            | "businessRuleTask"
            | "scriptTask"
            | "sendTask"
            | "receiveTask"
            | "task"
    )
}
