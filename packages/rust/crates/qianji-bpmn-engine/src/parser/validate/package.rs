//! BPMN validation for the bounded supported subset.

use super::boundary::validate_boundary_event;
use super::error_paths::{
    CallActivityOwner, collect_call_activity_owners, validate_supported_error_end_paths,
};
use super::escalation_paths::validate_supported_escalation_end_paths;
use super::recursion::detect_recursive_call_activity;
use crate::error::{BpmnEngineError, Result};
use crate::ir_event_api::BpmnEventKind;
use crate::ir_node_api::{BpmnGatewayKind, BpmnNodeKind};
use crate::parser::import::{
    NestedShellKind, RawAssociation, RawNode, RawPackageDocument, RawProcess, RawProcessScope,
    RawRepeatSpec, RawSequenceFlow,
};
use crate::repeat_condition::{
    is_supported_gateway_condition, is_supported_multi_instance_completion_condition,
};
use std::collections::{HashMap, HashSet};

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
    let call_activity_owners = collect_call_activity_owners(raw);
    let mut seen_process_ids = HashSet::new();
    for process in &raw.processes {
        ensure_unique_process_id(raw, process, &mut seen_process_ids)?;
        let node_ids = collect_node_ids(process)?;
        validate_process_topology(
            process,
            &all_process_ids,
            &node_ids,
            &process_by_id,
            &call_activity_owners,
        )?;
        validate_sequence_flows(process, &node_ids)?;
        validate_standard_loops(process)?;
        validate_multi_instances(process)?;
        validate_compensation_handlers(process)?;
        validate_task_routing(process)?;
        validate_gateways(process)?;
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
    process: &'a RawProcess,
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

fn collect_node_ids(process: &RawProcess) -> Result<HashSet<&str>> {
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
    call_activity_owners: &HashMap<&str, Vec<CallActivityOwner<'_>>>,
) -> Result<()> {
    let mut boundary_attachments = HashMap::new();
    validate_transaction_cancel_path(process, process_by_id)?;
    validate_supported_error_end_paths(process, process_by_id, call_activity_owners)?;
    validate_supported_escalation_end_paths(process, process_by_id, call_activity_owners)?;
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

fn validate_node_event_shape(process: &RawProcess, node: &RawNode) -> Result<()> {
    if matches!(
        node.kind,
        BpmnNodeKind::IntermediateThrowEvent
            | BpmnNodeKind::IntermediateCatchEvent
            | BpmnNodeKind::BoundaryEvent
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

    if let Some(event) = &node.event
        && event.kind == BpmnEventKind::Conditional
    {
        let Some(condition_expression) = event.condition_expression.as_deref() else {
            return Err(BpmnEngineError::MissingRequiredNodeElement {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                element: "conditional_expression",
            });
        };
        if !is_supported_gateway_condition(condition_expression) {
            return Err(BpmnEngineError::UnsupportedEventConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "unsupported_conditional_event_expression",
            });
        }
    }

    validate_message_task_shape(process, node)?;

    Ok(())
}

fn validate_message_task_shape(process: &RawProcess, node: &RawNode) -> Result<()> {
    let expected_detail = match node.kind {
        BpmnNodeKind::SendTask => Some("unsupported_send_task_event_kind"),
        BpmnNodeKind::ReceiveTask => Some("unsupported_receive_task_event_kind"),
        _ => None,
    };
    let Some(expected_detail) = expected_detail else {
        return Ok(());
    };

    if node.task_message_ref.is_some() && node.event.is_some() {
        return Err(BpmnEngineError::UnsupportedTaskConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "multiple_task_message_bindings",
        });
    }

    let attribute_binding = node
        .task_message_ref
        .as_deref()
        .map(str::trim)
        .filter(|binding| !binding.is_empty());
    if let Some(event) = &node.event {
        if event.kind != BpmnEventKind::Message {
            return Err(BpmnEngineError::UnsupportedTaskConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: expected_detail,
            });
        }
        let event_binding = event
            .reference_id
            .as_deref()
            .map(str::trim)
            .filter(|binding| !binding.is_empty());
        if event_binding.is_none() {
            return Err(BpmnEngineError::MissingRequiredNodeElement {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                element: "message_binding",
            });
        }
        return Ok(());
    }

    if attribute_binding.is_some() {
        return Ok(());
    }

    Err(BpmnEngineError::MissingRequiredNodeElement {
        process_id: process.process_id.clone(),
        node_id: node.bpmn_id.clone(),
        element: "message_binding",
    })
}

fn validate_called_process_reference(
    process: &RawProcess,
    node: &RawNode,
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

fn validate_compensation_handlers(process: &RawProcess) -> Result<()> {
    let node_by_id = process
        .nodes
        .iter()
        .map(|node| (node.bpmn_id.as_str(), node))
        .collect::<HashMap<_, _>>();
    let compensation_boundaries = collect_compensation_boundaries(process);
    let throw_compensation_nodes = collect_throw_compensation_nodes(process);
    if !has_compensation_shape(process, &compensation_boundaries, &throw_compensation_nodes) {
        return Ok(());
    }

    ensure_compensation_transaction_scope(
        process,
        &compensation_boundaries,
        &throw_compensation_nodes,
    )?;

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
    for throw_node in throw_compensation_nodes {
        validate_throw_compensation_node(process, &node_by_id, throw_node)?;
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

fn collect_throw_compensation_nodes(process: &RawProcess) -> Vec<&RawNode> {
    process
        .nodes
        .iter()
        .filter(|node| {
            matches!(
                node.kind,
                BpmnNodeKind::EndEvent | BpmnNodeKind::IntermediateThrowEvent
            ) && node
                .event
                .as_ref()
                .is_some_and(|event| event.kind == BpmnEventKind::Compensation)
        })
        .collect()
}

fn has_compensation_shape(
    process: &RawProcess,
    compensation_boundaries: &[&RawNode],
    throw_compensation_nodes: &[&RawNode],
) -> bool {
    !compensation_boundaries.is_empty()
        || !throw_compensation_nodes.is_empty()
        || process.nodes.iter().any(|node| node.is_for_compensation)
}

fn ensure_compensation_transaction_scope(
    process: &RawProcess,
    compensation_boundaries: &[&RawNode],
    throw_compensation_nodes: &[&RawNode],
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

    let node_id = first_compensation_shape_node_id(
        process,
        compensation_boundaries,
        throw_compensation_nodes,
    );
    Err(compensation_error(
        process,
        node_id.as_str(),
        "compensation_requires_transaction_shell",
    ))
}

fn first_compensation_shape_node_id(
    process: &RawProcess,
    compensation_boundaries: &[&RawNode],
    throw_compensation_nodes: &[&RawNode],
) -> String {
    compensation_boundaries
        .first()
        .map(|node| node.bpmn_id.clone())
        .or_else(|| {
            throw_compensation_nodes
                .first()
                .map(|node| node.bpmn_id.clone())
        })
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

fn validate_throw_compensation_node<'a>(
    process: &RawProcess,
    node_by_id: &HashMap<&'a str, &'a RawNode>,
    throw_node: &'a RawNode,
) -> Result<()> {
    let Some(target_activity_id) = throw_node
        .event
        .as_ref()
        .and_then(|event| event.reference_id.as_deref())
    else {
        if matches!(
            throw_node.kind,
            BpmnNodeKind::EndEvent | BpmnNodeKind::IntermediateThrowEvent
        ) {
            return Ok(());
        }
        return Err(compensation_error(
            process,
            throw_node.bpmn_id.as_str(),
            "missing_throw_compensation_target",
        ));
    };
    let Some(activity) = node_by_id.get(target_activity_id).copied() else {
        return Err(compensation_error(
            process,
            throw_node.bpmn_id.as_str(),
            "unknown_throw_compensation_target",
        ));
    };

    ensure_supported_compensated_activity_kind(process, activity)?;
    if activity.is_for_compensation {
        return Err(compensation_error(
            process,
            activity.bpmn_id.as_str(),
            "throw_compensation_targets_handler",
        ));
    }
    if activity.repeat.is_some() {
        return Err(compensation_error(
            process,
            activity.bpmn_id.as_str(),
            "unsupported_compensated_activity_repeat",
        ));
    }

    let has_boundary_handler = process.nodes.iter().any(|node| {
        node.kind == BpmnNodeKind::BoundaryEvent
            && node.attached_to_ref.as_deref() == Some(target_activity_id)
            && node
                .event
                .as_ref()
                .is_some_and(|event| event.kind == BpmnEventKind::Compensation)
    });
    if has_boundary_handler {
        return Ok(());
    }

    Err(compensation_error(
        process,
        throw_node.bpmn_id.as_str(),
        "throw_compensation_target_without_handler",
    ))
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
            | BpmnNodeKind::ScriptTask
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
    association: &RawAssociation,
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
            | BpmnNodeKind::ScriptTask
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

fn validate_sequence_flows(process: &RawProcess, node_ids: &HashSet<&str>) -> Result<()> {
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

fn validate_task_routing(process: &RawProcess) -> Result<()> {
    let mut outgoing_counts = HashMap::<&str, usize>::new();
    for flow in &process.flows {
        *outgoing_counts.entry(flow.source_ref.as_str()).or_default() += 1;
    }

    for node in &process.nodes {
        if node.is_for_compensation || !requires_single_outgoing_task_route(&node.kind) {
            continue;
        }
        let outgoing_count = outgoing_counts
            .get(node.bpmn_id.as_str())
            .copied()
            .unwrap_or_default();
        if outgoing_count != 1 {
            return Err(BpmnEngineError::UnsupportedTaskConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "task_requires_single_outgoing",
            });
        }
    }
    Ok(())
}

fn requires_single_outgoing_task_route(kind: &BpmnNodeKind) -> bool {
    matches!(
        kind,
        BpmnNodeKind::ServiceTask
            | BpmnNodeKind::ScriptTask
            | BpmnNodeKind::UserTask
            | BpmnNodeKind::ManualTask
            | BpmnNodeKind::BusinessRuleTask
            | BpmnNodeKind::SendTask
            | BpmnNodeKind::ReceiveTask
    )
}

fn validate_gateways(process: &RawProcess) -> Result<()> {
    let flow_by_id = process
        .flows
        .iter()
        .map(|flow| (flow.flow_id.as_str(), flow))
        .collect::<HashMap<_, _>>();

    for node in &process.nodes {
        let outgoing = process
            .flows
            .iter()
            .filter(|flow| flow.source_ref == node.bpmn_id)
            .collect::<Vec<_>>();

        for flow in &outgoing {
            if flow.condition_expression.is_none() {
                continue;
            }
            if !matches!(
                node.gateway_kind,
                Some(BpmnGatewayKind::Exclusive | BpmnGatewayKind::Inclusive)
            ) {
                return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                    process_id: process.process_id.clone(),
                    node_id: node.bpmn_id.clone(),
                    detail: "condition_expression_requires_conditional_gateway",
                });
            }
        }

        if !matches!(
            node.gateway_kind,
            Some(BpmnGatewayKind::Exclusive | BpmnGatewayKind::Inclusive)
        ) {
            continue;
        }

        validate_conditional_gateway_defaults_and_conditions(
            process,
            node,
            &outgoing,
            &flow_by_id,
        )?;

        if node.gateway_kind == Some(BpmnGatewayKind::Inclusive) {
            validate_structured_inclusive_gateway(process, node, &outgoing)?;
        }
    }

    Ok(())
}

fn validate_conditional_gateway_defaults_and_conditions<'a>(
    process: &RawProcess,
    node: &RawNode,
    outgoing: &[&'a RawSequenceFlow],
    flow_by_id: &HashMap<&'a str, &'a RawSequenceFlow>,
) -> Result<()> {
    if outgoing.len() <= 1 {
        if node.default_flow_ref.is_some() {
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "default_flow_requires_multiple_outgoing",
            });
        }
        return Ok(());
    }

    let default_flow_id = node.default_flow_ref.as_deref();
    if let Some(default_flow_id) = default_flow_id {
        let Some(default_flow) = flow_by_id.get(default_flow_id).copied() else {
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "unknown_default_flow",
            });
        };
        if default_flow.source_ref != node.bpmn_id {
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "default_flow_not_outgoing",
            });
        }
        if default_flow.condition_expression.is_some() {
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "default_flow_must_not_have_condition_expression",
            });
        }
    }

    for flow in outgoing {
        if Some(flow.flow_id.as_str()) == default_flow_id {
            continue;
        }
        let Some(condition_expression) = flow.condition_expression.as_deref() else {
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "missing_condition_expression",
            });
        };
        if !is_supported_gateway_condition(condition_expression) {
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "unsupported_condition_expression",
            });
        }
    }

    Ok(())
}

fn validate_structured_inclusive_gateway(
    process: &RawProcess,
    node: &RawNode,
    outgoing: &[&RawSequenceFlow],
) -> Result<()> {
    let incoming_len = process
        .flows
        .iter()
        .filter(|flow| flow.target_ref == node.bpmn_id)
        .count();

    if incoming_len == 1 && outgoing.len() > 1 {
        let _ = resolve_structured_inclusive_join(process, node)?;
        return Ok(());
    }

    if incoming_len > 1 && outgoing.len() == 1 {
        if node.default_flow_ref.is_some() {
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "inclusive_join_default_not_supported",
            });
        }
        if outgoing[0].condition_expression.is_some() {
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "inclusive_join_condition_expression_not_supported",
            });
        }
        return Ok(());
    }

    Err(BpmnEngineError::UnsupportedGatewayConfiguration {
        process_id: process.process_id.clone(),
        node_id: node.bpmn_id.clone(),
        detail: "inclusive_gateway_requires_structured_split_or_join",
    })
}

pub(crate) fn resolve_structured_inclusive_join(
    process: &RawProcess,
    node: &RawNode,
) -> Result<Option<String>> {
    if node.gateway_kind != Some(BpmnGatewayKind::Inclusive) {
        return Ok(None);
    }

    let outgoing = process
        .flows
        .iter()
        .filter(|flow| flow.source_ref == node.bpmn_id)
        .collect::<Vec<_>>();
    let incoming_len = process
        .flows
        .iter()
        .filter(|flow| flow.target_ref == node.bpmn_id)
        .count();

    if !(incoming_len == 1 && outgoing.len() > 1) {
        return Ok(None);
    }

    let flow_by_source = process.flows.iter().fold(
        HashMap::<&str, Vec<&RawSequenceFlow>>::new(),
        |mut acc, flow| {
            acc.entry(flow.source_ref.as_str()).or_default().push(flow);
            acc
        },
    );
    let node_by_id = process
        .nodes
        .iter()
        .map(|raw_node| (raw_node.bpmn_id.as_str(), raw_node))
        .collect::<HashMap<_, _>>();

    let mut join_node_id = None::<String>;
    let mut seen_join_inputs = HashSet::new();
    for flow in outgoing {
        let (branch_join_node_id, join_input_flow_id) =
            trace_inclusive_branch_to_join(process, node, flow, &node_by_id, &flow_by_source)?;
        if !seen_join_inputs.insert(join_input_flow_id.clone()) {
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "inclusive_split_branch_duplicate_join_input",
            });
        }
        match &join_node_id {
            Some(expected_join_node_id) if expected_join_node_id != &branch_join_node_id => {
                return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                    process_id: process.process_id.clone(),
                    node_id: node.bpmn_id.clone(),
                    detail: "inclusive_split_branch_mismatched_join",
                });
            }
            None => join_node_id = Some(branch_join_node_id),
            Some(_) => {}
        }
    }

    join_node_id
        .ok_or_else(|| BpmnEngineError::UnsupportedGatewayConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "inclusive_split_missing_join",
        })
        .map(Some)
}

fn trace_inclusive_branch_to_join<'a>(
    process: &RawProcess,
    split_node: &RawNode,
    initial_flow: &'a RawSequenceFlow,
    node_by_id: &HashMap<&'a str, &'a RawNode>,
    flow_by_source: &HashMap<&'a str, Vec<&'a RawSequenceFlow>>,
) -> Result<(String, String)> {
    let mut current_flow = initial_flow;
    let mut visited_nodes = HashSet::new();

    loop {
        let Some(current_node) = node_by_id.get(current_flow.target_ref.as_str()).copied() else {
            return Err(BpmnEngineError::UnknownSequenceFlowEndpoint {
                process_id: process.process_id.clone(),
                flow_id: current_flow.flow_id.clone(),
                endpoint: "target",
                node_id: current_flow.target_ref.clone(),
            });
        };

        if !visited_nodes.insert(current_node.bpmn_id.as_str()) {
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: split_node.bpmn_id.clone(),
                detail: "inclusive_split_branch_not_linear",
            });
        }

        if current_node.gateway_kind == Some(BpmnGatewayKind::Inclusive) {
            let incoming_len = process
                .flows
                .iter()
                .filter(|flow| flow.target_ref == current_node.bpmn_id)
                .count();
            let outgoing_len = flow_by_source
                .get(current_node.bpmn_id.as_str())
                .map_or(0, Vec::len);
            if incoming_len > 1 && outgoing_len == 1 {
                return Ok((current_node.bpmn_id.clone(), current_flow.flow_id.clone()));
            }
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: split_node.bpmn_id.clone(),
                detail: "inclusive_split_branch_unsupported_gateway",
            });
        }

        if current_node.kind == BpmnNodeKind::Gateway {
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: split_node.bpmn_id.clone(),
                detail: "inclusive_split_branch_unsupported_gateway",
            });
        }

        let outgoing = flow_by_source
            .get(current_node.bpmn_id.as_str())
            .cloned()
            .unwrap_or_default();
        match outgoing.as_slice() {
            [] => {
                return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                    process_id: process.process_id.clone(),
                    node_id: split_node.bpmn_id.clone(),
                    detail: "inclusive_split_branch_ends_before_join",
                });
            }
            [next_flow] => {
                current_flow = next_flow;
            }
            _ => {
                return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                    process_id: process.process_id.clone(),
                    node_id: split_node.bpmn_id.clone(),
                    detail: "inclusive_split_branch_not_linear",
                });
            }
        }
    }
}

fn validate_standard_loops(process: &RawProcess) -> Result<()> {
    for node in &process.nodes {
        let Some(RawRepeatSpec::StandardLoop(loop_spec)) = &node.repeat else {
            continue;
        };

        if !matches!(
            node.kind,
            BpmnNodeKind::ServiceTask
                | BpmnNodeKind::ScriptTask
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

fn validate_multi_instances(process: &RawProcess) -> Result<()> {
    for node in &process.nodes {
        let Some(shape) = multi_instance_validation_shape(node) else {
            continue;
        };

        if !matches!(
            node.kind,
            BpmnNodeKind::ServiceTask
                | BpmnNodeKind::ScriptTask
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

fn multi_instance_validation_shape(node: &RawNode) -> Option<MultiInstanceValidationShape<'_>> {
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
    process: &RawProcess,
    node: &RawNode,
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

fn validate_event_based_gateways(process: &RawProcess, node_ids: &HashSet<&str>) -> Result<()> {
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
