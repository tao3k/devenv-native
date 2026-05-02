use crate::error::{BpmnEngineError, Result};
use crate::ir_event_api::BpmnEventKind;
use crate::ir_node_api::BpmnNodeKind;
use crate::parser::import::{RawNode, RawProcess, RawRepeatSpec, RawSubProcessKind};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub(in crate::parser) struct BoundaryAttachmentUsage {
    total: u32,
    cancel: u32,
    transaction_error: u32,
    transaction_escalation: u32,
    transaction_external: u32,
    call_activity_error: u32,
    call_activity_escalation: u32,
    call_activity_external: u32,
    embedded_error: u32,
    embedded_escalation: u32,
    embedded_external: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BoundaryOwnerKind {
    Other,
    CallActivity,
    EmbeddedShell,
    TransactionShell,
}

pub(in crate::parser) fn validate_boundary_event(
    process: &RawProcess,
    node: &RawNode,
    node_ids: &HashSet<&str>,
    boundary_attachments: &mut HashMap<String, BoundaryAttachmentUsage>,
) -> Result<()> {
    let (attached_to_ref, attached_node) = resolve_boundary_attachment(process, node, node_ids)?;
    let event_kind = node.event.as_ref().map(|event| &event.kind);
    let usage = boundary_attachments
        .entry(attached_to_ref.to_string())
        .or_default();
    if !node.cancel_activity {
        return validate_non_interrupting_boundary(process, node, attached_node, event_kind, usage);
    }
    if validate_supported_subprocess_boundary(
        process,
        node,
        event_kind,
        usage,
        boundary_owner_kind(attached_node),
    )? {
        return Ok(());
    }
    if usage.total > 0 {
        return Err(BpmnEngineError::UnsupportedBoundaryEventConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "multiple_boundary_events_for_attached_node",
        });
    }
    usage.total += 1;

    if event_kind == Some(&BpmnEventKind::Cancel) {
        return Err(BpmnEngineError::UnsupportedBoundaryEventConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "cancel_boundary_requires_transaction_shell",
        });
    }
    if event_kind == Some(&BpmnEventKind::Error) {
        return Err(BpmnEngineError::UnsupportedBoundaryEventConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "error_boundary_requires_supported_subprocess_shell",
        });
    }
    if event_kind == Some(&BpmnEventKind::Escalation) {
        return Err(BpmnEngineError::UnsupportedBoundaryEventConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "escalation_boundary_requires_supported_subprocess_shell",
        });
    }
    if event_kind == Some(&BpmnEventKind::Compensation) {
        return Ok(());
    }
    if !matches!(
        attached_node.kind,
        BpmnNodeKind::ServiceTask
            | BpmnNodeKind::ScriptTask
            | BpmnNodeKind::UserTask
            | BpmnNodeKind::ManualTask
            | BpmnNodeKind::BusinessRuleTask
    ) {
        return Err(BpmnEngineError::UnsupportedBoundaryEventConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "unsupported_boundary_attachment_kind",
        });
    }
    if !matches!(
        event_kind,
        Some(
            BpmnEventKind::Timer
                | BpmnEventKind::Message
                | BpmnEventKind::Signal
                | BpmnEventKind::Conditional
        )
    ) {
        return Err(BpmnEngineError::UnsupportedBoundaryEventConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "unsupported_boundary_event_kind",
        });
    }

    Ok(())
}

fn validate_non_interrupting_boundary(
    process: &RawProcess,
    node: &RawNode,
    attached_node: &RawNode,
    event_kind: Option<&BpmnEventKind>,
    usage: &mut BoundaryAttachmentUsage,
) -> Result<()> {
    if usage.total > 0 {
        return Err(BpmnEngineError::UnsupportedBoundaryEventConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "multiple_boundary_events_for_attached_node",
        });
    }
    usage.total += 1;

    if event_kind == Some(&BpmnEventKind::Escalation) {
        return Err(BpmnEngineError::UnsupportedBoundaryEventConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "non_interrupting_escalation_boundary_deferred",
        });
    }

    if !matches!(
        attached_node.kind,
        BpmnNodeKind::ServiceTask
            | BpmnNodeKind::ScriptTask
            | BpmnNodeKind::UserTask
            | BpmnNodeKind::ManualTask
            | BpmnNodeKind::BusinessRuleTask
    ) {
        return Err(BpmnEngineError::UnsupportedBoundaryEventConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "unsupported_boundary_attachment_kind",
        });
    }
    if !supports_non_interrupting_boundary_repeat(attached_node) {
        return Err(BpmnEngineError::UnsupportedBoundaryEventConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "non_interrupting_boundary_requires_supported_task_repeat_owner",
        });
    }
    if !matches!(
        event_kind,
        Some(
            BpmnEventKind::Timer
                | BpmnEventKind::Message
                | BpmnEventKind::Signal
                | BpmnEventKind::Conditional
        )
    ) {
        return Err(BpmnEngineError::UnsupportedBoundaryEventConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "unsupported_boundary_event_kind",
        });
    }

    Ok(())
}

fn supports_non_interrupting_boundary_repeat(attached_node: &RawNode) -> bool {
    matches!(
        attached_node.repeat,
        None | Some(
            RawRepeatSpec::StandardLoop(_)
                | RawRepeatSpec::SequentialMultiInstance(_)
                | RawRepeatSpec::ParallelMultiInstance(_),
        )
    )
}

fn resolve_boundary_attachment<'a>(
    process: &'a RawProcess,
    node: &'a RawNode,
    node_ids: &HashSet<&str>,
) -> Result<(&'a str, &'a RawNode)> {
    let attached_to_ref = node.attached_to_ref.as_deref().ok_or_else(|| {
        BpmnEngineError::MissingRequiredNodeElement {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            element: "attached_to_ref",
        }
    })?;
    if !node_ids.contains(attached_to_ref) {
        return Err(BpmnEngineError::UnknownBoundaryAttachment {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            attached_to_node_id: attached_to_ref.to_string(),
        });
    }
    let attached_node = process
        .nodes
        .iter()
        .find(|candidate| candidate.bpmn_id == attached_to_ref)
        .ok_or_else(|| BpmnEngineError::UnknownBoundaryAttachment {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            attached_to_node_id: attached_to_ref.to_string(),
        })?;
    Ok((attached_to_ref, attached_node))
}

fn boundary_owner_kind(attached_node: &RawNode) -> BoundaryOwnerKind {
    if attached_node.kind != BpmnNodeKind::SubProcess {
        return BoundaryOwnerKind::Other;
    }
    match attached_node.subprocess_kind {
        Some(RawSubProcessKind::CallActivity) => BoundaryOwnerKind::CallActivity,
        Some(RawSubProcessKind::EmbeddedSubProcess) => BoundaryOwnerKind::EmbeddedShell,
        Some(RawSubProcessKind::Transaction) => BoundaryOwnerKind::TransactionShell,
        _ => BoundaryOwnerKind::Other,
    }
}

fn validate_supported_subprocess_boundary(
    process: &RawProcess,
    node: &RawNode,
    event_kind: Option<&BpmnEventKind>,
    usage: &mut BoundaryAttachmentUsage,
    owner_kind: BoundaryOwnerKind,
) -> Result<bool> {
    match owner_kind {
        BoundaryOwnerKind::TransactionShell => {
            validate_transaction_shell_boundary(process, node, event_kind, usage)
        }
        BoundaryOwnerKind::CallActivity => {
            validate_call_activity_boundary(process, node, event_kind, usage)
        }
        BoundaryOwnerKind::EmbeddedShell => {
            validate_embedded_shell_boundary(process, node, event_kind, usage)
        }
        BoundaryOwnerKind::Other => Ok(false),
    }
}

fn validate_transaction_shell_boundary(
    process: &RawProcess,
    node: &RawNode,
    event_kind: Option<&BpmnEventKind>,
    usage: &mut BoundaryAttachmentUsage,
) -> Result<bool> {
    if event_kind == Some(&BpmnEventKind::Cancel) {
        if usage.cancel > 0 {
            return boundary_configuration_error(
                process,
                node,
                "multiple_transaction_cancel_boundaries",
            );
        }
        usage.total += 1;
        usage.cancel += 1;
        return Ok(true);
    }
    if event_kind == Some(&BpmnEventKind::Error) {
        usage.total += 1;
        usage.transaction_error += 1;
        return Ok(true);
    }
    if event_kind == Some(&BpmnEventKind::Escalation) {
        usage.total += 1;
        usage.transaction_escalation += 1;
        return Ok(true);
    }
    if matches!(
        event_kind,
        Some(
            BpmnEventKind::Timer
                | BpmnEventKind::Message
                | BpmnEventKind::Signal
                | BpmnEventKind::Conditional
        )
    ) {
        if usage.transaction_external > 0 {
            return boundary_configuration_error(
                process,
                node,
                "multiple_boundary_events_for_attached_node",
            );
        }
        usage.total += 1;
        usage.transaction_external += 1;
        return Ok(true);
    }
    Ok(false)
}

fn validate_call_activity_boundary(
    process: &RawProcess,
    node: &RawNode,
    event_kind: Option<&BpmnEventKind>,
    usage: &mut BoundaryAttachmentUsage,
) -> Result<bool> {
    if event_kind == Some(&BpmnEventKind::Error) {
        usage.total += 1;
        usage.call_activity_error += 1;
        return Ok(true);
    }
    if event_kind == Some(&BpmnEventKind::Escalation) {
        usage.total += 1;
        usage.call_activity_escalation += 1;
        return Ok(true);
    }
    if matches!(
        event_kind,
        Some(
            BpmnEventKind::Timer
                | BpmnEventKind::Message
                | BpmnEventKind::Signal
                | BpmnEventKind::Conditional
        )
    ) {
        if usage.call_activity_external > 0
            || usage.total > usage.call_activity_error + usage.call_activity_escalation
        {
            return boundary_configuration_error(
                process,
                node,
                "multiple_boundary_events_for_attached_node",
            );
        }
        usage.total += 1;
        usage.call_activity_external += 1;
        return Ok(true);
    }
    Ok(false)
}

fn validate_embedded_shell_boundary(
    process: &RawProcess,
    node: &RawNode,
    event_kind: Option<&BpmnEventKind>,
    usage: &mut BoundaryAttachmentUsage,
) -> Result<bool> {
    if event_kind == Some(&BpmnEventKind::Error) {
        usage.total += 1;
        usage.embedded_error += 1;
        return Ok(true);
    }
    if event_kind == Some(&BpmnEventKind::Escalation) {
        usage.total += 1;
        usage.embedded_escalation += 1;
        return Ok(true);
    }
    if matches!(
        event_kind,
        Some(
            BpmnEventKind::Timer
                | BpmnEventKind::Message
                | BpmnEventKind::Signal
                | BpmnEventKind::Conditional
        )
    ) {
        if usage.embedded_external > 0
            || usage.total > usage.embedded_error + usage.embedded_escalation
        {
            return boundary_configuration_error(
                process,
                node,
                "multiple_boundary_events_for_attached_node",
            );
        }
        usage.total += 1;
        usage.embedded_external += 1;
        return Ok(true);
    }
    Ok(false)
}

fn boundary_configuration_error(
    process: &RawProcess,
    node: &RawNode,
    detail: &'static str,
) -> Result<bool> {
    Err(BpmnEngineError::UnsupportedBoundaryEventConfiguration {
        process_id: process.process_id.clone(),
        node_id: node.bpmn_id.clone(),
        detail,
    })
}
