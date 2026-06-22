use super::state::StandardLoopState;
use crate::ir_index_api::BpmnNodeIndex;
use crate::runtime::instance::shell::BpmnInstanceState;

pub(crate) fn standard_loop_completed_iterations(
    instance: &BpmnInstanceState,
    node_index: BpmnNodeIndex,
) -> u32 {
    instance
        .standard_loops
        .iter()
        .find(|state| state.node_index == node_index)
        .map_or(0, |state| state.completed_iterations)
}

pub(crate) fn ensure_standard_loop_state(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
) {
    if instance
        .standard_loops
        .iter()
        .any(|state| state.node_index == node_index)
    {
        return;
    }
    instance.standard_loops.push(StandardLoopState {
        node_index,
        completed_iterations: 0,
    });
}

pub(crate) fn increment_standard_loop_iterations(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
) -> u32 {
    if let Some(state) = instance
        .standard_loops
        .iter_mut()
        .find(|state| state.node_index == node_index)
    {
        state.completed_iterations += 1;
        return state.completed_iterations;
    }

    instance.standard_loops.push(StandardLoopState {
        node_index,
        completed_iterations: 1,
    });
    1
}

pub(crate) fn clear_standard_loop_state(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
) {
    instance
        .standard_loops
        .retain(|state| state.node_index != node_index);
}
