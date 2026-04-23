use super::boundary::cancel_attached_boundary_siblings;
use crate::runtime::lifecycle::scope::{
    BpmnAdvanceOutcome, BpmnEngineError, BpmnEventKind, BpmnInstanceState, BpmnNodeIndex,
    BpmnPackage, BpmnProcessSpec, BpmnSubProcessKind, InstanceLifecycle, NodeRuntimeStatus, Result,
    pop_call_activity_frame, resolve_process_for_instance, restore_call_activity_frame,
};
use crate::runtime::lifecycle::state;

pub(crate) fn fail_root_process(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    thrown_reference_id: Option<&str>,
    now_ms: u64,
) -> Result<BpmnAdvanceOutcome> {
    let failure_message =
        root_error_failure_message(process, current_node_index, thrown_reference_id)?;
    let mut cancelled_node_indices = instance
        .active_tokens
        .iter()
        .map(|token| token.node_index)
        .filter(|node_index| *node_index != current_node_index)
        .collect::<Vec<_>>();
    cancelled_node_indices.extend(
        instance
            .waits
            .iter()
            .flat_map(|wait| [Some(wait.node_index), wait.blocking_node_index])
            .flatten()
            .filter(|node_index| *node_index != current_node_index),
    );
    cancelled_node_indices.extend(
        instance
            .pending_host_work
            .iter()
            .map(|pending| pending.node_index)
            .filter(|node_index| *node_index != current_node_index),
    );
    cancelled_node_indices.sort_unstable();
    cancelled_node_indices.dedup();
    for node_index in cancelled_node_indices {
        state::set_node_status(instance, node_index, NodeRuntimeStatus::Cancelled);
    }

    state::set_node_status(instance, current_node_index, NodeRuntimeStatus::Failed);
    let _ = state::remove_active_token(instance, current_token_index);
    instance.active_tokens.clear();
    instance.joins.clear();
    instance.standard_loops.clear();
    instance.sequential_multi_instances.clear();
    instance.parallel_multi_instances.clear();
    instance.waits.clear();
    instance.event_competition = None;
    instance.detached_transaction_compensation = None;
    instance.pending_host_work.clear();
    instance.suspend_reason = None;
    state::record_transition(instance, now_ms, InstanceLifecycle::Failed);

    Ok(BpmnAdvanceOutcome::Failed(failure_message))
}

pub(crate) fn error_subprocess_shell(
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
        operation: "error_subprocess_shell_missing_parent_frame",
    })?;
    let return_node_index = restore_call_activity_frame(instance, frame);

    let process = resolve_process_for_instance(package, instance)?;
    let subprocess_node = process.nodes.get(return_node_index as usize).ok_or(
        BpmnEngineError::UnsupportedOperation {
            operation: "error_subprocess_shell_missing_parent_node",
        },
    )?;
    if !matches!(
        subprocess_node.subprocess_kind,
        Some(
            BpmnSubProcessKind::CallActivity
                | BpmnSubProcessKind::Embedded
                | BpmnSubProcessKind::Transaction
        )
    ) {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "error_subprocess_shell_parent_not_supported_shell",
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
            operation: "error_subprocess_shell_missing_boundary",
        });
    }

    let parent_token_index = state::token_index_for_node(instance, return_node_index).ok_or(
        BpmnEngineError::UnsupportedOperation {
            operation: "error_subprocess_shell_missing_parent_token",
        },
    )?;
    state::set_node_status(instance, return_node_index, NodeRuntimeStatus::Cancelled);
    cancel_attached_boundary_siblings(process, instance, return_node_index, &matching_boundaries)?;
    state::clear_boundary_wait_for_node(instance, return_node_index);
    if instance.waits.is_empty() {
        instance.suspend_reason = None;
    }

    let mut selected_routes = Vec::with_capacity(matching_boundaries.len());
    for boundary_index in &matching_boundaries {
        state::set_node_status(instance, *boundary_index, NodeRuntimeStatus::Completed);
        let edge_index = state::resolve_single_outgoing_edge(
            process,
            *boundary_index,
            "error_subprocess_shell_routing",
        )?;
        let next_node_index = process.edges[edge_index as usize].to;
        state::set_node_status(instance, next_node_index, NodeRuntimeStatus::Queued);
        selected_routes.push((edge_index, next_node_index));
    }

    let Some((first_edge_index, first_next_node_index)) = selected_routes.first().copied() else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "error_subprocess_shell_missing_boundary_route",
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

fn root_error_failure_message(
    process: &BpmnProcessSpec,
    current_node_index: BpmnNodeIndex,
    thrown_reference_id: Option<&str>,
) -> Result<String> {
    let node = process.nodes.get(current_node_index as usize).ok_or(
        BpmnEngineError::UnsupportedOperation {
            operation: "root_error_end_missing_node",
        },
    )?;
    Ok(match thrown_reference_id {
        Some(thrown_reference_id) => format!(
            "process '{}' terminated with BPMN error end '{}' (errorRef='{}')",
            process.key.process_id, node.bpmn_id, thrown_reference_id
        ),
        None => format!(
            "process '{}' terminated with BPMN error end '{}'",
            process.key.process_id, node.bpmn_id
        ),
    })
}
