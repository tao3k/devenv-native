use super::attributes::local_name;
use super::model::{CaptureTarget, ProcessChildStartOutcome};
pub(crate) use super::model::{
    NestedShellKind, RawAssociation, RawEventSpec, RawNode, RawPackageDocument,
    RawParallelMultiInstanceSpec, RawProcess, RawProcessScope, RawRepeatSpec, RawScriptTaskSpec,
    RawSequenceFlow, RawSequentialMultiInstanceSpec, RawSubProcessKind,
};
use super::nested::handle_nested_start_tag;
use super::process::{
    complete_process_scope, handle_package_start_tag, handle_process_child_start_tag,
};
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::{BpmnEngineError, Result};
use quick_xml::Reader;
use quick_xml::escape::{resolve_predefined_entity, unescape};
use quick_xml::events::{BytesRef, BytesStart, Event};
use std::borrow::Cow;

pub(crate) fn import_bpmn_source(source: &BpmnSourceFile) -> Result<RawPackageDocument> {
    let mut reader = Reader::from_str(&source.contents);
    reader.config_mut().trim_text(true);

    let mut saw_root = false;
    let mut stack = Vec::new();
    let mut package_id = None;
    let mut processes = Vec::new();
    let mut process_stack = Vec::new();
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
                    &mut process_stack,
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
                    &mut process_stack,
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
            Ok(Event::GeneralRef(event)) => append_capture_reference(
                source,
                capture_target.as_ref(),
                &mut capture_buffer,
                &event,
            )?,
            Ok(Event::End(event)) => {
                let tag = local_name(event.name().as_ref()).to_string();
                handle_end_tag(
                    source,
                    &tag,
                    &mut process_stack,
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
                    offset: Some(reader.error_position()),
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
    process_stack: &mut Vec<RawProcess>,
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
        process_stack,
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
        offset: None,
    })?;
    let text = unescape(text.as_ref()).map_err(|error| BpmnEngineError::InvalidXml {
        source_id: source.source_id.clone(),
        message: error.to_string(),
        offset: None,
    })?;
    capture_buffer.push_str(text.as_ref());
    Ok(())
}

fn append_capture_reference(
    source: &BpmnSourceFile,
    capture_target: Option<&CaptureTarget>,
    capture_buffer: &mut String,
    reference: &BytesRef<'_>,
) -> Result<()> {
    if capture_target.is_none() {
        return Ok(());
    }
    if let Some(ch) = reference
        .resolve_char_ref()
        .map_err(|error| BpmnEngineError::InvalidXml {
            source_id: source.source_id.clone(),
            message: error.to_string(),
            offset: None,
        })?
    {
        capture_buffer.push(ch);
        return Ok(());
    }

    let reference = reference
        .decode()
        .map_err(|error| BpmnEngineError::InvalidXml {
            source_id: source.source_id.clone(),
            message: error.to_string(),
            offset: None,
        })?;
    let entity = resolve_predefined_entity(reference.as_ref()).ok_or_else(|| {
        BpmnEngineError::InvalidXml {
            source_id: source.source_id.clone(),
            message: format!("unrecognized XML entity reference '&{reference};'"),
            offset: None,
        }
    })?;
    capture_buffer.push_str(entity);
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
    process_stack: &mut Vec<RawProcess>,
    processes: &mut Vec<RawProcess>,
    capture_target: &mut Option<CaptureTarget>,
    capture_buffer: &mut String,
    is_empty: bool,
) -> Result<()> {
    if !process_stack.is_empty()
        && let Some(element) = legacy_custom_qname_element(event)
    {
        let process_id = process_stack
            .last()
            .map(|process| process.process_id.clone())
            .unwrap_or_default();
        return Err(BpmnEngineError::UnsupportedElement {
            source_id: source.source_id.clone(),
            process_id,
            element,
        });
    }

    if handle_package_start_tag(
        source,
        reader,
        event,
        tag,
        package_id,
        process_stack,
        processes,
        is_empty,
    )? {
        return Ok(());
    }

    if process_stack.is_empty() {
        return Ok(());
    }

    match handle_process_child_start_tag(
        source,
        reader,
        event,
        tag,
        parent,
        process_stack,
        is_empty,
    )? {
        ProcessChildStartOutcome::NotHandled => {}
        ProcessChildStartOutcome::Handled => return Ok(()),
        ProcessChildStartOutcome::OpenedNestedShell => {
            if is_empty {
                complete_process_scope(tag, process_stack, processes);
            }
            return Ok(());
        }
    }

    let Some(process) = process_stack.last_mut() else {
        return Ok(());
    };

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

fn legacy_custom_qname_element(event: &BytesStart<'_>) -> Option<String> {
    let event_name = event.name();
    let name = std::str::from_utf8(event_name.as_ref()).ok()?;
    matches!(
        name,
        "qianji:config"
            | "qianji:interaction"
            | "qianji:choice"
            | "qianji:choices"
            | "qianji:freeText"
            | "qianji:inputs"
            | "qianji:outputSchema"
            | "qianji:outputs"
            | "qianji:prompt"
            | "qianji:question"
            | "qianji:result"
            | "qianji:tools"
            | "qianji:toolScope"
    )
    .then(|| name.to_string())
}

fn handle_end_tag(
    source: &BpmnSourceFile,
    tag: &str,
    process_stack: &mut Vec<RawProcess>,
    processes: &mut Vec<RawProcess>,
    capture_target: &mut Option<CaptureTarget>,
    capture_buffer: &mut String,
) -> Result<()> {
    if matches!(tag, "process" | "subProcess" | "transaction") {
        complete_process_scope(tag, process_stack, processes);
        return Ok(());
    }

    let Some(process) = process_stack.last_mut() else {
        return Ok(());
    };

    super::task_io::complete_task_io_end_tag(source, process, tag)?;
    super::human_task_io::complete_human_task_io_end_tag(source, process, tag)?;

    let Some(target) = capture_target.clone() else {
        return Ok(());
    };

    if apply_task_io_capture_end(source, process, &target, tag, capture_buffer.trim())? {
        *capture_target = None;
        capture_buffer.clear();
        return Ok(());
    }

    if apply_human_task_io_capture_end(source, process, &target, tag, capture_buffer.trim())? {
        *capture_target = None;
        capture_buffer.clear();
        return Ok(());
    }

    match (target, tag) {
        (CaptureTarget::TimerExpression(kind), "timeDate")
            if kind == crate::ir_event_api::BpmnTimerKind::Date =>
        {
            super::capture::apply_timer_expression(process, kind, capture_buffer.trim())?;
        }
        (CaptureTarget::TimerExpression(kind), "timeDuration")
            if kind == crate::ir_event_api::BpmnTimerKind::Duration =>
        {
            super::capture::apply_timer_expression(process, kind, capture_buffer.trim())?;
        }
        (CaptureTarget::TimerExpression(kind), "timeCycle")
            if kind == crate::ir_event_api::BpmnTimerKind::Cycle =>
        {
            super::capture::apply_timer_expression(process, kind, capture_buffer.trim())?;
        }
        (CaptureTarget::ConditionalExpression, "condition") => {
            super::capture::apply_conditional_expression(process, capture_buffer.trim())?;
        }
        (CaptureTarget::StandardLoopCondition, "loopCondition") => {
            super::capture::apply_standard_loop_condition(process, capture_buffer.trim())?;
        }
        (CaptureTarget::MultiInstanceLoopCardinality, "loopCardinality") => {
            super::capture::apply_multi_instance_loop_cardinality(process, capture_buffer.trim())?;
        }
        (CaptureTarget::MultiInstanceLoopDataInputRef, "loopDataInputRef") => {
            super::capture::apply_multi_instance_loop_data_input_ref(
                process,
                capture_buffer.trim(),
            )?;
        }
        (CaptureTarget::MultiInstanceLoopDataOutputRef, "loopDataOutputRef") => {
            super::capture::apply_multi_instance_loop_data_output_ref(
                process,
                capture_buffer.trim(),
            )?;
        }
        (CaptureTarget::MultiInstanceCompletionCondition, "completionCondition") => {
            super::capture::apply_multi_instance_completion_condition(
                process,
                capture_buffer.trim(),
            )?;
        }
        (CaptureTarget::SequenceFlowConditionExpression, "conditionExpression") => {
            super::capture::apply_sequence_flow_condition_expression(
                process,
                capture_buffer.trim(),
            )?;
        }
        (CaptureTarget::TaskScriptBody, "script") => {
            super::capture::apply_script_task_body(process, capture_buffer.trim())?;
        }
        (CaptureTarget::HumanTaskResourceRef(kind), "resourceRef") => {
            super::capture::apply_human_task_resource_ref(process, kind, capture_buffer.trim())?;
        }
        (CaptureTarget::HumanTaskAssignmentExpression(kind), "formalExpression") => {
            super::capture::apply_human_task_assignment_expression(
                process,
                kind,
                capture_buffer.trim(),
            )?;
        }
        _ => return Ok(()),
    }

    *capture_target = None;
    capture_buffer.clear();
    Ok(())
}

fn apply_task_io_capture_end(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    target: &CaptureTarget,
    tag: &str,
    text: &str,
) -> Result<bool> {
    match (target, tag) {
        (CaptureTarget::TaskIoSourceRef, "sourceRef") => {
            super::task_io::apply_task_io_source_ref(source, process, text)?;
        }
        (CaptureTarget::TaskIoTargetRef, "targetRef") => {
            super::task_io::apply_task_io_target_ref(source, process, text)?;
        }
        (CaptureTarget::TaskIoAssignmentFrom, "from") => {
            super::task_io::apply_task_io_assignment_from(source, process, text)?;
        }
        (CaptureTarget::TaskIoAssignmentTo, "to") => {
            super::task_io::apply_task_io_assignment_to(source, process, text)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn apply_human_task_io_capture_end(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    target: &CaptureTarget,
    tag: &str,
    text: &str,
) -> Result<bool> {
    match (target, tag) {
        (CaptureTarget::HumanTaskDocumentationText, "documentation") => {
            super::human_task_io::apply_human_task_documentation_text(source, process, text)?;
        }
        (CaptureTarget::HumanTaskIoSourceRef, "sourceRef") => {
            super::human_task_io::apply_human_task_io_source_ref(source, process, text)?;
        }
        (CaptureTarget::HumanTaskIoTargetRef, "targetRef") => {
            super::human_task_io::apply_human_task_io_target_ref(source, process, text)?;
        }
        (CaptureTarget::HumanTaskIoAssignmentFrom, "from") => {
            super::human_task_io::apply_human_task_io_assignment_from(source, process, text)?;
        }
        (CaptureTarget::HumanTaskIoAssignmentTo, "to") => {
            super::human_task_io::apply_human_task_io_assignment_to(source, process, text)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}
