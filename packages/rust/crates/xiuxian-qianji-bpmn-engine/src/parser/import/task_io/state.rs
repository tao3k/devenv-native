use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::{BpmnEngineError, Result};
use crate::ir_node_api::BpmnNodeKind;
use crate::parser::import::capture::last_process_node_mut;
use crate::parser::import::model::{RawProcess, RawTaskIoSpec};

pub(super) fn task_io_mut<'a>(
    source: &BpmnSourceFile,
    process: &'a mut RawProcess,
) -> Result<&'a mut RawTaskIoSpec> {
    let node = last_process_node_mut(source, process)?;
    if !is_task_io_owner(&node.kind) {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "task_io_without_supported_task",
        });
    }
    if node.task_io.is_none() {
        node.task_io = Some(RawTaskIoSpec::default());
    }
    node.task_io
        .as_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "task_io_missing_state",
        })
}

pub(super) fn last_node_is_task_io_owner(process: &RawProcess) -> bool {
    process
        .nodes
        .last()
        .is_some_and(|node| is_task_io_owner(&node.kind))
}

pub(super) fn record_task_property_id(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    property_id: String,
) -> Result<()> {
    let io = task_io_mut(source, process)?;
    if !io.property_ids.iter().any(|id| id == &property_id) {
        io.property_ids.push(property_id);
    }
    Ok(())
}

fn is_task_io_owner(kind: &BpmnNodeKind) -> bool {
    matches!(
        kind,
        BpmnNodeKind::Task
            | BpmnNodeKind::SendTask
            | BpmnNodeKind::ReceiveTask
            | BpmnNodeKind::ServiceTask
            | BpmnNodeKind::ScriptTask
            | BpmnNodeKind::BusinessRuleTask
    )
}
