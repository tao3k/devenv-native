use crate::error::{BpmnEngineError, Result};
use crate::ir_event_api::BpmnEventKind;
use crate::ir_node_api::BpmnNodeKind;
use crate::parser::import::{
    NestedShellKind, RawAssociation, RawNode, RawProcess, RawProcessScope,
};
use std::collections::{HashMap, HashSet};

pub(super) fn validate_compensation_handlers(process: &RawProcess) -> Result<()> {
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
            process_id: (process.process_id.clone()).into(),
            node_id: (boundary.bpmn_id.clone()).into(),
            attached_to_node_id: (attached_to_ref.to_string()).into(),
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
            process_id: (process.process_id.clone()).into(),
            flow_id: (association.association_id.clone()).into(),
            endpoint: "target",
            node_id: (association.target_ref.clone()).into(),
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
        process_id: (process.process_id.clone()).into(),
        node_id: (node_id.to_string()).into(),
        detail,
    }
}
