use super::scope::{
    BpmnEngineError, BpmnInstanceState, BpmnNodeIndex, BpmnProcessSpec, BpmnRepeatSpec,
    InstanceLifecycle, NodeRuntimeStatus, Result, clear_parallel_multi_instance_state,
    clear_sequential_multi_instance_state, clear_standard_loop_state,
    ensure_sequential_multi_instance_state, ensure_standard_loop_state,
    has_parallel_multi_instance_state, register_parallel_multi_instance_iteration,
    sequential_multi_instance_state, standard_loop_completed_iterations,
};
use super::{completion, repeat, state};

pub(super) fn prepare_standard_loop_iteration(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<bool> {
    let Some(BpmnRepeatSpec::StandardLoop(loop_spec)) =
        process.nodes[node_index as usize].repeat.as_ref()
    else {
        return Ok(false);
    };

    let completed_iterations = standard_loop_completed_iterations(instance, node_index);
    if loop_spec.test_before
        && !repeat::standard_loop_should_continue(
            process,
            node_index,
            loop_spec,
            completed_iterations,
            &instance.variables,
        )?
    {
        clear_standard_loop_state(instance, node_index);
        completion::complete_node_and_route(
            process,
            instance,
            current_token_index,
            node_index,
            now_ms,
            "advance_instance_standard_loop_routing",
        )?;
        return Ok(true);
    }

    ensure_standard_loop_state(instance, node_index);
    Ok(false)
}

pub(super) fn prepare_sequential_multi_instance_iteration(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<bool> {
    let Some(BpmnRepeatSpec::SequentialMultiInstance(loop_spec)) =
        process.nodes[node_index as usize].repeat.as_ref()
    else {
        return Ok(false);
    };

    if sequential_multi_instance_state(instance, node_index).is_some() {
        return Ok(false);
    }

    let (total_iterations, data_binding) = repeat::resolve_multi_instance_iteration_plan(
        process,
        node_index,
        loop_spec.loop_cardinality,
        loop_spec.data_binding.as_ref(),
        &instance.variables,
    )?;

    if total_iterations == 0 {
        if let Some(data_binding) = data_binding.as_ref() {
            repeat::finalize_multi_instance_output_collection(
                &mut instance.variables,
                data_binding,
            )?;
        }
        clear_sequential_multi_instance_state(instance, node_index);
        completion::complete_node_and_route(
            process,
            instance,
            current_token_index,
            node_index,
            now_ms,
            "advance_instance_sequential_multi_instance_routing",
        )?;
        return Ok(true);
    }

    ensure_sequential_multi_instance_state(
        instance,
        node_index,
        total_iterations,
        data_binding.as_ref(),
    );
    Ok(false)
}

pub(super) fn prepare_parallel_multi_instance_iteration(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    node_index: BpmnNodeIndex,
    repeat: Option<&BpmnRepeatSpec>,
    now_ms: u64,
) -> Result<bool> {
    let Some(BpmnRepeatSpec::ParallelMultiInstance(loop_spec)) = repeat else {
        return Ok(false);
    };

    if has_parallel_multi_instance_state(instance, node_index) {
        return Ok(false);
    }

    let (total_iterations, data_binding) = repeat::resolve_multi_instance_iteration_plan(
        process,
        node_index,
        loop_spec.loop_cardinality,
        loop_spec.data_binding.as_ref(),
        &instance.variables,
    )?;

    if total_iterations == 0 {
        if let Some(data_binding) = data_binding.as_ref() {
            repeat::finalize_multi_instance_output_collection(
                &mut instance.variables,
                data_binding,
            )?;
        }
        clear_parallel_multi_instance_state(instance, node_index);
        completion::complete_node_and_route(
            process,
            instance,
            current_token_index,
            node_index,
            now_ms,
            "advance_instance_parallel_multi_instance_routing",
        )?;
        return Ok(true);
    }

    let (current_token_id, incoming_edge_index) = instance
        .active_tokens
        .get(current_token_index)
        .map(|token| (token.token_id, token.incoming_edge_index))
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "prepare_parallel_multi_instance_missing_active_token",
        })?;
    register_parallel_multi_instance_iteration(
        instance,
        node_index,
        total_iterations,
        data_binding.as_ref(),
        current_token_id,
        0,
    );
    if total_iterations == 1 {
        return Ok(false);
    }

    for iteration_index in 1..total_iterations {
        let token_id =
            state::push_active_token_with_arrival(instance, incoming_edge_index, node_index);
        register_parallel_multi_instance_iteration(
            instance,
            node_index,
            total_iterations,
            data_binding.as_ref(),
            token_id,
            iteration_index,
        );
    }
    state::set_node_status(instance, node_index, NodeRuntimeStatus::Queued);
    state::record_transition(instance, now_ms, InstanceLifecycle::Running);
    Ok(true)
}
