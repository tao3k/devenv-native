use super::scope::{current_process_mut, is_process_scope_tag};
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::{BpmnEngineError, Result};
use crate::ir_node_api::{BpmnGatewayKind, BpmnNodeKind};
use crate::parser::import::attributes::{
    attribute_value, boolean_attribute_value, cancel_activity_value, decision_reference,
    required_attribute,
};
use crate::parser::import::model::ProcessChildStartOutcome;
use crate::parser::import::{
    NestedShellKind, RawAssociation, RawNode, RawProcess, RawScriptTaskSpec, RawSequenceFlow,
    RawSubProcessKind,
};
use quick_xml::Reader;
use quick_xml::events::BytesStart;

pub(in crate::parser::import) fn handle_process_child_start_tag(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    parent: Option<&str>,
    process_stack: &mut Vec<RawProcess>,
    _is_empty: bool,
) -> Result<ProcessChildStartOutcome> {
    if !parent.is_some_and(is_process_scope_tag) {
        return Ok(ProcessChildStartOutcome::NotHandled);
    }
    if let Some(kind) = nested_shell_kind(tag) {
        open_nested_shell(source, reader, event, tag, kind, process_stack)?;
        return Ok(ProcessChildStartOutcome::OpenedNestedShell);
    }
    if push_process_child_node(source, reader, event, tag, process_stack)? {
        return Ok(ProcessChildStartOutcome::Handled);
    }
    if push_process_child_sequence_flow(source, reader, event, tag, process_stack)? {
        return Ok(ProcessChildStartOutcome::Handled);
    }
    if push_process_child_association(source, reader, event, tag, process_stack)? {
        return Ok(ProcessChildStartOutcome::Handled);
    }
    if is_ignored_process_child(tag) {
        return Ok(ProcessChildStartOutcome::Handled);
    }
    let process = current_process_mut(process_stack, "bpmn_process_child_without_process_frame")?;
    Err(BpmnEngineError::UnsupportedElement {
        source_id: source.source_id.clone(),
        process_id: process.process_id.clone(),
        element: tag.to_string(),
    })
}

fn open_nested_shell(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    kind: NestedShellKind,
    process_stack: &mut Vec<RawProcess>,
) -> Result<()> {
    let parent_process_id = process_stack
        .last()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "bpmn_nested_shell_without_parent_process",
        })?
        .process_id
        .clone();
    let bpmn_id = required_attribute(source, reader, event, tag, "id")?;
    if kind == NestedShellKind::EmbeddedSubProcess
        && boolean_attribute_value(reader, event, "triggeredByEvent")?.unwrap_or(false)
    {
        return Err(BpmnEngineError::UnsupportedSubProcessConfiguration {
            process_id: parent_process_id,
            node_id: bpmn_id,
            detail: "event_subprocess",
        });
    }
    let decision = decision_reference(reader, event)?;
    let synthetic_process_id =
        synthetic_nested_shell_process_id(kind, &parent_process_id, &bpmn_id);
    let process = current_process_mut(process_stack, "bpmn_nested_shell_missing_process_frame")?;
    process.nodes.push(RawNode {
        bpmn_id: bpmn_id.clone(),
        kind: BpmnNodeKind::SubProcess,
        gateway_kind: None,
        decision,
        lane: None,
        task_message_ref: None,
        script_task: None,
        human_task_form: None,
        human_task_assignment: None,
        called_process_ref: Some(synthetic_process_id.clone()),
        subprocess_kind: Some(raw_subprocess_kind_for_nested_shell(kind)),
        repeat: None,
        attached_to_ref: None,
        default_flow_ref: None,
        cancel_activity: true,
        is_for_compensation: false,
        event: None,
    });
    process_stack.push(RawProcess::new_nested_shell(
        synthetic_process_id,
        parent_process_id,
        bpmn_id,
        kind,
    ));
    Ok(())
}

fn push_process_child_node(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    process_stack: &mut [RawProcess],
) -> Result<bool> {
    let Some((kind, gateway_kind)) = supported_node_kind(tag) else {
        return Ok(false);
    };
    let bpmn_id = required_attribute(source, reader, event, tag, "id")?;
    let decision = decision_reference(reader, event)?;
    let task_message_ref = if matches!(tag, "sendTask" | "receiveTask") {
        attribute_value(reader, event, "messageRef")?
    } else {
        None
    };
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
    let script_task = if tag == "scriptTask" {
        Some(RawScriptTaskSpec {
            script_format: attribute_value(reader, event, "scriptFormat")?,
            script_body: None,
        })
    } else {
        None
    };
    let subprocess_kind = if tag == "callActivity" {
        Some(RawSubProcessKind::CallActivity)
    } else {
        None
    };
    let default_flow_ref = if matches!(
        gateway_kind,
        Some(BpmnGatewayKind::Exclusive | BpmnGatewayKind::Inclusive)
    ) {
        attribute_value(reader, event, "default")?
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
    let is_for_compensation =
        boolean_attribute_value(reader, event, "isForCompensation")?.unwrap_or(false);
    let process = current_process_mut(process_stack, "bpmn_process_child_without_process_frame")?;
    process.nodes.push(RawNode {
        bpmn_id,
        kind,
        gateway_kind,
        decision,
        lane: None,
        task_message_ref,
        script_task,
        human_task_form: None,
        human_task_assignment: None,
        called_process_ref,
        subprocess_kind,
        repeat: None,
        attached_to_ref,
        default_flow_ref,
        cancel_activity,
        is_for_compensation,
        event: None,
    });
    Ok(true)
}

fn push_process_child_sequence_flow(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    process_stack: &mut [RawProcess],
) -> Result<bool> {
    if tag != "sequenceFlow" {
        return Ok(false);
    }
    let flow_id = required_attribute(source, reader, event, tag, "id")?;
    let source_ref = required_attribute(source, reader, event, tag, "sourceRef")?;
    let target_ref = required_attribute(source, reader, event, tag, "targetRef")?;
    let label = attribute_value(reader, event, "name")?;
    let process = current_process_mut(process_stack, "bpmn_process_child_without_process_frame")?;
    process.flows.push(RawSequenceFlow {
        flow_id,
        source_ref,
        target_ref,
        label,
        condition_expression: None,
    });
    Ok(true)
}

fn push_process_child_association(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    process_stack: &mut [RawProcess],
) -> Result<bool> {
    if tag != "association" {
        return Ok(false);
    }
    let process = current_process_mut(process_stack, "bpmn_process_child_without_process_frame")?;
    process.associations.push(RawAssociation {
        association_id: required_attribute(source, reader, event, tag, "id")?,
        source_ref: required_attribute(source, reader, event, tag, "sourceRef")?,
        target_ref: required_attribute(source, reader, event, tag, "targetRef")?,
    });
    Ok(true)
}

fn supported_node_kind(tag: &str) -> Option<(BpmnNodeKind, Option<BpmnGatewayKind>)> {
    match tag {
        "startEvent" => Some((BpmnNodeKind::StartEvent, None)),
        "endEvent" => Some((BpmnNodeKind::EndEvent, None)),
        "intermediateThrowEvent" => Some((BpmnNodeKind::IntermediateThrowEvent, None)),
        "intermediateCatchEvent" => Some((BpmnNodeKind::IntermediateCatchEvent, None)),
        "boundaryEvent" => Some((BpmnNodeKind::BoundaryEvent, None)),
        "callActivity" => Some((BpmnNodeKind::SubProcess, None)),
        "sendTask" => Some((BpmnNodeKind::SendTask, None)),
        "receiveTask" => Some((BpmnNodeKind::ReceiveTask, None)),
        "serviceTask" => Some((BpmnNodeKind::ServiceTask, None)),
        "scriptTask" => Some((BpmnNodeKind::ScriptTask, None)),
        "userTask" => Some((BpmnNodeKind::UserTask, None)),
        "manualTask" => Some((BpmnNodeKind::ManualTask, None)),
        "businessRuleTask" => Some((BpmnNodeKind::BusinessRuleTask, None)),
        "parallelGateway" => Some((BpmnNodeKind::Gateway, Some(BpmnGatewayKind::Parallel))),
        "exclusiveGateway" => Some((BpmnNodeKind::Gateway, Some(BpmnGatewayKind::Exclusive))),
        "inclusiveGateway" => Some((BpmnNodeKind::Gateway, Some(BpmnGatewayKind::Inclusive))),
        "eventBasedGateway" => Some((BpmnNodeKind::Gateway, Some(BpmnGatewayKind::EventBased))),
        _ => None,
    }
}

pub(in crate::parser::import) fn is_supported_node_tag(tag: &str) -> bool {
    supported_node_kind(tag).is_some()
}

fn nested_shell_kind(tag: &str) -> Option<NestedShellKind> {
    match tag {
        "subProcess" => Some(NestedShellKind::EmbeddedSubProcess),
        "transaction" => Some(NestedShellKind::Transaction),
        _ => None,
    }
}

fn synthetic_nested_shell_process_id(
    kind: NestedShellKind,
    parent_process_id: &str,
    node_id: &str,
) -> String {
    let prefix = match kind {
        NestedShellKind::EmbeddedSubProcess => "__embedded_subprocess__",
        NestedShellKind::Transaction => "__transaction__",
    };
    format!("{prefix}::{parent_process_id}::{node_id}")
}

fn raw_subprocess_kind_for_nested_shell(kind: NestedShellKind) -> RawSubProcessKind {
    match kind {
        NestedShellKind::EmbeddedSubProcess => RawSubProcessKind::EmbeddedSubProcess,
        NestedShellKind::Transaction => RawSubProcessKind::Transaction,
    }
}

fn is_ignored_process_child(tag: &str) -> bool {
    matches!(
        tag,
        "documentation"
            | "extensionElements"
            | "incoming"
            | "outgoing"
            | "laneSet"
            | "textAnnotation"
    )
}
