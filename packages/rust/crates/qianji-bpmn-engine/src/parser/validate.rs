//! BPMN validation for the bounded supported subset.

use super::import::{
    NestedShellKind, RawNode, RawPackageDocument, RawProcess, RawProcessScope, RawRepeatSpec,
    RawSubProcessKind,
};
use crate::error::{BpmnEngineError, Result};
use crate::ir_event_api::BpmnEventKind;
use crate::ir_node_api::{BpmnGatewayKind, BpmnNodeKind};
use crate::repeat_condition::is_supported_multi_instance_completion_condition;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
struct BoundaryAttachmentUsage {
    total_count: u32,
    cancel_count: u32,
}

pub(crate) fn validate_raw_package(raw: &RawPackageDocument) -> Result<()> {
    ensure_process_definitions(raw)?;
    let all_process_ids = raw
        .processes
        .iter()
        .map(|process| process.process_id.as_str())
        .collect::<HashSet<_>>();
    let process_by_id = raw
        .processes
        .iter()
        .map(|process| (process.process_id.as_str(), process))
        .collect::<HashMap<_, _>>();
    let mut seen_process_ids = HashSet::new();
    for process in &raw.processes {
        ensure_unique_process_id(raw, process, &mut seen_process_ids)?;
        let node_ids = collect_node_ids(process)?;
        validate_process_topology(process, &all_process_ids, &node_ids, &process_by_id)?;
        validate_sequence_flows(process, &node_ids)?;
        validate_standard_loops(process)?;
        validate_multi_instances(process)?;
        validate_compensation_handlers(process)?;
        validate_event_based_gateways(process, &node_ids)?;
    }

    detect_recursive_call_activity(raw)?;

    Ok(())
}

fn ensure_process_definitions(raw: &RawPackageDocument) -> Result<()> {
    if raw.processes.is_empty() {
        return Err(BpmnEngineError::MissingProcessDefinitions {
            source_id: raw.source_id.clone(),
        });
    }
    Ok(())
}

fn ensure_unique_process_id<'a>(
    raw: &RawPackageDocument,
    process: &'a super::import::RawProcess,
    seen_process_ids: &mut HashSet<&'a str>,
) -> Result<()> {
    if seen_process_ids.insert(process.process_id.as_str()) {
        return Ok(());
    }
    Err(BpmnEngineError::DuplicateProcessId {
        package_id: raw.package_id.clone(),
        process_id: process.process_id.clone(),
    })
}

fn collect_node_ids(process: &super::import::RawProcess) -> Result<HashSet<&str>> {
    let mut seen_node_ids = HashSet::new();
    let mut node_ids = HashSet::new();
    let mut start_event_count = 0usize;
    let mut has_end_event = false;

    for node in &process.nodes {
        if !seen_node_ids.insert(node.bpmn_id.as_str()) {
            return Err(BpmnEngineError::DuplicateNodeId {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
            });
        }
        node_ids.insert(node.bpmn_id.as_str());
        if matches!(node.kind, BpmnNodeKind::StartEvent) {
            start_event_count += 1;
        }
        has_end_event |= matches!(node.kind, BpmnNodeKind::EndEvent);
    }

    match &process.scope {
        RawProcessScope::TopLevel => {
            if start_event_count == 0 {
                return Err(BpmnEngineError::MissingRequiredProcessElement {
                    process_id: process.process_id.clone(),
                    element: "start_event",
                });
            }
        }
        RawProcessScope::NestedShell {
            owner_process_id,
            owner_node_id,
            kind,
        } => {
            if start_event_count != 1 {
                return Err(BpmnEngineError::UnsupportedSubProcessConfiguration {
                    process_id: owner_process_id.clone(),
                    node_id: owner_node_id.clone(),
                    detail: nested_shell_start_event_detail(*kind),
                });
            }
        }
    }
    if !has_end_event {
        return Err(match &process.scope {
            RawProcessScope::TopLevel => BpmnEngineError::MissingRequiredProcessElement {
                process_id: process.process_id.clone(),
                element: "end_event",
            },
            RawProcessScope::NestedShell {
                owner_process_id,
                owner_node_id,
                kind,
            } => BpmnEngineError::UnsupportedSubProcessConfiguration {
                process_id: owner_process_id.clone(),
                node_id: owner_node_id.clone(),
                detail: nested_shell_missing_end_detail(*kind),
            },
        });
    }

    Ok(node_ids)
}

fn nested_shell_start_event_detail(kind: NestedShellKind) -> &'static str {
    match kind {
        NestedShellKind::EmbeddedSubProcess => "embedded_subprocess_start_event_count",
        NestedShellKind::Transaction => "transaction_start_event_count",
    }
}

fn nested_shell_missing_end_detail(kind: NestedShellKind) -> &'static str {
    match kind {
        NestedShellKind::EmbeddedSubProcess => "embedded_subprocess_missing_end_event",
        NestedShellKind::Transaction => "transaction_missing_end_event",
    }
}

fn validate_process_topology(
    process: &RawProcess,
    all_process_ids: &HashSet<&str>,
    node_ids: &HashSet<&str>,
    process_by_id: &HashMap<&str, &RawProcess>,
) -> Result<()> {
    let mut boundary_attachments = HashMap::new();
    validate_transaction_cancel_path(process, process_by_id)?;
    validate_transaction_error_path(process, process_by_id)?;
    for node in &process.nodes {
        validate_node_event_shape(process, node)?;
        if node.kind == BpmnNodeKind::SubProcess {
            validate_called_process_reference(process, node, all_process_ids)?;
        }
        if node.kind == BpmnNodeKind::BoundaryEvent {
            validate_boundary_event(process, node, node_ids, &mut boundary_attachments)?;
        }
    }
    Ok(())
}

fn validate_transaction_cancel_path(
    process: &RawProcess,
    process_by_id: &HashMap<&str, &RawProcess>,
) -> Result<()> {
    let cancel_end_nodes = process
        .nodes
        .iter()
        .filter(|node| {
            node.kind == BpmnNodeKind::EndEvent
                && node.event.as_ref().map(|event| event.kind.clone())
                    == Some(BpmnEventKind::Cancel)
        })
        .collect::<Vec<_>>();
    if cancel_end_nodes.is_empty() {
        return Ok(());
    }

    let RawProcessScope::NestedShell {
        owner_process_id,
        owner_node_id,
        kind: NestedShellKind::Transaction,
    } = &process.scope
    else {
        return Err(BpmnEngineError::UnsupportedTransactionConfiguration {
            process_id: process.process_id.clone(),
            node_id: cancel_end_nodes[0].bpmn_id.clone(),
            detail: "cancel_end_requires_transaction_shell",
        });
    };

    if cancel_end_nodes.len() > 1 {
        return Err(BpmnEngineError::UnsupportedTransactionConfiguration {
            process_id: owner_process_id.clone(),
            node_id: owner_node_id.clone(),
            detail: "multiple_transaction_cancel_end_events",
        });
    }

    let Some(parent_process) = process_by_id.get(owner_process_id.as_str()).copied() else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "validate_transaction_cancel_missing_parent_process",
        });
    };

    let has_matching_boundary = parent_process.nodes.iter().any(|node| {
        node.kind == BpmnNodeKind::BoundaryEvent
            && node.attached_to_ref.as_deref() == Some(owner_node_id.as_str())
            && node.cancel_activity
            && node.event.as_ref().map(|event| event.kind.clone()) == Some(BpmnEventKind::Cancel)
    });
    if has_matching_boundary {
        return Ok(());
    }

    Err(BpmnEngineError::UnsupportedTransactionConfiguration {
        process_id: owner_process_id.clone(),
        node_id: owner_node_id.clone(),
        detail: "transaction_cancel_missing_boundary",
    })
}

fn validate_transaction_error_path(
    process: &RawProcess,
    process_by_id: &HashMap<&str, &RawProcess>,
) -> Result<()> {
    let error_end_nodes = process
        .nodes
        .iter()
        .filter(|node| {
            node.kind == BpmnNodeKind::EndEvent
                && node.event.as_ref().map(|event| event.kind.clone()) == Some(BpmnEventKind::Error)
        })
        .collect::<Vec<_>>();
    if error_end_nodes.is_empty() {
        return Ok(());
    }

    let RawProcessScope::NestedShell {
        owner_process_id,
        owner_node_id,
        kind: NestedShellKind::Transaction,
    } = &process.scope
    else {
        return Err(BpmnEngineError::UnsupportedTransactionConfiguration {
            process_id: process.process_id.clone(),
            node_id: error_end_nodes[0].bpmn_id.clone(),
            detail: "error_end_requires_transaction_shell",
        });
    };

    if error_end_nodes.len() > 1 {
        return Err(BpmnEngineError::UnsupportedTransactionConfiguration {
            process_id: owner_process_id.clone(),
            node_id: owner_node_id.clone(),
            detail: "multiple_transaction_error_end_events",
        });
    }

    let thrown_reference_id = error_end_nodes[0]
        .event
        .as_ref()
        .and_then(|event| event.reference_id.as_deref());
    let Some(parent_process) = process_by_id.get(owner_process_id.as_str()).copied() else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "validate_transaction_error_missing_parent_process",
        });
    };

    let has_matching_boundary = parent_process.nodes.iter().any(|node| {
        node.kind == BpmnNodeKind::BoundaryEvent
            && node.attached_to_ref.as_deref() == Some(owner_node_id.as_str())
            && node.cancel_activity
            && node.event.as_ref().is_some_and(|event| {
                event.kind == BpmnEventKind::Error
                    && error_boundary_matches(thrown_reference_id, event.reference_id.as_deref())
            })
    });
    if has_matching_boundary {
        return Ok(());
    }

    Err(BpmnEngineError::UnsupportedTransactionConfiguration {
        process_id: owner_process_id.clone(),
        node_id: owner_node_id.clone(),
        detail: "transaction_error_missing_boundary",
    })
}

fn error_boundary_matches(
    thrown_reference_id: Option<&str>,
    boundary_reference_id: Option<&str>,
) -> bool {
    match boundary_reference_id {
        None => true,
        Some(boundary_reference_id) => thrown_reference_id == Some(boundary_reference_id),
    }
}

fn validate_node_event_shape(
    process: &super::import::RawProcess,
    node: &super::import::RawNode,
) -> Result<()> {
    if matches!(
        node.kind,
        BpmnNodeKind::IntermediateCatchEvent | BpmnNodeKind::BoundaryEvent
    ) && node.event.is_none()
    {
        return Err(BpmnEngineError::MissingRequiredNodeElement {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            element: "event_definition",
        });
    }

    if let Some(event) = &node.event
        && event.kind == BpmnEventKind::Timer
        && event
            .timer
            .as_ref()
            .is_none_or(|timer| timer.expression.trim().is_empty())
    {
        return Err(BpmnEngineError::MissingRequiredNodeElement {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            element: "timer_expression",
        });
    }

    Ok(())
}

fn validate_called_process_reference(
    process: &super::import::RawProcess,
    node: &super::import::RawNode,
    all_process_ids: &HashSet<&str>,
) -> Result<()> {
    let called_process_id = node.called_process_ref.as_deref().ok_or_else(|| {
        BpmnEngineError::MissingRequiredNodeElement {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            element: "called_process",
        }
    })?;
    if all_process_ids.contains(called_process_id) {
        return Ok(());
    }
    Err(BpmnEngineError::UnknownCalledProcess {
        process_id: process.process_id.clone(),
        node_id: node.bpmn_id.clone(),
        called_process_id: called_process_id.to_string(),
    })
}

fn validate_boundary_event(
    process: &RawProcess,
    node: &super::import::RawNode,
    node_ids: &HashSet<&str>,
    boundary_attachments: &mut HashMap<String, BoundaryAttachmentUsage>,
) -> Result<()> {
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
    if !node.cancel_activity {
        return Err(BpmnEngineError::UnsupportedBoundaryEventConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "non_interrupting_boundary_event",
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
    let event_kind = node.event.as_ref().map(|event| &event.kind);
    let usage = boundary_attachments
        .entry(attached_to_ref.to_string())
        .or_default();
    let is_transaction_shell = attached_node.kind == BpmnNodeKind::SubProcess
        && attached_node.subprocess_kind == Some(RawSubProcessKind::Transaction);

    if is_transaction_shell {
        usage.total_count += 1;
        if event_kind == Some(&BpmnEventKind::Cancel) {
            if usage.cancel_count > 0 {
                return Err(BpmnEngineError::UnsupportedBoundaryEventConfiguration {
                    process_id: process.process_id.clone(),
                    node_id: node.bpmn_id.clone(),
                    detail: "multiple_transaction_cancel_boundaries",
                });
            }
            usage.cancel_count += 1;
            return Ok(());
        }
        if event_kind == Some(&BpmnEventKind::Error) {
            return Ok(());
        }
    } else {
        if usage.total_count > 0 {
            return Err(BpmnEngineError::UnsupportedBoundaryEventConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "multiple_boundary_events_for_attached_node",
            });
        }
        usage.total_count += 1;
    }

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
            detail: "error_boundary_requires_transaction_shell",
        });
    }
    if event_kind == Some(&BpmnEventKind::Compensation) {
        return Ok(());
    }
    if !matches!(
        attached_node.kind,
        BpmnNodeKind::ServiceTask
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
    if event_kind != Some(&BpmnEventKind::Timer) {
        return Err(BpmnEngineError::UnsupportedBoundaryEventConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "unsupported_boundary_event_kind",
        });
    }

    Ok(())
}

fn validate_compensation_handlers(process: &super::import::RawProcess) -> Result<()> {
    let node_by_id = process
        .nodes
        .iter()
        .map(|node| (node.bpmn_id.as_str(), node))
        .collect::<HashMap<_, _>>();
    let compensation_boundaries = collect_compensation_boundaries(process);
    if !has_compensation_shape(process, &compensation_boundaries) {
        return Ok(());
    }

    ensure_compensation_transaction_scope(process, &compensation_boundaries)?;

    let mut seen_compensated_activities = HashSet::new();
    let mut seen_compensation_handlers = HashSet::new();
    for boundary in compensation_boundaries {
        validate_compensation_boundary(
            process,
            &node_by_id,
            boundary,
            &mut seen_compensated_activities,
            &mut seen_compensation_handlers,
        )?;
    }

    validate_orphan_compensation_handlers(process)?;

    Ok(())
}

fn collect_compensation_boundaries(process: &RawProcess) -> Vec<&RawNode> {
    process
        .nodes
        .iter()
        .filter(|node| {
            node.kind == BpmnNodeKind::BoundaryEvent
                && node
                    .event
                    .as_ref()
                    .is_some_and(|event| event.kind == BpmnEventKind::Compensation)
        })
        .collect()
}

fn has_compensation_shape(process: &RawProcess, compensation_boundaries: &[&RawNode]) -> bool {
    !compensation_boundaries.is_empty() || process.nodes.iter().any(|node| node.is_for_compensation)
}

fn ensure_compensation_transaction_scope(
    process: &RawProcess,
    compensation_boundaries: &[&RawNode],
) -> Result<()> {
    if matches!(
        process.scope,
        RawProcessScope::NestedShell {
            kind: NestedShellKind::Transaction,
            ..
        }
    ) {
        return Ok(());
    }

    let node_id = first_compensation_shape_node_id(process, compensation_boundaries);
    Err(compensation_error(
        process,
        node_id.as_str(),
        "compensation_requires_transaction_shell",
    ))
}

fn first_compensation_shape_node_id(
    process: &RawProcess,
    compensation_boundaries: &[&RawNode],
) -> String {
    compensation_boundaries
        .first()
        .map(|node| node.bpmn_id.clone())
        .or_else(|| {
            process
                .nodes
                .iter()
                .find(|node| node.is_for_compensation)
                .map(|node| node.bpmn_id.clone())
        })
        .unwrap_or_else(|| process.process_id.clone())
}

fn validate_compensation_boundary<'a>(
    process: &RawProcess,
    node_by_id: &HashMap<&'a str, &'a RawNode>,
    boundary: &'a RawNode,
    seen_compensated_activities: &mut HashSet<&'a str>,
    seen_compensation_handlers: &mut HashSet<&'a str>,
) -> Result<()> {
    let activity = resolve_compensated_activity(process, node_by_id, boundary)?;
    validate_compensated_activity(process, boundary, activity, seen_compensated_activities)?;
    ensure_node_has_no_sequence_flows(
        process,
        boundary,
        "unsupported_compensation_boundary_routing",
    )?;
    validate_compensation_handler_binding(process, node_by_id, boundary, seen_compensation_handlers)
}

fn resolve_compensated_activity<'a>(
    process: &RawProcess,
    node_by_id: &HashMap<&'a str, &'a RawNode>,
    boundary: &'a RawNode,
) -> Result<&'a RawNode> {
    let Some(attached_to_ref) = boundary.attached_to_ref.as_deref() else {
        return Err(compensation_error(
            process,
            boundary.bpmn_id.as_str(),
            "missing_compensated_activity_attachment",
        ));
    };
    let Some(activity) = node_by_id.get(attached_to_ref).copied() else {
        return Err(BpmnEngineError::UnknownBoundaryAttachment {
            process_id: process.process_id.clone(),
            node_id: boundary.bpmn_id.clone(),
            attached_to_node_id: attached_to_ref.to_string(),
        });
    };
    Ok(activity)
}

fn validate_compensated_activity<'a>(
    process: &RawProcess,
    boundary: &RawNode,
    activity: &'a RawNode,
    seen_compensated_activities: &mut HashSet<&'a str>,
) -> Result<()> {
    ensure_single_boundary_owner(process, boundary, activity)?;
    ensure_supported_compensated_activity_kind(process, activity)?;
    if activity.is_for_compensation {
        return Err(compensation_error(
            process,
            activity.bpmn_id.as_str(),
            "compensation_boundary_attaches_to_handler",
        ));
    }
    if activity.repeat.is_some() {
        return Err(compensation_error(
            process,
            activity.bpmn_id.as_str(),
            "unsupported_compensated_activity_repeat",
        ));
    }
    if !seen_compensated_activities.insert(activity.bpmn_id.as_str()) {
        return Err(compensation_error(
            process,
            activity.bpmn_id.as_str(),
            "multiple_compensation_boundaries_for_activity",
        ));
    }
    Ok(())
}

fn ensure_single_boundary_owner(
    process: &RawProcess,
    boundary: &RawNode,
    activity: &RawNode,
) -> Result<()> {
    let Some(attached_to_ref) = boundary.attached_to_ref.as_deref() else {
        return Err(compensation_error(
            process,
            boundary.bpmn_id.as_str(),
            "missing_compensated_activity_attachment",
        ));
    };
    let boundary_owner_count = process
        .nodes
        .iter()
        .filter(|node| node.attached_to_ref.as_deref() == Some(attached_to_ref))
        .count();
    if boundary_owner_count > 1 {
        return Err(compensation_error(
            process,
            activity.bpmn_id.as_str(),
            "unsupported_compensation_multi_boundary_owner",
        ));
    }
    Ok(())
}

fn ensure_supported_compensated_activity_kind(
    process: &RawProcess,
    activity: &RawNode,
) -> Result<()> {
    if matches!(
        activity.kind,
        BpmnNodeKind::ServiceTask
            | BpmnNodeKind::UserTask
            | BpmnNodeKind::ManualTask
            | BpmnNodeKind::BusinessRuleTask
    ) {
        return Ok(());
    }
    Err(compensation_error(
        process,
        activity.bpmn_id.as_str(),
        "unsupported_compensated_activity_kind",
    ))
}

fn validate_compensation_handler_binding<'a>(
    process: &RawProcess,
    node_by_id: &HashMap<&'a str, &'a RawNode>,
    boundary: &'a RawNode,
    seen_compensation_handlers: &mut HashSet<&'a str>,
) -> Result<()> {
    let handler_associations = process
        .associations
        .iter()
        .filter(|association| association.source_ref == boundary.bpmn_id)
        .collect::<Vec<_>>();
    match handler_associations.as_slice() {
        [] => Err(compensation_error(
            process,
            boundary.bpmn_id.as_str(),
            "missing_compensation_association",
        )),
        [association] => {
            let handler = resolve_compensation_handler(process, node_by_id, association)?;
            validate_compensation_handler(process, handler, seen_compensation_handlers)
        }
        _ => Err(compensation_error(
            process,
            boundary.bpmn_id.as_str(),
            "multiple_compensation_associations",
        )),
    }
}

fn resolve_compensation_handler<'a>(
    process: &RawProcess,
    node_by_id: &HashMap<&'a str, &'a RawNode>,
    association: &super::import::RawAssociation,
) -> Result<&'a RawNode> {
    let Some(handler) = node_by_id.get(association.target_ref.as_str()).copied() else {
        return Err(BpmnEngineError::UnknownSequenceFlowEndpoint {
            process_id: process.process_id.clone(),
            flow_id: association.association_id.clone(),
            endpoint: "target",
            node_id: association.target_ref.clone(),
        });
    };
    Ok(handler)
}

fn validate_compensation_handler<'a>(
    process: &RawProcess,
    handler: &'a RawNode,
    seen_compensation_handlers: &mut HashSet<&'a str>,
) -> Result<()> {
    if !matches!(
        handler.kind,
        BpmnNodeKind::ServiceTask
            | BpmnNodeKind::UserTask
            | BpmnNodeKind::ManualTask
            | BpmnNodeKind::BusinessRuleTask
    ) {
        return Err(compensation_error(
            process,
            handler.bpmn_id.as_str(),
            "unsupported_compensation_handler_kind",
        ));
    }
    if !handler.is_for_compensation {
        return Err(compensation_error(
            process,
            handler.bpmn_id.as_str(),
            "missing_compensation_handler_marker",
        ));
    }
    if handler.repeat.is_some() {
        return Err(compensation_error(
            process,
            handler.bpmn_id.as_str(),
            "unsupported_compensation_handler_repeat",
        ));
    }
    ensure_node_has_no_sequence_flows(
        process,
        handler,
        "unsupported_compensation_handler_routing",
    )?;
    if !seen_compensation_handlers.insert(handler.bpmn_id.as_str()) {
        return Err(compensation_error(
            process,
            handler.bpmn_id.as_str(),
            "compensation_handler_reused",
        ));
    }
    Ok(())
}

fn ensure_node_has_no_sequence_flows(
    process: &RawProcess,
    node: &RawNode,
    detail: &'static str,
) -> Result<()> {
    let flow_count = process
        .flows
        .iter()
        .filter(|flow| flow.source_ref == node.bpmn_id || flow.target_ref == node.bpmn_id)
        .count();
    if flow_count > 0 {
        return Err(compensation_error(process, node.bpmn_id.as_str(), detail));
    }
    Ok(())
}

fn validate_orphan_compensation_handlers(process: &RawProcess) -> Result<()> {
    for handler in process.nodes.iter().filter(|node| node.is_for_compensation) {
        let incoming_associations = process
            .associations
            .iter()
            .filter(|association| association.target_ref == handler.bpmn_id)
            .count();
        if incoming_associations != 1 {
            return Err(compensation_error(
                process,
                handler.bpmn_id.as_str(),
                "orphan_compensation_handler",
            ));
        }
    }
    Ok(())
}

fn compensation_error(
    process: &RawProcess,
    node_id: &str,
    detail: &'static str,
) -> BpmnEngineError {
    BpmnEngineError::UnsupportedCompensationConfiguration {
        process_id: process.process_id.clone(),
        node_id: node_id.to_string(),
        detail,
    }
}

fn validate_sequence_flows(
    process: &super::import::RawProcess,
    node_ids: &HashSet<&str>,
) -> Result<()> {
    let mut seen_flow_ids = HashSet::new();
    for flow in &process.flows {
        if !seen_flow_ids.insert(flow.flow_id.as_str()) {
            return Err(BpmnEngineError::DuplicateSequenceFlowId {
                process_id: process.process_id.clone(),
                flow_id: flow.flow_id.clone(),
            });
        }
        if !node_ids.contains(flow.source_ref.as_str()) {
            return Err(BpmnEngineError::UnknownSequenceFlowEndpoint {
                process_id: process.process_id.clone(),
                flow_id: flow.flow_id.clone(),
                endpoint: "source",
                node_id: flow.source_ref.clone(),
            });
        }
        if !node_ids.contains(flow.target_ref.as_str()) {
            return Err(BpmnEngineError::UnknownSequenceFlowEndpoint {
                process_id: process.process_id.clone(),
                flow_id: flow.flow_id.clone(),
                endpoint: "target",
                node_id: flow.target_ref.clone(),
            });
        }
    }
    Ok(())
}

fn validate_standard_loops(process: &super::import::RawProcess) -> Result<()> {
    for node in &process.nodes {
        let Some(RawRepeatSpec::StandardLoop(loop_spec)) = &node.repeat else {
            continue;
        };

        if !matches!(
            node.kind,
            BpmnNodeKind::ServiceTask
                | BpmnNodeKind::UserTask
                | BpmnNodeKind::ManualTask
                | BpmnNodeKind::BusinessRuleTask
        ) {
            return Err(BpmnEngineError::UnsupportedLoopConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "unsupported_standard_loop_host_kind",
            });
        }

        let loop_condition = loop_spec
            .loop_condition
            .as_deref()
            .map(str::trim)
            .filter(|condition| !condition.is_empty());
        if loop_spec.loop_maximum.is_none() && loop_condition.is_none() {
            return Err(BpmnEngineError::UnsupportedLoopConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "missing_loop_maximum_or_condition",
            });
        }

        if loop_spec.loop_maximum == Some(0) {
            return Err(BpmnEngineError::UnsupportedLoopConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "non_positive_loop_maximum",
            });
        }

        if let Some(loop_condition) = loop_condition
            && !is_supported_standard_loop_condition(loop_condition)
        {
            return Err(BpmnEngineError::UnsupportedLoopConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "unsupported_loop_condition_expression",
            });
        }
    }

    Ok(())
}

fn validate_multi_instances(process: &super::import::RawProcess) -> Result<()> {
    for node in &process.nodes {
        let Some(shape) = multi_instance_validation_shape(node) else {
            continue;
        };

        if !matches!(
            node.kind,
            BpmnNodeKind::ServiceTask
                | BpmnNodeKind::UserTask
                | BpmnNodeKind::ManualTask
                | BpmnNodeKind::BusinessRuleTask
        ) {
            return Err(BpmnEngineError::UnsupportedLoopConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "unsupported_multi_instance_host_kind",
            });
        }

        validate_multi_instance_expansion(process, node, &shape)?;

        if let Some(completion_condition) = shape.completion_condition
            && !is_supported_multi_instance_completion_condition(completion_condition)
        {
            return Err(BpmnEngineError::UnsupportedLoopConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "unsupported_multi_instance_completion_condition_expression",
            });
        }
    }

    Ok(())
}

struct MultiInstanceValidationShape<'a> {
    loop_cardinality: Option<u32>,
    loop_data_input_ref: Option<&'a str>,
    input_data_item: Option<&'a str>,
    loop_data_output_ref: Option<&'a str>,
    output_data_item: Option<&'a str>,
    completion_condition: Option<&'a str>,
}

fn multi_instance_validation_shape(
    node: &super::import::RawNode,
) -> Option<MultiInstanceValidationShape<'_>> {
    match &node.repeat {
        Some(RawRepeatSpec::SequentialMultiInstance(multi_instance_spec)) => {
            Some(MultiInstanceValidationShape {
                loop_cardinality: multi_instance_spec.loop_cardinality,
                loop_data_input_ref: multi_instance_spec.loop_data_input_ref.as_deref(),
                input_data_item: multi_instance_spec.input_data_item.as_deref(),
                loop_data_output_ref: multi_instance_spec.loop_data_output_ref.as_deref(),
                output_data_item: multi_instance_spec.output_data_item.as_deref(),
                completion_condition: multi_instance_spec.completion_condition.as_deref(),
            })
        }
        Some(RawRepeatSpec::ParallelMultiInstance(multi_instance_spec)) => {
            Some(MultiInstanceValidationShape {
                loop_cardinality: multi_instance_spec.loop_cardinality,
                loop_data_input_ref: multi_instance_spec.loop_data_input_ref.as_deref(),
                input_data_item: multi_instance_spec.input_data_item.as_deref(),
                loop_data_output_ref: multi_instance_spec.loop_data_output_ref.as_deref(),
                output_data_item: multi_instance_spec.output_data_item.as_deref(),
                completion_condition: multi_instance_spec.completion_condition.as_deref(),
            })
        }
        _ => None,
    }
}

fn validate_multi_instance_expansion(
    process: &super::import::RawProcess,
    node: &super::import::RawNode,
    shape: &MultiInstanceValidationShape<'_>,
) -> Result<()> {
    if shape.loop_cardinality.is_some() && shape.loop_data_input_ref.is_some() {
        return Err(BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "mixed_multi_instance_expansion",
        });
    }
    if shape.loop_cardinality.is_none() && shape.loop_data_input_ref.is_none() {
        return Err(BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "missing_loop_cardinality_or_data_input",
        });
    }
    if shape.loop_data_input_ref.is_some() && shape.input_data_item.is_none() {
        return Err(BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "missing_input_data_item",
        });
    }
    if shape.loop_data_input_ref.is_none() && shape.input_data_item.is_some() {
        return Err(BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "missing_loop_data_input_ref",
        });
    }
    if shape.loop_data_output_ref.is_some() && shape.output_data_item.is_none() {
        return Err(BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "missing_output_data_item",
        });
    }
    if shape.loop_data_output_ref.is_none() && shape.output_data_item.is_some() {
        return Err(BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "missing_loop_data_output_ref",
        });
    }
    if let (Some(loop_data_input_ref), Some(loop_data_output_ref)) =
        (shape.loop_data_input_ref, shape.loop_data_output_ref)
        && loop_data_input_ref == loop_data_output_ref
    {
        return Err(BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "unsupported_multi_instance_in_place_output",
        });
    }
    Ok(())
}

fn validate_event_based_gateways(
    process: &super::import::RawProcess,
    node_ids: &HashSet<&str>,
) -> Result<()> {
    let node_by_id = process
        .nodes
        .iter()
        .map(|node| (node.bpmn_id.as_str(), node))
        .collect::<HashMap<_, _>>();

    for gateway in process.nodes.iter().filter(|node| {
        node.kind == BpmnNodeKind::Gateway && node.gateway_kind == Some(BpmnGatewayKind::EventBased)
    }) {
        let outgoing_targets = process
            .flows
            .iter()
            .filter(|flow| flow.source_ref == gateway.bpmn_id)
            .map(|flow| flow.target_ref.as_str())
            .collect::<Vec<_>>();

        if outgoing_targets.len() < 2 {
            return Err(BpmnEngineError::UnsupportedEventBasedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: gateway.bpmn_id.clone(),
                detail: "insufficient_outgoing_waits",
            });
        }

        for target_id in outgoing_targets {
            if !node_ids.contains::<str>(target_id) {
                continue;
            }

            let Some(target) = node_by_id.get(target_id) else {
                return Err(BpmnEngineError::UnknownSequenceFlowEndpoint {
                    process_id: process.process_id.clone(),
                    flow_id: gateway.bpmn_id.clone(),
                    endpoint: "target",
                    node_id: target_id.to_string(),
                });
            };
            if target.kind != BpmnNodeKind::IntermediateCatchEvent {
                return Err(BpmnEngineError::UnsupportedEventBasedGatewayConfiguration {
                    process_id: process.process_id.clone(),
                    node_id: gateway.bpmn_id.clone(),
                    detail: "unsupported_wait_target_kind",
                });
            }

            let Some(event) = target.event.as_ref() else {
                return Err(BpmnEngineError::MissingRequiredNodeElement {
                    process_id: process.process_id.clone(),
                    node_id: target.bpmn_id.clone(),
                    element: "event_definition",
                });
            };
            if !matches!(
                event.kind,
                BpmnEventKind::Message | BpmnEventKind::Signal | BpmnEventKind::Timer
            ) {
                return Err(BpmnEngineError::UnsupportedEventBasedGatewayConfiguration {
                    process_id: process.process_id.clone(),
                    node_id: gateway.bpmn_id.clone(),
                    detail: "unsupported_wait_event_kind",
                });
            }
        }
    }

    Ok(())
}

fn detect_recursive_call_activity(raw: &RawPackageDocument) -> Result<()> {
    let call_graph = raw
        .processes
        .iter()
        .map(|process| {
            let edges = process
                .nodes
                .iter()
                .filter(|node| node.kind == BpmnNodeKind::SubProcess)
                .filter_map(|node| {
                    node.called_process_ref
                        .clone()
                        .map(|called_process_id| (called_process_id, node.bpmn_id.clone()))
                })
                .collect::<Vec<_>>();
            (process.process_id.clone(), edges)
        })
        .collect::<std::collections::HashMap<_, _>>();

    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    for process_id in call_graph.keys() {
        if let Some(error) = detect_recursive_call_activity_from(
            process_id,
            &call_graph,
            &mut visiting,
            &mut visited,
        ) {
            return Err(error);
        }
    }
    Ok(())
}

fn detect_recursive_call_activity_from(
    process_id: &str,
    call_graph: &std::collections::HashMap<String, Vec<(String, String)>>,
    visiting: &mut HashSet<String>,
    visited: &mut HashSet<String>,
) -> Option<BpmnEngineError> {
    if visited.contains(process_id) {
        return None;
    }

    visiting.insert(process_id.to_string());
    if let Some(edges) = call_graph.get(process_id) {
        for (called_process_id, node_id) in edges {
            if visiting.contains(called_process_id) {
                return Some(BpmnEngineError::UnsupportedSubProcessConfiguration {
                    process_id: process_id.to_string(),
                    node_id: node_id.clone(),
                    detail: "recursive_call_activity",
                });
            }
            if let Some(error) = detect_recursive_call_activity_from(
                called_process_id,
                call_graph,
                visiting,
                visited,
            ) {
                return Some(error);
            }
        }
    }
    visiting.remove(process_id);
    visited.insert(process_id.to_string());
    None
}

fn is_supported_standard_loop_condition(condition: &str) -> bool {
    let trimmed = condition.trim();
    let path = trimmed.strip_prefix("not ").map_or(trimmed, str::trim);
    !path.is_empty() && path.split('.').all(is_identifier_segment)
}

fn is_identifier_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}
