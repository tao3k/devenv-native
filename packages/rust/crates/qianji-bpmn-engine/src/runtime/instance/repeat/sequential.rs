use super::state::{MultiInstanceDataRuntimeState, SequentialMultiInstanceState};
use crate::ir_index_api::BpmnNodeIndex;
use crate::runtime::instance::shell::BpmnInstanceState;

pub(crate) fn ensure_sequential_multi_instance_state(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
    total_iterations: u32,
    data_binding: Option<&MultiInstanceDataRuntimeState>,
) {
    if let Some(state) = instance
        .sequential_multi_instances
        .iter()
        .find(|state| state.node_index == node_index)
    {
        let _ = state;
        return;
    }
    instance
        .sequential_multi_instances
        .push(SequentialMultiInstanceState {
            node_index,
            total_iterations,
            completed_iterations: 0,
            data_binding: data_binding.cloned(),
        });
}

pub(crate) fn increment_sequential_multi_instance_iterations(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
) -> Option<(u32, u32)> {
    let state = instance
        .sequential_multi_instances
        .iter_mut()
        .find(|state| state.node_index == node_index)?;
    state.completed_iterations += 1;
    Some((state.completed_iterations, state.total_iterations))
}

pub(crate) fn clear_sequential_multi_instance_state(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
) {
    instance
        .sequential_multi_instances
        .retain(|state| state.node_index != node_index);
}

pub(crate) fn sequential_multi_instance_state(
    instance: &BpmnInstanceState,
    node_index: BpmnNodeIndex,
) -> Option<&SequentialMultiInstanceState> {
    instance
        .sequential_multi_instances
        .iter()
        .find(|state| state.node_index == node_index)
}

pub(crate) fn sequential_multi_instance_state_mut(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
) -> Option<&mut SequentialMultiInstanceState> {
    instance
        .sequential_multi_instances
        .iter_mut()
        .find(|state| state.node_index == node_index)
}
