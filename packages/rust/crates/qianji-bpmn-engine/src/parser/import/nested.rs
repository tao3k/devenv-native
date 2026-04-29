use super::attributes::{
    attribute_value, boolean_attribute_value, event_reference_id, parse_optional_u32_attribute,
};
use super::capture::{
    apply_conditional_expression, apply_human_task_assignment_expression,
    apply_human_task_resource_ref, apply_multi_instance_completion_condition,
    apply_multi_instance_input_data_item, apply_multi_instance_loop_cardinality,
    apply_multi_instance_loop_data_input_ref, apply_multi_instance_loop_data_output_ref,
    apply_multi_instance_output_data_item, apply_script_task_body,
    apply_sequence_flow_condition_expression, apply_standard_loop_condition,
    apply_timer_expression, last_process_node_mut, push_human_task_resource_role,
};
use super::human_task_io::handle_human_task_io_child_start;
use super::model::{
    CaptureTarget, NestedShellKind, RawEventSpec, RawHumanTaskResourceRoleKind,
    RawParallelMultiInstanceSpec, RawProcess, RawProcessScope, RawRepeatSpec,
    RawSequentialMultiInstanceSpec, RawStandardLoopSpec,
};
use super::process::is_supported_node_tag;
use super::task_io::handle_task_io_child_start;
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::{BpmnEngineError, Result};
use crate::ir_event_api::{BpmnEventKind, BpmnTimerKind};
use quick_xml::Reader;
use quick_xml::events::BytesStart;

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_nested_start_tag(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    parent: &str,
    process: &mut RawProcess,
    capture_target: &mut Option<CaptureTarget>,
    capture_buffer: &mut String,
    is_empty: bool,
) -> Result<()> {
    if handle_intermediate_throw_event_child_start(source, reader, event, tag, parent, process)? {
        return Ok(());
    }
    if handle_multi_instance_data_item_start(source, reader, event, process, tag, parent)? {
        return Ok(());
    }
    if handle_loop_child_start(
        source,
        process,
        tag,
        parent,
        capture_target,
        capture_buffer,
        is_empty,
    )? {
        return Ok(());
    }
    if handle_event_child_start(
        source,
        reader,
        event,
        tag,
        parent,
        process,
        capture_target,
        capture_buffer,
        is_empty,
    )? {
        return Ok(());
    }
    if handle_script_task_child_start(
        source,
        tag,
        parent,
        process,
        capture_target,
        capture_buffer,
        is_empty,
    )? {
        return Ok(());
    }
    if handle_task_io_child_start(
        source,
        reader,
        event,
        tag,
        parent,
        process,
        capture_target,
        capture_buffer,
        is_empty,
    )? {
        return Ok(());
    }
    if handle_human_task_io_child_start(
        source,
        reader,
        event,
        tag,
        parent,
        process,
        capture_target,
        capture_buffer,
        is_empty,
    )? {
        return Ok(());
    }
    if handle_human_task_assignment_child_start(
        source,
        reader,
        event,
        tag,
        parent,
        process,
        capture_target,
        capture_buffer,
        is_empty,
    )? {
        return Ok(());
    }
    if handle_supported_node_child_start(source, reader, event, tag, parent, process)? {
        return Ok(());
    }
    if handle_sequence_flow_child_start(
        source,
        process,
        tag,
        parent,
        capture_target,
        capture_buffer,
        is_empty,
    )? {
        return Ok(());
    }
    Ok(())
}

fn handle_sequence_flow_child_start(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    tag: &str,
    parent: &str,
    capture_target: &mut Option<CaptureTarget>,
    capture_buffer: &mut String,
    is_empty: bool,
) -> Result<bool> {
    if parent != "sequenceFlow" {
        return Ok(false);
    }
    if tag == "conditionExpression" {
        *capture_target = Some(CaptureTarget::SequenceFlowConditionExpression);
        capture_buffer.clear();
        if is_empty {
            apply_sequence_flow_condition_expression(process, "")?;
            *capture_target = None;
            capture_buffer.clear();
        }
        return Ok(true);
    }
    if is_ignored_flow_child(tag) {
        return Ok(true);
    }
    Err(BpmnEngineError::UnsupportedElement {
        source_id: source.source_id.clone(),
        process_id: process.process_id.clone(),
        element: tag.to_string(),
    })
}

#[allow(clippy::too_many_arguments)]
fn handle_human_task_assignment_child_start(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    parent: &str,
    process: &mut RawProcess,
    capture_target: &mut Option<CaptureTarget>,
    capture_buffer: &mut String,
    is_empty: bool,
) -> Result<bool> {
    if matches!(parent, "userTask" | "manualTask") && is_human_task_assignment_role(tag) {
        let node = last_process_node_mut(source, process)?;
        if !matches!(
            node.kind,
            crate::ir_node_api::BpmnNodeKind::UserTask
                | crate::ir_node_api::BpmnNodeKind::ManualTask
        ) {
            return Ok(true);
        }
        let kind = human_task_resource_role_kind(tag)?;
        push_human_task_resource_role(process, kind, attribute_value(reader, event, "name")?)?;
        return Ok(true);
    }

    let Some(kind) = human_task_resource_role_kind(parent).ok() else {
        if parent == "resourceAssignmentExpression" && tag == "formalExpression" {
            let kind = last_human_task_resource_role_kind(source, process)?;
            *capture_target = Some(CaptureTarget::HumanTaskAssignmentExpression(kind));
            capture_buffer.clear();
            if is_empty {
                apply_human_task_assignment_expression(process, kind, "")?;
                *capture_target = None;
                capture_buffer.clear();
            }
            return Ok(true);
        }
        return Ok(false);
    };

    match tag {
        "resourceRef" => {
            *capture_target = Some(CaptureTarget::HumanTaskResourceRef(kind));
            capture_buffer.clear();
            if is_empty {
                apply_human_task_resource_ref(process, kind, "")?;
                *capture_target = None;
                capture_buffer.clear();
            }
            Ok(true)
        }
        _ => Ok(true),
    }
}

fn is_human_task_assignment_role(tag: &str) -> bool {
    matches!(tag, "humanPerformer" | "potentialOwner")
}

fn human_task_resource_role_kind(tag: &str) -> Result<RawHumanTaskResourceRoleKind> {
    match tag {
        "humanPerformer" => Ok(RawHumanTaskResourceRoleKind::HumanPerformer),
        "potentialOwner" => Ok(RawHumanTaskResourceRoleKind::PotentialOwner),
        _ => Err(BpmnEngineError::UnsupportedOperation {
            operation: "unknown_human_task_resource_role_kind",
        }),
    }
}

fn last_human_task_resource_role_kind(
    source: &BpmnSourceFile,
    process: &RawProcess,
) -> Result<RawHumanTaskResourceRoleKind> {
    let node = process
        .nodes
        .last()
        .ok_or(BpmnEngineError::UnsupportedElement {
            source_id: source.source_id.clone(),
            process_id: process.process_id.clone(),
            element: "resourceAssignmentExpression".to_string(),
        })?;
    let assignment =
        node.human_task_assignment
            .as_ref()
            .ok_or(BpmnEngineError::UnsupportedOperation {
                operation: "human_task_assignment_expression_without_role",
            })?;
    assignment
        .last_role_kind
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "human_task_assignment_expression_without_role",
        })
}

fn handle_intermediate_throw_event_child_start(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    parent: &str,
    process: &mut RawProcess,
) -> Result<bool> {
    if parent != "intermediateThrowEvent" {
        return Ok(false);
    }
    if tag == "compensateEventDefinition" {
        handle_compensation_intermediate_event_definition(source, reader, event, process, tag)?;
        return Ok(true);
    }
    let _ = source;
    Ok(false)
}

fn handle_loop_child_start(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    tag: &str,
    parent: &str,
    capture_target: &mut Option<CaptureTarget>,
    capture_buffer: &mut String,
    is_empty: bool,
) -> Result<bool> {
    match parent {
        "standardLoopCharacteristics" => {
            if tag == "loopCondition" {
                *capture_target = Some(CaptureTarget::StandardLoopCondition);
                capture_buffer.clear();
                if is_empty {
                    apply_standard_loop_condition(process, "")?;
                    *capture_target = None;
                    capture_buffer.clear();
                }
                return Ok(true);
            }
            if is_ignored_loop_child(tag) {
                return Ok(true);
            }
            let process_id = process.process_id.clone();
            let node = last_process_node_mut(source, process)?;
            Err(BpmnEngineError::UnsupportedLoopConfiguration {
                process_id,
                node_id: node.bpmn_id.clone(),
                detail: "unsupported_standard_loop_child",
            })
        }
        "multiInstanceLoopCharacteristics" => {
            if tag == "loopCardinality" {
                *capture_target = Some(CaptureTarget::MultiInstanceLoopCardinality);
                capture_buffer.clear();
                if is_empty {
                    apply_multi_instance_loop_cardinality(process, "")?;
                    *capture_target = None;
                    capture_buffer.clear();
                }
                return Ok(true);
            }
            if tag == "loopDataInputRef" {
                *capture_target = Some(CaptureTarget::MultiInstanceLoopDataInputRef);
                capture_buffer.clear();
                if is_empty {
                    apply_multi_instance_loop_data_input_ref(process, "")?;
                    *capture_target = None;
                    capture_buffer.clear();
                }
                return Ok(true);
            }
            if tag == "loopDataOutputRef" {
                *capture_target = Some(CaptureTarget::MultiInstanceLoopDataOutputRef);
                capture_buffer.clear();
                if is_empty {
                    apply_multi_instance_loop_data_output_ref(process, "")?;
                    *capture_target = None;
                    capture_buffer.clear();
                }
                return Ok(true);
            }
            if tag == "completionCondition" {
                *capture_target = Some(CaptureTarget::MultiInstanceCompletionCondition);
                capture_buffer.clear();
                if is_empty {
                    apply_multi_instance_completion_condition(process, "")?;
                    *capture_target = None;
                    capture_buffer.clear();
                }
                return Ok(true);
            }
            if is_ignored_loop_child(tag) {
                return Ok(true);
            }
            let process_id = process.process_id.clone();
            let node = last_process_node_mut(source, process)?;
            let detail = "unsupported_multi_instance_child";
            Err(BpmnEngineError::UnsupportedLoopConfiguration {
                process_id,
                node_id: node.bpmn_id.clone(),
                detail,
            })
        }
        _ => Ok(false),
    }
}

fn handle_multi_instance_data_item_start(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    process: &mut RawProcess,
    tag: &str,
    parent: &str,
) -> Result<bool> {
    if parent != "multiInstanceLoopCharacteristics" {
        return Ok(false);
    }
    match tag {
        "inputDataItem" => {
            apply_multi_instance_input_data_item(source, reader, event, process, tag)?;
            Ok(true)
        }
        "outputDataItem" => {
            apply_multi_instance_output_data_item(source, reader, event, process, tag)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_event_child_start(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    parent: &str,
    process: &mut RawProcess,
    capture_target: &mut Option<CaptureTarget>,
    capture_buffer: &mut String,
    is_empty: bool,
) -> Result<bool> {
    if parent == "endEvent" && tag == "compensateEventDefinition" {
        handle_compensation_end_event_definition(source, reader, event, process, tag)?;
        return Ok(true);
    }
    if parent == "intermediateThrowEvent" && tag == "compensateEventDefinition" {
        handle_compensation_intermediate_event_definition(source, reader, event, process, tag)?;
        return Ok(true);
    }
    if let Some(kind) = supported_event_definition(parent, tag) {
        assign_event_definition(source, reader, event, process, kind, tag)?;
        return Ok(true);
    }
    if matches!(
        parent,
        "startEvent" | "intermediateCatchEvent" | "boundaryEvent" | "sendTask" | "receiveTask"
    ) && tag == "timerEventDefinition"
    {
        assign_event_definition(source, reader, event, process, BpmnEventKind::Timer, tag)?;
        return Ok(true);
    }
    if parent == "timerEventDefinition"
        && let Some(timer_kind) = supported_timer_expression(tag)
    {
        *capture_target = Some(CaptureTarget::TimerExpression(timer_kind.clone()));
        capture_buffer.clear();
        if is_empty {
            apply_timer_expression(process, timer_kind, "")?;
            *capture_target = None;
            capture_buffer.clear();
        }
        return Ok(true);
    }
    if parent == "conditionalEventDefinition" && tag == "condition" {
        *capture_target = Some(CaptureTarget::ConditionalExpression);
        capture_buffer.clear();
        if is_empty {
            apply_conditional_expression(process, "")?;
            *capture_target = None;
            capture_buffer.clear();
        }
        return Ok(true);
    }
    Ok(false)
}

fn handle_compensation_intermediate_event_definition(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    process: &mut RawProcess,
    tag: &str,
) -> Result<()> {
    let process_id = process.process_id.clone();
    let inside_transaction_shell = matches!(
        process.scope,
        RawProcessScope::NestedShell {
            kind: NestedShellKind::Transaction,
            ..
        }
    );
    let node = last_process_node_mut(source, process)?;
    let wait_for_completion =
        boolean_attribute_value(reader, event, "waitForCompletion")?.unwrap_or(true);
    if !inside_transaction_shell {
        return Err(BpmnEngineError::UnsupportedCompensationConfiguration {
            process_id,
            node_id: node.bpmn_id.clone(),
            detail: "throw_compensation_intermediate_event",
        });
    }
    assign_event_definition(
        source,
        reader,
        event,
        process,
        BpmnEventKind::Compensation,
        tag,
    )?;
    let node = last_process_node_mut(source, process)?;
    if let Some(event) = node.event.as_mut() {
        event.wait_for_completion = wait_for_completion;
    }
    Ok(())
}

fn handle_compensation_end_event_definition(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    process: &mut RawProcess,
    tag: &str,
) -> Result<()> {
    let process_id = process.process_id.clone();
    let inside_transaction_shell = matches!(
        process.scope,
        RawProcessScope::NestedShell {
            kind: NestedShellKind::Transaction,
            ..
        }
    );
    let node = last_process_node_mut(source, process)?;
    let wait_for_completion =
        boolean_attribute_value(reader, event, "waitForCompletion")?.unwrap_or(true);
    if !inside_transaction_shell {
        return Err(BpmnEngineError::UnsupportedCompensationConfiguration {
            process_id,
            node_id: node.bpmn_id.clone(),
            detail: "throw_compensation_end_event",
        });
    }
    assign_event_definition(
        source,
        reader,
        event,
        process,
        BpmnEventKind::Compensation,
        tag,
    )?;
    let node = last_process_node_mut(source, process)?;
    if let Some(event) = node.event.as_mut() {
        event.wait_for_completion = wait_for_completion;
    }
    Ok(())
}

fn handle_script_task_child_start(
    source: &BpmnSourceFile,
    tag: &str,
    parent: &str,
    process: &mut RawProcess,
    capture_target: &mut Option<CaptureTarget>,
    capture_buffer: &mut String,
    is_empty: bool,
) -> Result<bool> {
    if parent != "scriptTask" || tag != "script" {
        return Ok(false);
    }
    let process_id = process.process_id.clone();
    let node = last_process_node_mut(source, process)?;
    let Some(script_task) = node.script_task.as_ref() else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "handle_script_task_child_missing_script_task_spec",
        });
    };
    if script_task.script_body.is_some() {
        return Err(BpmnEngineError::UnsupportedTaskConfiguration {
            process_id,
            node_id: node.bpmn_id.clone(),
            detail: "multiple_script_task_bodies",
        });
    }
    *capture_target = Some(CaptureTarget::TaskScriptBody);
    capture_buffer.clear();
    if is_empty {
        apply_script_task_body(process, "")?;
        *capture_target = None;
        capture_buffer.clear();
    }
    Ok(true)
}

fn handle_supported_node_child_start(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    parent: &str,
    process: &mut RawProcess,
) -> Result<bool> {
    if is_supported_node_tag(parent) && tag == "standardLoopCharacteristics" {
        let process_id = process.process_id.clone();
        let node = last_process_node_mut(source, process)?;
        if node.repeat.is_some() {
            return Err(BpmnEngineError::UnsupportedLoopConfiguration {
                process_id,
                node_id: node.bpmn_id.clone(),
                detail: "multiple_loop_characteristics",
            });
        }
        node.repeat = Some(RawRepeatSpec::StandardLoop(RawStandardLoopSpec {
            test_before: boolean_attribute_value(reader, event, "testBefore")?.unwrap_or(false),
            loop_maximum: parse_optional_u32_attribute(
                reader,
                event,
                "loopMaximum",
                &process_id,
                &node.bpmn_id,
                "invalid_loop_maximum",
            )?,
            loop_condition: None,
        }));
        return Ok(true);
    }
    if is_supported_node_tag(parent) && tag == "multiInstanceLoopCharacteristics" {
        let process_id = process.process_id.clone();
        let node = last_process_node_mut(source, process)?;
        if node.repeat.is_some() {
            return Err(BpmnEngineError::UnsupportedLoopConfiguration {
                process_id,
                node_id: node.bpmn_id.clone(),
                detail: "multiple_loop_characteristics",
            });
        }
        node.repeat = Some(
            if boolean_attribute_value(reader, event, "isSequential")?.unwrap_or(false) {
                RawRepeatSpec::SequentialMultiInstance(RawSequentialMultiInstanceSpec {
                    loop_cardinality: None,
                    loop_data_input_ref: None,
                    input_data_item: None,
                    loop_data_output_ref: None,
                    output_data_item: None,
                    completion_condition: None,
                })
            } else {
                RawRepeatSpec::ParallelMultiInstance(RawParallelMultiInstanceSpec {
                    loop_cardinality: None,
                    loop_data_input_ref: None,
                    input_data_item: None,
                    loop_data_output_ref: None,
                    output_data_item: None,
                    completion_condition: None,
                })
            },
        );
        return Ok(true);
    }
    if is_supported_node_tag(parent)
        || matches!(
            parent,
            "timerEventDefinition" | "conditionalEventDefinition" | "escalationEventDefinition"
        )
    {
        if is_ignored_node_child(tag) {
            return Ok(true);
        }
        return Err(BpmnEngineError::UnsupportedElement {
            source_id: source.source_id.clone(),
            process_id: process.process_id.clone(),
            element: tag.to_string(),
        });
    }
    Ok(false)
}

fn assign_event_definition(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    process: &mut RawProcess,
    kind: BpmnEventKind,
    tag: &str,
) -> Result<()> {
    let process_id = process.process_id.clone();
    let node = last_process_node_mut(source, process)?;
    if node.event.is_some() {
        return Err(BpmnEngineError::UnsupportedMultipleEventDefinitions {
            source_id: source.source_id.clone(),
            process_id,
            node_id: node.bpmn_id.clone(),
        });
    }
    let reference_id = if kind == BpmnEventKind::Timer {
        None
    } else {
        event_reference_id(reader, event, tag)?
    };
    node.event = Some(RawEventSpec {
        kind,
        reference_id,
        wait_for_completion: true,
        name: attribute_value(reader, event, "name")?,
        timer: None,
        condition_expression: None,
    });
    Ok(())
}

fn supported_event_definition(parent: &str, tag: &str) -> Option<BpmnEventKind> {
    match (parent, tag) {
        (
            "startEvent" | "intermediateCatchEvent" | "boundaryEvent" | "sendTask" | "receiveTask",
            "messageEventDefinition",
        ) => Some(BpmnEventKind::Message),
        (
            "startEvent" | "intermediateCatchEvent" | "boundaryEvent" | "sendTask" | "receiveTask",
            "signalEventDefinition",
        ) => Some(BpmnEventKind::Signal),
        ("boundaryEvent" | "endEvent", "errorEventDefinition") => Some(BpmnEventKind::Error),
        ("boundaryEvent" | "endEvent" | "intermediateThrowEvent", "escalationEventDefinition") => {
            Some(BpmnEventKind::Escalation)
        }
        ("boundaryEvent" | "endEvent", "cancelEventDefinition") => Some(BpmnEventKind::Cancel),
        ("boundaryEvent", "compensateEventDefinition") => Some(BpmnEventKind::Compensation),
        ("endEvent", "terminateEventDefinition") => Some(BpmnEventKind::Terminate),
        (
            "startEvent" | "intermediateCatchEvent" | "boundaryEvent",
            "conditionalEventDefinition",
        ) => Some(BpmnEventKind::Conditional),
        _ => None,
    }
}

fn supported_timer_expression(tag: &str) -> Option<BpmnTimerKind> {
    match tag {
        "timeDate" => Some(BpmnTimerKind::Date),
        "timeDuration" => Some(BpmnTimerKind::Duration),
        "timeCycle" => Some(BpmnTimerKind::Cycle),
        _ => None,
    }
}

fn is_ignored_node_child(tag: &str) -> bool {
    matches!(
        tag,
        "documentation" | "extensionElements" | "incoming" | "outgoing"
    )
}

fn is_ignored_loop_child(tag: &str) -> bool {
    matches!(tag, "documentation" | "extensionElements")
}

fn is_ignored_flow_child(tag: &str) -> bool {
    matches!(tag, "documentation" | "extensionElements")
}
