use super::scope::*;
use super::state;

pub(super) fn cancel_transaction_shell(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<()> {
    state::set_node_status(instance, current_node_index, NodeRuntimeStatus::Completed);
    let _ = state::remove_active_token(instance, current_token_index);

    let frame = pop_call_activity_frame(instance).ok_or(BpmnEngineError::UnsupportedOperation {
        operation: "cancel_transaction_shell_missing_parent_frame",
    })?;
    let transaction_cancel_variables = frame.transaction_cancel_variables.clone();
    let return_node_index = restore_call_activity_frame(instance, frame);
    let Some(transaction_cancel_variables) = transaction_cancel_variables else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "cancel_transaction_shell_missing_variable_snapshot",
        });
    };
    instance.variables = transaction_cancel_variables;

    let process = resolve_process_for_instance(package, instance)?;
    let transaction_node = process.nodes.get(return_node_index as usize).ok_or(
        BpmnEngineError::UnsupportedOperation {
            operation: "cancel_transaction_shell_missing_parent_node",
        },
    )?;
    if transaction_node.subprocess_kind != Some(BpmnSubProcessKind::Transaction) {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "cancel_transaction_shell_parent_not_transaction",
        });
    }

    let Some(boundary) = process
        .boundary_events_for_attached_node(return_node_index)
        .find(|boundary| {
            process
                .event_for_node(boundary.index)
                .is_some_and(|event| event.kind == BpmnEventKind::Cancel)
        })
    else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "cancel_transaction_shell_missing_boundary",
        });
    };

    let parent_token_index = state::token_index_for_node(instance, return_node_index).ok_or(
        BpmnEngineError::UnsupportedOperation {
            operation: "cancel_transaction_shell_missing_parent_token",
        },
    )?;
    state::set_node_status(instance, return_node_index, NodeRuntimeStatus::Cancelled);
    cancel_transaction_boundary_siblings(process, instance, return_node_index, &[boundary.index])?;
    state::set_node_status(instance, boundary.index, NodeRuntimeStatus::Completed);
    let edge_index = state::resolve_single_outgoing_edge(
        process,
        boundary.index,
        "cancel_transaction_shell_routing",
    )?;
    let next_node_index = process.edges[edge_index as usize].to;
    state::set_active_node_index(instance, parent_token_index, edge_index, next_node_index);
    state::set_node_status(instance, next_node_index, NodeRuntimeStatus::Queued);
    state::record_transition(instance, now_ms, InstanceLifecycle::Running);
    Ok(())
}

pub(super) fn error_transaction_shell(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    thrown_reference_id: Option<&str>,
    now_ms: u64,
) -> Result<()> {
    state::set_node_status(instance, current_node_index, NodeRuntimeStatus::Completed);
    let _ = state::remove_active_token(instance, current_token_index);

    let frame = pop_call_activity_frame(instance).ok_or(BpmnEngineError::UnsupportedOperation {
        operation: "error_transaction_shell_missing_parent_frame",
    })?;
    let return_node_index = restore_call_activity_frame(instance, frame);

    let process = resolve_process_for_instance(package, instance)?;
    let transaction_node = process.nodes.get(return_node_index as usize).ok_or(
        BpmnEngineError::UnsupportedOperation {
            operation: "error_transaction_shell_missing_parent_node",
        },
    )?;
    if transaction_node.subprocess_kind != Some(BpmnSubProcessKind::Transaction) {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "error_transaction_shell_parent_not_transaction",
        });
    }

    let matching_boundaries = process
        .boundary_events_for_attached_node(return_node_index)
        .filter_map(|boundary| {
            let boundary_event = process.event_for_node(boundary.index)?;
            (boundary_event.kind == BpmnEventKind::Error
                && error_reference_matches(
                    thrown_reference_id,
                    boundary_event.reference_id.as_deref(),
                ))
            .then_some(boundary.index)
        })
        .collect::<Vec<_>>();
    if matching_boundaries.is_empty() {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "error_transaction_shell_missing_boundary",
        });
    }

    let parent_token_index = state::token_index_for_node(instance, return_node_index).ok_or(
        BpmnEngineError::UnsupportedOperation {
            operation: "error_transaction_shell_missing_parent_token",
        },
    )?;
    state::set_node_status(instance, return_node_index, NodeRuntimeStatus::Cancelled);
    cancel_transaction_boundary_siblings(
        process,
        instance,
        return_node_index,
        &matching_boundaries,
    )?;

    let mut selected_routes = Vec::with_capacity(matching_boundaries.len());
    for boundary_index in &matching_boundaries {
        state::set_node_status(instance, *boundary_index, NodeRuntimeStatus::Completed);
        let edge_index = state::resolve_single_outgoing_edge(
            process,
            *boundary_index,
            "error_transaction_shell_routing",
        )?;
        let next_node_index = process.edges[edge_index as usize].to;
        state::set_node_status(instance, next_node_index, NodeRuntimeStatus::Queued);
        selected_routes.push((edge_index, next_node_index));
    }

    let Some((first_edge_index, first_next_node_index)) = selected_routes.first().copied() else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "error_transaction_shell_missing_boundary_route",
        });
    };
    state::set_active_node_index(
        instance,
        parent_token_index,
        first_edge_index,
        first_next_node_index,
    );
    for (edge_index, next_node_index) in selected_routes.into_iter().skip(1) {
        state::push_active_token(instance, edge_index, next_node_index);
    }
    state::record_transition(instance, now_ms, InstanceLifecycle::Running);
    Ok(())
}

fn error_reference_matches(
    thrown_reference_id: Option<&str>,
    boundary_reference_id: Option<&str>,
) -> bool {
    match boundary_reference_id {
        None => true,
        Some(boundary_reference_id) => thrown_reference_id == Some(boundary_reference_id),
    }
}

pub(super) fn cancel_transaction_boundary_siblings(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    owner_node_index: BpmnNodeIndex,
    selected_boundary_indices: &[BpmnNodeIndex],
) -> Result<()> {
    for boundary in process.boundary_events_for_attached_node(owner_node_index) {
        if selected_boundary_indices.contains(&boundary.index) {
            continue;
        }
        let _ = process.event_for_node(boundary.index).ok_or_else(|| {
            BpmnEngineError::MissingRequiredNodeElement {
                process_id: process.key.process_id.to_string(),
                node_id: boundary.bpmn_id.to_string(),
                element: "event_definition",
            }
        })?;
        state::set_node_status(instance, boundary.index, NodeRuntimeStatus::Cancelled);
    }
    Ok(())
}
