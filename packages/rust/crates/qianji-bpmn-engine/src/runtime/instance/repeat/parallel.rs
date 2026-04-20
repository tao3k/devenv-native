use super::state::{
    MultiInstanceDataRuntimeState, ParallelMultiInstanceIterationState, ParallelMultiInstanceState,
};
use crate::ir_index_api::BpmnNodeIndex;
use crate::runtime::instance::shell::BpmnInstanceState;

pub(crate) fn has_parallel_multi_instance_state(
    instance: &BpmnInstanceState,
    node_index: BpmnNodeIndex,
) -> bool {
    instance
        .parallel_multi_instances
        .iter()
        .any(|state| state.node_index == node_index)
}

pub(crate) fn register_parallel_multi_instance_iteration(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
    total_iterations: u32,
    data_binding: Option<&MultiInstanceDataRuntimeState>,
    token_id: u64,
    iteration_index: u32,
) {
    if let Some(state) = instance
        .parallel_multi_instances
        .iter_mut()
        .find(|state| state.node_index == node_index)
    {
        state.total_iterations = total_iterations;
        if state.data_binding.is_none() {
            state.data_binding = data_binding.cloned();
        }
        if let Some(iteration) = state
            .active_iterations
            .iter_mut()
            .find(|iteration| iteration.token_id == token_id)
        {
            iteration.iteration_index = iteration_index;
        } else {
            state
                .active_iterations
                .push(ParallelMultiInstanceIterationState {
                    token_id,
                    iteration_index,
                });
        }
        state
            .active_iterations
            .sort_by_key(|iteration| (iteration.iteration_index, iteration.token_id));
        return;
    }

    instance
        .parallel_multi_instances
        .push(ParallelMultiInstanceState {
            node_index,
            total_iterations,
            completed_iterations: 0,
            data_binding: data_binding.cloned(),
            active_iterations: vec![ParallelMultiInstanceIterationState {
                token_id,
                iteration_index,
            }],
        });
}

pub(crate) fn complete_parallel_multi_instance_iteration(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
    token_id: u64,
) -> Option<(u32, u32)> {
    let state = instance
        .parallel_multi_instances
        .iter_mut()
        .find(|state| state.node_index == node_index)?;
    state
        .active_iterations
        .retain(|iteration| iteration.token_id != token_id);
    state.completed_iterations += 1;
    Some((state.completed_iterations, state.total_iterations))
}

pub(crate) fn parallel_multi_instance_token_ids(
    instance: &BpmnInstanceState,
    node_index: BpmnNodeIndex,
) -> Vec<u64> {
    instance
        .parallel_multi_instances
        .iter()
        .find(|state| state.node_index == node_index)
        .map_or_else(Vec::new, |state| {
            state
                .active_iterations
                .iter()
                .map(|iteration| iteration.token_id)
                .collect()
        })
}

pub(crate) fn clear_parallel_multi_instance_state(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
) {
    instance
        .parallel_multi_instances
        .retain(|state| state.node_index != node_index);
}

pub(crate) fn parallel_multi_instance_state(
    instance: &BpmnInstanceState,
    node_index: BpmnNodeIndex,
) -> Option<&ParallelMultiInstanceState> {
    instance
        .parallel_multi_instances
        .iter()
        .find(|state| state.node_index == node_index)
}

pub(crate) fn parallel_multi_instance_state_mut(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
) -> Option<&mut ParallelMultiInstanceState> {
    instance
        .parallel_multi_instances
        .iter_mut()
        .find(|state| state.node_index == node_index)
}
