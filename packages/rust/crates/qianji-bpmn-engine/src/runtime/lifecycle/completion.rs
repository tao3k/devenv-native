use super::scope::{
    BpmnEngineError, BpmnInstanceState, BpmnNodeIndex, BpmnProcessSpec, BpmnRepeatSpec,
    InstanceLifecycle, MultiInstanceCompletionCounts, NodeRuntimeStatus, Result,
    clear_parallel_multi_instance_state, clear_sequential_multi_instance_state,
    clear_standard_loop_state, complete_parallel_multi_instance_iteration,
    ensure_sequential_multi_instance_state, increment_sequential_multi_instance_iterations,
    increment_standard_loop_iterations, parallel_multi_instance_state,
    sequential_multi_instance_state,
};
use super::{repeat, state, transaction};

pub(super) fn complete_node_and_route(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    node_index: BpmnNodeIndex,
    now_ms: u64,
    operation: &'static str,
) -> Result<()> {
    state::set_node_status(instance, node_index, NodeRuntimeStatus::Completed);
    let edge_index = state::resolve_single_outgoing_edge(process, node_index, operation)?;
    let next_node_index = process.edges[edge_index as usize].to;
    state::set_active_node_index(instance, current_token_index, edge_index, next_node_index);
    state::set_node_status(instance, next_node_index, NodeRuntimeStatus::Queued);
    state::record_transition(instance, now_ms, InstanceLifecycle::Running);
    Ok(())
}

pub(super) fn complete_local_task_execution(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    node_index: BpmnNodeIndex,
    output_data: &serde_json::Value,
    now_ms: u64,
) -> Result<()> {
    let mut routing_token_index = current_token_index;
    let current_token_id = instance
        .active_tokens
        .get(current_token_index)
        .map(|token| token.token_id)
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "complete_local_task_execution_missing_active_token",
        })?;
    let excluded_output_keys = repeat::capture_multi_instance_iteration_output(
        instance,
        node_index,
        current_token_id,
        output_data,
    )?;
    repeat::merge_output_data_excluding(
        &mut instance.variables,
        output_data,
        &excluded_output_keys,
    );

    if complete_standard_loop_execution(process, instance, node_index, now_ms)? {
        return Ok(());
    }

    if complete_sequential_multi_instance_execution(process, instance, node_index, now_ms)? {
        return Ok(());
    }

    if let Some(next_routing_token_index) = complete_parallel_multi_instance_execution(
        process,
        instance,
        current_token_index,
        current_token_id,
        node_index,
        now_ms,
    )? {
        routing_token_index = next_routing_token_index;
    } else if matches!(
        process.nodes[node_index as usize].repeat.as_ref(),
        Some(BpmnRepeatSpec::ParallelMultiInstance(_))
    ) {
        return Ok(());
    }

    transaction::record_completed_compensable_activity(process, instance, node_index);
    complete_node_and_route(
        process,
        instance,
        routing_token_index,
        node_index,
        now_ms,
        "complete_local_task_execution_routing",
    )
}

fn complete_standard_loop_execution(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<bool> {
    let Some(BpmnRepeatSpec::StandardLoop(loop_spec)) =
        process.nodes[node_index as usize].repeat.as_ref()
    else {
        return Ok(false);
    };
    let completed_iterations = increment_standard_loop_iterations(instance, node_index);
    if repeat::standard_loop_should_continue(
        process,
        node_index,
        loop_spec,
        completed_iterations,
        &instance.variables,
    )? {
        state::set_node_status(instance, node_index, NodeRuntimeStatus::Queued);
        state::record_transition(instance, now_ms, InstanceLifecycle::Running);
        return Ok(true);
    }
    clear_standard_loop_state(instance, node_index);
    Ok(false)
}

fn complete_sequential_multi_instance_execution(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<bool> {
    let Some(BpmnRepeatSpec::SequentialMultiInstance(loop_spec)) =
        process.nodes[node_index as usize].repeat.as_ref()
    else {
        return Ok(false);
    };
    let total_iterations =
        resolve_sequential_multi_instance_total_iterations(instance, node_index, loop_spec)?;
    ensure_sequential_multi_instance_state(instance, node_index, total_iterations, None);
    let Some((completed_iterations, total_iterations)) =
        increment_sequential_multi_instance_iterations(instance, node_index)
    else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "complete_local_task_execution_missing_multi_instance_state",
        });
    };
    let completion_reached = repeat::multi_instance_completion_condition_reached(
        process,
        node_index,
        loop_spec.completion_condition.as_deref(),
        &instance.variables,
        MultiInstanceCompletionCounts {
            total: total_iterations,
            completed: completed_iterations,
            active: total_iterations.saturating_sub(completed_iterations),
        },
    )?;
    if !completion_reached && completed_iterations < total_iterations {
        state::set_node_status(instance, node_index, NodeRuntimeStatus::Queued);
        state::record_transition(instance, now_ms, InstanceLifecycle::Running);
        return Ok(true);
    }
    finalize_sequential_multi_instance_output(instance, node_index)?;
    clear_sequential_multi_instance_state(instance, node_index);
    Ok(false)
}

fn resolve_sequential_multi_instance_total_iterations(
    instance: &BpmnInstanceState,
    node_index: BpmnNodeIndex,
    loop_spec: &crate::ir_repeat_api::BpmnSequentialMultiInstanceSpec,
) -> Result<u32> {
    loop_spec
        .loop_cardinality
        .or_else(|| {
            sequential_multi_instance_state(instance, node_index)
                .map(|state| state.total_iterations)
        })
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "complete_local_task_execution_missing_sequential_multi_instance_plan",
        })
}

fn finalize_sequential_multi_instance_output(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
) -> Result<()> {
    if let Some(data_binding) = sequential_multi_instance_state(instance, node_index)
        .and_then(|state| state.data_binding.clone())
    {
        repeat::finalize_multi_instance_output_collection(&mut instance.variables, &data_binding)?;
    }
    Ok(())
}

fn complete_parallel_multi_instance_execution(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_token_id: u64,
    node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<Option<usize>> {
    let Some(BpmnRepeatSpec::ParallelMultiInstance(loop_spec)) =
        process.nodes[node_index as usize].repeat.as_ref()
    else {
        return Ok(Some(current_token_index));
    };
    let Some((completed_iterations, total_iterations)) =
        complete_parallel_multi_instance_iteration(instance, node_index, current_token_id)
    else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "complete_local_task_execution_missing_parallel_multi_instance_state",
        });
    };
    let completion_reached = repeat::multi_instance_completion_condition_reached(
        process,
        node_index,
        loop_spec.completion_condition.as_deref(),
        &instance.variables,
        MultiInstanceCompletionCounts {
            total: total_iterations,
            completed: completed_iterations,
            active: total_iterations.saturating_sub(completed_iterations),
        },
    )?;
    if completion_reached {
        repeat::cancel_parallel_multi_instance_siblings(instance, node_index, current_token_id);
        let routing_token_index = state::token_index_for_id(instance, current_token_id).ok_or(
            BpmnEngineError::UnsupportedOperation {
                operation: "complete_local_task_execution_missing_parallel_multi_instance_survivor",
            },
        )?;
        finalize_parallel_multi_instance_output(instance, node_index)?;
        clear_parallel_multi_instance_state(instance, node_index);
        return Ok(Some(routing_token_index));
    }
    if completed_iterations < total_iterations {
        let _ = state::remove_active_token(instance, current_token_index);
        state::set_node_status(instance, node_index, NodeRuntimeStatus::Executing);
        state::record_transition(instance, now_ms, InstanceLifecycle::Running);
        return Ok(None);
    }
    finalize_parallel_multi_instance_output(instance, node_index)?;
    clear_parallel_multi_instance_state(instance, node_index);
    Ok(Some(current_token_index))
}

fn finalize_parallel_multi_instance_output(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
) -> Result<()> {
    if let Some(data_binding) = parallel_multi_instance_state(instance, node_index)
        .and_then(|state| state.data_binding.clone())
    {
        repeat::finalize_multi_instance_output_collection(&mut instance.variables, &data_binding)?;
    }
    Ok(())
}
