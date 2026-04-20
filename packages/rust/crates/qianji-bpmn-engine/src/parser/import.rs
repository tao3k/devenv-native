//! BPMN source ingestion and XML extraction.

use super::package::BpmnSourceFile;
use crate::dmn::DmnDecisionRef;
use crate::error::{BpmnEngineError, Result};
use crate::ir::{BpmnEventKind, BpmnGatewayKind, BpmnNodeKind, BpmnTimerKind};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawPackageDocument {
    pub(crate) source_id: String,
    pub(crate) package_id: String,
    pub(crate) processes: Vec<RawProcess>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawProcess {
    pub(crate) process_id: String,
    pub(crate) nodes: Vec<RawNode>,
    pub(crate) flows: Vec<RawSequenceFlow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawNode {
    pub(crate) bpmn_id: String,
    pub(crate) kind: BpmnNodeKind,
    pub(crate) gateway_kind: Option<BpmnGatewayKind>,
    pub(crate) decision: Option<DmnDecisionRef>,
    pub(crate) called_process_ref: Option<String>,
    pub(crate) repeat: Option<RawRepeatSpec>,
    pub(crate) attached_to_ref: Option<String>,
    pub(crate) cancel_activity: bool,
    pub(crate) event: Option<RawEventSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RawRepeatSpec {
    StandardLoop(RawStandardLoopSpec),
    SequentialMultiInstance(RawSequentialMultiInstanceSpec),
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
enum CaptureTarget {
    TimerExpression(BpmnTimerKind),
    StandardLoopCondition,
    MultiInstanceLoopCardinality,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawSequenceFlow {
    pub(crate) flow_id: String,
    pub(crate) source_ref: String,
    pub(crate) target_ref: String,
    pub(crate) label: Option<String>,
}

pub(crate) fn import_bpmn_source(source: &BpmnSourceFile) -> Result<RawPackageDocument> {
    let mut reader = Reader::from_str(&source.contents);
    reader.config_mut().trim_text(true);

    let mut saw_root = false;
    let mut stack = Vec::new();
    let mut package_id = None;
    let mut processes = Vec::new();
    let mut current_process: Option<RawProcess> = None;
    let mut capture_target = None;
    let mut capture_buffer = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                saw_root = true;
                handle_open_event(
                    source,
                    &reader,
                    &event,
                    &mut stack,
                    &mut package_id,
                    &mut current_process,
                    &mut processes,
                    &mut capture_target,
                    &mut capture_buffer,
                    false,
                )?;
            }
            Ok(Event::Empty(event)) => {
                saw_root = true;
                handle_open_event(
                    source,
                    &reader,
                    &event,
                    &mut stack,
                    &mut package_id,
                    &mut current_process,
                    &mut processes,
                    &mut capture_target,
                    &mut capture_buffer,
                    true,
                )?;
            }
            Ok(Event::Text(event)) => append_capture_text(
                source,
                capture_target.as_ref(),
                &mut capture_buffer,
                event.decode(),
            )?,
            Ok(Event::CData(event)) => append_capture_text(
                source,
                capture_target.as_ref(),
                &mut capture_buffer,
                event.decode(),
            )?,
            Ok(Event::End(event)) => {
                let tag = local_name(event.name().as_ref()).to_string();
                handle_end_tag(
                    source,
                    &tag,
                    &mut current_process,
                    &mut processes,
                    &mut capture_target,
                    &mut capture_buffer,
                )?;
                let _ = stack.pop();
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(error) => {
                return Err(BpmnEngineError::InvalidXml {
                    source_id: source.source_id.clone(),
                    message: error.to_string(),
                });
            }
        }
    }

    if !saw_root {
        return Err(BpmnEngineError::MissingRootElement {
            source_id: source.source_id.clone(),
        });
    }

    Ok(RawPackageDocument {
        source_id: source.source_id.clone(),
        package_id: package_id.unwrap_or_else(|| source.source_id.clone()),
        processes,
    })
}

#[allow(clippy::too_many_arguments)]
fn handle_open_event(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    stack: &mut Vec<String>,
    package_id: &mut Option<String>,
    current_process: &mut Option<RawProcess>,
    processes: &mut Vec<RawProcess>,
    capture_target: &mut Option<CaptureTarget>,
    capture_buffer: &mut String,
    is_empty: bool,
) -> Result<()> {
    let tag = local_name(event.name().as_ref()).to_string();
    let parent = stack.last().map(String::as_str);
    handle_start_tag(
        source,
        reader,
        event,
        &tag,
        parent,
        package_id,
        current_process,
        processes,
        capture_target,
        capture_buffer,
        is_empty,
    )?;
    if !is_empty {
        stack.push(tag);
    }
    Ok(())
}

fn append_capture_text(
    source: &BpmnSourceFile,
    capture_target: Option<&CaptureTarget>,
    capture_buffer: &mut String,
    decoded: std::result::Result<Cow<'_, str>, quick_xml::encoding::EncodingError>,
) -> Result<()> {
    if capture_target.is_none() {
        return Ok(());
    }
    let text = decoded.map_err(|error| BpmnEngineError::InvalidXml {
        source_id: source.source_id.clone(),
        message: error.to_string(),
    })?;
    capture_buffer.push_str(text.as_ref());
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn handle_start_tag(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    parent: Option<&str>,
    package_id: &mut Option<String>,
    current_process: &mut Option<RawProcess>,
    processes: &mut Vec<RawProcess>,
    capture_target: &mut Option<CaptureTarget>,
    capture_buffer: &mut String,
    is_empty: bool,
) -> Result<()> {
    if handle_package_start_tag(
        source,
        reader,
        event,
        tag,
        package_id,
        current_process,
        processes,
        is_empty,
    )? {
        return Ok(());
    }

    let Some(process) = current_process.as_mut() else {
        return Ok(());
    };

    if handle_process_child_start_tag(source, reader, event, tag, parent, process)? {
        return Ok(());
    }

    if let Some(parent) = parent {
        return handle_nested_start_tag(
            source,
            reader,
            event,
            tag,
            parent,
            process,
            capture_target,
            capture_buffer,
            is_empty,
        );
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn handle_package_start_tag(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    package_id: &mut Option<String>,
    current_process: &mut Option<RawProcess>,
    processes: &mut Vec<RawProcess>,
    is_empty: bool,
) -> Result<bool> {
    if tag == "definitions" {
        if package_id.is_none() {
            *package_id = attribute_value(reader, event, "id")?;
        }
        return Ok(true);
    }
    if current_process.is_some() || tag != "process" {
        return Ok(false);
    }

    let process_id = required_attribute(source, reader, event, "process", "id")?;
    *current_process = Some(RawProcess {
        process_id,
        nodes: Vec::new(),
        flows: Vec::new(),
    });
    if is_empty && let Some(process) = current_process.take() {
        processes.push(process);
    }
    Ok(true)
}

fn handle_process_child_start_tag(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    parent: Option<&str>,
    process: &mut RawProcess,
) -> Result<bool> {
    if parent != Some("process") {
        return Ok(false);
    }
    if let Some((kind, gateway_kind)) = supported_node_kind(tag) {
        let bpmn_id = required_attribute(source, reader, event, tag, "id")?;
        let decision = decision_reference(reader, event)?;
        let called_process_ref = if tag == "callActivity" {
            Some(required_attribute(
                source,
                reader,
                event,
                tag,
                "calledElement",
            )?)
        } else {
            None
        };
        let attached_to_ref = if kind == BpmnNodeKind::BoundaryEvent {
            attribute_value(reader, event, "attachedToRef")?
        } else {
            None
        };
        let cancel_activity = if kind == BpmnNodeKind::BoundaryEvent {
            cancel_activity_value(reader, event)?
        } else {
            true
        };
        process.nodes.push(RawNode {
            bpmn_id,
            kind,
            gateway_kind,
            decision,
            called_process_ref,
            repeat: None,
            attached_to_ref,
            cancel_activity,
            event: None,
        });
        return Ok(true);
    }
    if tag == "sequenceFlow" {
        let flow_id = required_attribute(source, reader, event, tag, "id")?;
        let source_ref = required_attribute(source, reader, event, tag, "sourceRef")?;
        let target_ref = required_attribute(source, reader, event, tag, "targetRef")?;
        let label = attribute_value(reader, event, "name")?;
        process.flows.push(RawSequenceFlow {
            flow_id,
            source_ref,
            target_ref,
            label,
        });
        return Ok(true);
    }
    if is_ignored_process_child(tag) {
        return Ok(true);
    }
    Err(BpmnEngineError::UnsupportedElement {
        source_id: source.source_id.clone(),
        process_id: process.process_id.clone(),
        element: tag.to_string(),
    })
}

#[allow(clippy::too_many_arguments)]
fn handle_nested_start_tag(
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
            if is_ignored_loop_child(tag) {
                return Ok(true);
            }
            let process_id = process.process_id.clone();
            let node = last_process_node_mut(source, process)?;
            let detail = match tag {
                "loopDataInputRef" => "unsupported_multi_instance_data_input",
                "loopDataOutputRef" => "unsupported_multi_instance_data_output",
                "inputDataItem" => "unsupported_multi_instance_input_item",
                "outputDataItem" => "unsupported_multi_instance_output_item",
                "completionCondition" => "unsupported_multi_instance_completion_condition",
                _ => "unsupported_multi_instance_child",
            };
            Err(BpmnEngineError::UnsupportedLoopConfiguration {
                process_id,
                node_id: node.bpmn_id.clone(),
                detail,
            })
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
    if matches!(parent, "intermediateCatchEvent" | "boundaryEvent") {
        if let Some(kind) = supported_event_definition(tag) {
            assign_event_definition(source, reader, event, process, kind, tag)?;
            return Ok(true);
        }
        if tag == "timerEventDefinition" {
            assign_event_definition(source, reader, event, process, BpmnEventKind::Timer, tag)?;
            return Ok(true);
        }
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
        if !boolean_attribute_value(reader, event, "isSequential")?.unwrap_or(false) {
            return Err(BpmnEngineError::UnsupportedLoopConfiguration {
                process_id,
                node_id: node.bpmn_id.clone(),
                detail: "parallel_multi_instance_deferred",
            });
        }
        node.repeat = Some(RawRepeatSpec::SequentialMultiInstance(
            RawSequentialMultiInstanceSpec {
                loop_cardinality: None,
            },
        ));
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

fn handle_end_tag(
    source: &BpmnSourceFile,
    tag: &str,
    current_process: &mut Option<RawProcess>,
    processes: &mut Vec<RawProcess>,
    capture_target: &mut Option<CaptureTarget>,
    capture_buffer: &mut String,
) -> Result<()> {
    if tag == "process" {
        if let Some(process) = current_process.take() {
            processes.push(process);
        }
        return Ok(());
    }

    let Some(process) = current_process.as_mut() else {
        return Ok(());
    };

    let Some(target) = capture_target.clone() else {
        return Ok(());
    };

    match (target, tag) {
        (CaptureTarget::TimerExpression(kind), "timeDate") if kind == BpmnTimerKind::Date => {
            apply_timer_expression(process, kind, capture_buffer.trim())?;
        }
        (CaptureTarget::TimerExpression(kind), "timeDuration")
            if kind == BpmnTimerKind::Duration =>
        {
            apply_timer_expression(process, kind, capture_buffer.trim())?;
        }
        (CaptureTarget::TimerExpression(kind), "timeCycle") if kind == BpmnTimerKind::Cycle => {
            apply_timer_expression(process, kind, capture_buffer.trim())?;
        }
        (CaptureTarget::StandardLoopCondition, "loopCondition") => {
            apply_standard_loop_condition(process, capture_buffer.trim())?;
        }
        (CaptureTarget::MultiInstanceLoopCardinality, "loopCardinality") => {
            apply_multi_instance_loop_cardinality(process, capture_buffer.trim())?;
        }
        _ => return Ok(()),
    }

    *capture_target = None;
    capture_buffer.clear();
    let _ = source;
    Ok(())
}

fn supported_node_kind(tag: &str) -> Option<(BpmnNodeKind, Option<BpmnGatewayKind>)> {
    match tag {
        "startEvent" => Some((BpmnNodeKind::StartEvent, None)),
        "endEvent" => Some((BpmnNodeKind::EndEvent, None)),
        "intermediateCatchEvent" => Some((BpmnNodeKind::IntermediateCatchEvent, None)),
        "boundaryEvent" => Some((BpmnNodeKind::BoundaryEvent, None)),
        "callActivity" => Some((BpmnNodeKind::SubProcess, None)),
        "serviceTask" => Some((BpmnNodeKind::ServiceTask, None)),
        "userTask" => Some((BpmnNodeKind::UserTask, None)),
        "manualTask" => Some((BpmnNodeKind::ManualTask, None)),
        "businessRuleTask" => Some((BpmnNodeKind::BusinessRuleTask, None)),
        "parallelGateway" => Some((BpmnNodeKind::Gateway, Some(BpmnGatewayKind::Parallel))),
        "exclusiveGateway" => Some((BpmnNodeKind::Gateway, Some(BpmnGatewayKind::Exclusive))),
        "eventBasedGateway" => Some((BpmnNodeKind::Gateway, Some(BpmnGatewayKind::EventBased))),
        _ => None,
    }
}

fn is_supported_node_tag(tag: &str) -> bool {
    supported_node_kind(tag).is_some()
}

fn supported_event_definition(tag: &str) -> Option<BpmnEventKind> {
    match tag {
        "messageEventDefinition" => Some(BpmnEventKind::Message),
        "signalEventDefinition" => Some(BpmnEventKind::Signal),
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

fn is_ignored_process_child(tag: &str) -> bool {
    matches!(
        tag,
        "documentation"
            | "extensionElements"
            | "incoming"
            | "outgoing"
            | "association"
            | "textAnnotation"
    )
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

fn event_reference_id(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
) -> Result<Option<String>> {
    let attribute_name = match tag {
        "messageEventDefinition" => "messageRef",
        "signalEventDefinition" => "signalRef",
        _ => return Ok(None),
    };
    attribute_value(reader, event, attribute_name)
}

fn required_attribute(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    element: &str,
    attribute: &str,
) -> Result<String> {
    attribute_value(reader, event, attribute)?.ok_or_else(|| BpmnEngineError::MissingAttribute {
        source_id: source.source_id.clone(),
        element: element.to_string(),
        attribute: attribute.to_string(),
    })
}

fn attribute_value(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    attribute_name: &str,
) -> Result<Option<String>> {
    for attribute in event.attributes() {
        let attribute =
            attribute.map_err(|error| BpmnEngineError::CheckpointCodec(error.to_string()))?;
        if local_name(attribute.key.as_ref()) == attribute_name {
            let value = attribute
                .decode_and_unescape_value(reader.decoder())
                .map_err(|error| BpmnEngineError::CheckpointCodec(error.to_string()))?;
            return Ok(Some(match value {
                Cow::Borrowed(value) => value.to_string(),
                Cow::Owned(value) => value,
            }));
        }
    }
    Ok(None)
}

fn boolean_attribute_value(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    attribute_name: &str,
) -> Result<Option<bool>> {
    Ok(
        match attribute_value(reader, event, attribute_name)?.as_deref() {
            None => None,
            Some("true" | "1") => Some(true),
            Some(_) => Some(false),
        },
    )
}

fn parse_optional_u32_attribute(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    attribute_name: &str,
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> Result<Option<u32>> {
    attribute_value(reader, event, attribute_name)?
        .map(|value| {
            value
                .parse::<u32>()
                .map_err(|_| BpmnEngineError::UnsupportedLoopConfiguration {
                    process_id: process_id.to_string(),
                    node_id: node_id.to_string(),
                    detail,
                })
        })
        .transpose()
}

fn decision_reference(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<Option<DmnDecisionRef>> {
    let Some(decision_id) = attribute_value(reader, event, "decisionRef")? else {
        return Ok(None);
    };
    let decision = match attribute_value(reader, event, "decisionRefSource")? {
        Some(source_id) => DmnDecisionRef::new(decision_id).with_source_id(source_id),
        None => DmnDecisionRef::new(decision_id),
    };
    Ok(Some(decision))
}

fn cancel_activity_value(reader: &Reader<&[u8]>, event: &BytesStart<'_>) -> Result<bool> {
    Ok(!matches!(
        attribute_value(reader, event, "cancelActivity")?.as_deref(),
        Some("false" | "0")
    ))
}

fn apply_timer_expression(
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

fn last_process_node_mut<'a>(
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

fn apply_standard_loop_condition(process: &mut RawProcess, loop_condition: &str) -> Result<()> {
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

fn apply_multi_instance_loop_cardinality(
    process: &mut RawProcess,
    loop_cardinality: &str,
) -> Result<()> {
    let process_id = process.process_id.clone();
    let node = process
        .nodes
        .last_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "apply_multi_instance_loop_cardinality_missing_node",
        })?;
    let node_id = node.bpmn_id.clone();
    let Some(RawRepeatSpec::SequentialMultiInstance(loop_spec)) = node.repeat.as_mut() else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "apply_multi_instance_loop_cardinality_missing_repeat_spec",
        });
    };
    let trimmed = loop_cardinality.trim();
    loop_spec.loop_cardinality =
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.parse::<u32>().map_err(|_| {
                BpmnEngineError::UnsupportedLoopConfiguration {
                    process_id,
                    node_id,
                    detail: "invalid_loop_cardinality",
                }
            })?)
        };
    Ok(())
}

fn local_name(name: &[u8]) -> &str {
    std::str::from_utf8(name)
        .ok()
        .map_or("", |raw| raw.rsplit(':').next().unwrap_or(raw))
}
