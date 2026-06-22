use super::parallel::parallel_multi_instance_state;
use super::sequential::sequential_multi_instance_state;
use super::state::MultiInstanceDataRuntimeState;
use crate::error::{BpmnEngineError, Result};
use crate::ir_index_api::BpmnNodeIndex;
use crate::runtime::instance::shell::BpmnInstanceState;

pub(crate) fn sequential_multi_instance_iteration_variables(
    instance: &BpmnInstanceState,
    node_index: BpmnNodeIndex,
    variables: &serde_json::Value,
) -> Result<Option<(u32, u32, serde_json::Value)>> {
    let Some(state) = sequential_multi_instance_state(instance, node_index) else {
        return Ok(None);
    };
    let iteration_index = state.completed_iterations;
    let variables = materialize_multi_instance_iteration_variables(
        variables,
        state.data_binding.as_ref(),
        iteration_index,
    )?;
    Ok(Some((iteration_index, state.total_iterations, variables)))
}

pub(crate) fn parallel_multi_instance_iteration_variables(
    instance: &BpmnInstanceState,
    node_index: BpmnNodeIndex,
    token_id: u64,
    variables: &serde_json::Value,
) -> Result<Option<(u32, u32, serde_json::Value)>> {
    let Some(state) = parallel_multi_instance_state(instance, node_index) else {
        return Ok(None);
    };
    let Some(iteration) = state
        .active_iterations
        .iter()
        .find(|iteration| iteration.token_id == token_id)
    else {
        return Ok(None);
    };
    let variables = materialize_multi_instance_iteration_variables(
        variables,
        state.data_binding.as_ref(),
        iteration.iteration_index,
    )?;
    Ok(Some((
        iteration.iteration_index,
        state.total_iterations,
        variables,
    )))
}

fn materialize_multi_instance_iteration_variables(
    variables: &serde_json::Value,
    data_binding: Option<&MultiInstanceDataRuntimeState>,
    iteration_index: u32,
) -> Result<serde_json::Value> {
    let Some(data_binding) = data_binding else {
        return Ok(variables.clone());
    };
    let slot = data_binding.slots.get(iteration_index as usize).ok_or(
        BpmnEngineError::UnsupportedOperation {
            operation: "materialize_multi_instance_iteration_variables_missing_slot",
        },
    )?;
    let mut variables = variables.clone();
    let Some(object) = variables.as_object_mut() else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "materialize_multi_instance_iteration_variables_non_object",
        });
    };
    object.insert(data_binding.input_data_item.to_string(), slot.input.clone());
    Ok(variables)
}
