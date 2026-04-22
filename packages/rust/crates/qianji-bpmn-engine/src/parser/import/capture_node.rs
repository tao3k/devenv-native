use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::{BpmnEngineError, Result};
use crate::ir_event_api::BpmnTimerKind;
use crate::parser::import::model::RawTimerSpec;
use crate::parser::import::{RawNode, RawProcess, RawRepeatSpec};

pub(in crate::parser::import) fn apply_timer_expression(
    process: &mut RawProcess,
    kind: BpmnTimerKind,
    expression: &str,
) -> Result<()> {
    let node = process
        .nodes
        .last_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "bpmn_timer_expression_without_node",
        })?;
    let event = node
        .event
        .as_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "bpmn_timer_expression_without_event_definition",
        })?;
    event.timer = Some(RawTimerSpec {
        kind,
        expression: expression.to_string(),
    });
    Ok(())
}

pub(in crate::parser::import) fn last_process_node_mut<'a>(
    source: &BpmnSourceFile,
    process: &'a mut RawProcess,
) -> Result<&'a mut RawNode> {
    process
        .nodes
        .last_mut()
        .ok_or(BpmnEngineError::UnsupportedElement {
            source_id: source.source_id.clone(),
            process_id: process.process_id.clone(),
            element: "event_definition_without_node".to_string(),
        })
}

pub(in crate::parser::import) fn apply_standard_loop_condition(
    process: &mut RawProcess,
    loop_condition: &str,
) -> Result<()> {
    let node = process
        .nodes
        .last_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "apply_standard_loop_condition_missing_node",
        })?;
    let Some(RawRepeatSpec::StandardLoop(loop_spec)) = node.repeat.as_mut() else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "apply_standard_loop_condition_missing_repeat_spec",
        });
    };
    loop_spec.loop_condition =
        (!loop_condition.trim().is_empty()).then(|| loop_condition.to_string());
    Ok(())
}

pub(in crate::parser::import) fn apply_sequence_flow_condition_expression(
    process: &mut RawProcess,
    condition_expression: &str,
) -> Result<()> {
    let flow = process
        .flows
        .last_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "apply_sequence_flow_condition_expression_missing_flow",
        })?;
    flow.condition_expression =
        (!condition_expression.trim().is_empty()).then(|| condition_expression.trim().to_string());
    Ok(())
}

pub(in crate::parser::import) fn apply_script_task_body(
    process: &mut RawProcess,
    script_body: &str,
) -> Result<()> {
    let node = process
        .nodes
        .last_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "apply_script_task_body_missing_node",
        })?;
    let script_task = node
        .script_task
        .as_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "apply_script_task_body_missing_script_task_spec",
        })?;
    script_task.script_body =
        (!script_body.trim().is_empty()).then(|| script_body.trim().to_string());
    Ok(())
}
