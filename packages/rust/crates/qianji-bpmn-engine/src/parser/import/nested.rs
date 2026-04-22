use super::attributes::{
    attribute_value, boolean_attribute_value, event_reference_id, parse_optional_u32_attribute,
};
use super::capture::{
    apply_multi_instance_completion_condition, apply_multi_instance_input_data_item,
    apply_multi_instance_loop_cardinality, apply_multi_instance_loop_data_input_ref,
    apply_multi_instance_loop_data_output_ref, apply_multi_instance_output_data_item,
    apply_sequence_flow_condition_expression, apply_standard_loop_condition,
    apply_timer_expression, last_process_node_mut,
};
use super::model::{
    CaptureTarget, NestedShellKind, RawEventSpec, RawParallelMultiInstanceSpec, RawProcess,
    RawProcessScope, RawRepeatSpec, RawSequentialMultiInstanceSpec, RawStandardLoopSpec,
};
use super::process::is_supported_node_tag;
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
    if handle_supported_node_child_start(source, reader, event, tag, parent, process)? {
        return Ok(());
    }
    if parent == "sequenceFlow" {
        if tag == "conditionExpression" {
            *capture_target = Some(CaptureTarget::SequenceFlowConditionExpression);
            capture_buffer.clear();
            if is_empty {
                apply_sequence_flow_condition_expression(process, "")?;
                *capture_target = None;
                capture_buffer.clear();
            }
            return Ok(());
        }
        if is_ignored_flow_child(tag) {
            return Ok(());
        }
        return Err(BpmnEngineError::UnsupportedElement {
            source_id: source.source_id.clone(),
            process_id: process.process_id.clone(),
            element: tag.to_string(),
        });
    }
    Ok(())
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
        "intermediateCatchEvent" | "boundaryEvent" | "sendTask" | "receiveTask"
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
    let Some(_) = event_reference_id(reader, event, tag)? else {
        return Err(BpmnEngineError::UnsupportedCompensationConfiguration {
            process_id,
            node_id: node.bpmn_id.clone(),
            detail: "default_compensation_intermediate_event",
        });
    };
    if boolean_attribute_value(reader, event, "waitForCompletion")? == Some(false) {
        return Err(BpmnEngineError::UnsupportedCompensationConfiguration {
            process_id,
            node_id: node.bpmn_id.clone(),
            detail: "async_throw_compensation_intermediate_event",
        });
    }
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
    )
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
    if boolean_attribute_value(reader, event, "waitForCompletion")? == Some(false) {
        return Err(BpmnEngineError::UnsupportedCompensationConfiguration {
            process_id,
            node_id: node.bpmn_id.clone(),
            detail: "async_throw_compensation_end_event",
        });
    }
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
    )
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
    if is_supported_node_tag(parent) || parent == "timerEventDefinition" {
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
        name: attribute_value(reader, event, "name")?,
        timer: None,
    });
    Ok(())
}

fn supported_event_definition(parent: &str, tag: &str) -> Option<BpmnEventKind> {
    match (parent, tag) {
        (
            "intermediateCatchEvent" | "boundaryEvent" | "sendTask" | "receiveTask",
            "messageEventDefinition",
        ) => Some(BpmnEventKind::Message),
        (
            "intermediateCatchEvent" | "boundaryEvent" | "sendTask" | "receiveTask",
            "signalEventDefinition",
        ) => Some(BpmnEventKind::Signal),
        ("boundaryEvent" | "endEvent", "errorEventDefinition") => Some(BpmnEventKind::Error),
        ("boundaryEvent" | "endEvent", "cancelEventDefinition") => Some(BpmnEventKind::Cancel),
        ("boundaryEvent", "compensateEventDefinition") => Some(BpmnEventKind::Compensation),
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
