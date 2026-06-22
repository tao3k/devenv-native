use crate::runtime::lifecycle::scope::{
    Arc, BpmnEngineError, BpmnInstanceState, BpmnMultiInstanceDataBindingSpec, BpmnNodeIndex,
    BpmnProcessSpec, Map, MultiInstanceCollectionKey, MultiInstanceCollectionKind,
    MultiInstanceCollectionSlot, MultiInstanceDataRuntimeState, MultiInstanceOutputCollectionState,
    Result, Value, parallel_multi_instance_iteration_variables, parallel_multi_instance_state_mut,
    sequential_multi_instance_iteration_variables, sequential_multi_instance_state_mut,
};

pub(crate) fn materialize_node_execution_variables(
    instance: &BpmnInstanceState,
    node_index: BpmnNodeIndex,
    token_id: u64,
) -> Result<Value> {
    if let Some((_, _, variables)) =
        sequential_multi_instance_iteration_variables(instance, node_index, &instance.variables)?
    {
        return Ok(variables);
    }
    if let Some((_, _, variables)) = parallel_multi_instance_iteration_variables(
        instance,
        node_index,
        token_id,
        &instance.variables,
    )? {
        return Ok(variables);
    }
    Ok(instance.variables.clone())
}

pub(crate) fn resolve_multi_instance_iteration_plan(
    process: &BpmnProcessSpec,
    node_index: BpmnNodeIndex,
    loop_cardinality: Option<u32>,
    data_binding: Option<&BpmnMultiInstanceDataBindingSpec>,
    variables: &Value,
) -> Result<(u32, Option<MultiInstanceDataRuntimeState>)> {
    if let Some(loop_cardinality) = loop_cardinality {
        return Ok((loop_cardinality, None));
    }

    let data_binding = data_binding.ok_or(BpmnEngineError::UnsupportedOperation {
        operation: "resolve_multi_instance_iteration_plan_missing_binding",
    })?;
    let source_collection =
        resolve_value_path(variables, data_binding.loop_data_input_ref.as_ref()).ok_or_else(
            || BpmnEngineError::UnsupportedLoopConfiguration {
                process_id: (process.key.process_id.to_string()).into(),
                node_id: (process.nodes[node_index as usize].bpmn_id.to_string()).into(),
                detail: "loop_data_input_collection_unresolved",
            },
        )?;
    let (collection_kind, slots) = match source_collection {
        Value::Array(items) => (
            MultiInstanceCollectionKind::Array,
            items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    Ok(MultiInstanceCollectionSlot {
                        key: MultiInstanceCollectionKey::Index(u32::try_from(index).map_err(
                            |_| BpmnEngineError::UnsupportedOperation {
                                operation: "resolve_multi_instance_iteration_plan_index_overflow",
                            },
                        )?),
                        input: item.clone(),
                    })
                })
                .collect::<Result<Vec<_>>>()?,
        ),
        Value::Object(entries) => (
            MultiInstanceCollectionKind::Object,
            entries
                .iter()
                .map(|(key, item)| MultiInstanceCollectionSlot {
                    key: MultiInstanceCollectionKey::Key(Arc::<str>::from(key.as_str())),
                    input: item.clone(),
                })
                .collect(),
        ),
        _ => {
            return Err(BpmnEngineError::UnsupportedLoopConfiguration {
                process_id: (process.key.process_id.to_string()).into(),
                node_id: (process.nodes[node_index as usize].bpmn_id.to_string()).into(),
                detail: "unsupported_multi_instance_data_input_collection",
            });
        }
    };
    let total_iterations =
        u32::try_from(slots.len()).map_err(|_| BpmnEngineError::UnsupportedOperation {
            operation: "resolve_multi_instance_iteration_plan_slot_count_overflow",
        })?;
    let output = match (
        data_binding.loop_data_output_ref.as_ref(),
        data_binding.output_data_item.as_ref(),
    ) {
        (Some(loop_data_output_ref), Some(output_data_item)) => {
            Some(MultiInstanceOutputCollectionState {
                loop_data_output_ref: loop_data_output_ref.clone(),
                output_data_item: output_data_item.clone(),
                values: vec![None; slots.len()],
            })
        }
        (None, None) => None,
        _ => {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "resolve_multi_instance_iteration_plan_incomplete_output_binding",
            });
        }
    };
    Ok((
        total_iterations,
        Some(MultiInstanceDataRuntimeState {
            collection_kind,
            input_data_item: data_binding.input_data_item.clone(),
            slots,
            output,
        }),
    ))
}

pub(crate) fn capture_multi_instance_iteration_output(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
    token_id: u64,
    output_data: &Value,
) -> Result<Vec<String>> {
    if let Some(state) = sequential_multi_instance_state_mut(instance, node_index) {
        let iteration_index = state.completed_iterations;
        return capture_multi_instance_output_for_state(
            state.data_binding.as_mut(),
            iteration_index,
            output_data,
        );
    }

    if let Some(state) = parallel_multi_instance_state_mut(instance, node_index) {
        let iteration_index = state
            .active_iterations
            .iter()
            .find(|iteration| iteration.token_id == token_id)
            .map(|iteration| iteration.iteration_index)
            .ok_or(BpmnEngineError::UnsupportedOperation {
                operation: "capture_multi_instance_iteration_output_missing_parallel_iteration",
            })?;
        return capture_multi_instance_output_for_state(
            state.data_binding.as_mut(),
            iteration_index,
            output_data,
        );
    }

    Ok(Vec::new())
}

fn capture_multi_instance_output_for_state(
    data_binding: Option<&mut MultiInstanceDataRuntimeState>,
    iteration_index: u32,
    output_data: &Value,
) -> Result<Vec<String>> {
    let Some(data_binding) = data_binding else {
        return Ok(Vec::new());
    };

    let mut excluded_keys = vec![data_binding.input_data_item.to_string()];
    let Some(output_state) = data_binding.output.as_mut() else {
        return Ok(excluded_keys);
    };
    excluded_keys.push(output_state.output_data_item.to_string());

    let output_object = output_data
        .as_object()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "capture_multi_instance_iteration_output_non_object",
        })?;
    let output_value = output_object
        .get(output_state.output_data_item.as_ref())
        .cloned()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "capture_multi_instance_iteration_output_missing_output_data_item",
        })?;
    let slot = output_state
        .values
        .get_mut(iteration_index as usize)
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "capture_multi_instance_iteration_output_missing_slot",
        })?;
    *slot = Some(output_value);
    Ok(excluded_keys)
}

pub(crate) fn finalize_multi_instance_output_collection(
    variables: &mut Value,
    data_binding: &MultiInstanceDataRuntimeState,
) -> Result<()> {
    let Some(output_state) = data_binding.output.as_ref() else {
        return Ok(());
    };
    let aggregated = build_multi_instance_output_collection(data_binding, output_state);
    assign_value_path(
        variables,
        output_state.loop_data_output_ref.as_ref(),
        aggregated,
    )
}

fn build_multi_instance_output_collection(
    data_binding: &MultiInstanceDataRuntimeState,
    output_state: &MultiInstanceOutputCollectionState,
) -> Value {
    match data_binding.collection_kind {
        MultiInstanceCollectionKind::Array => Value::Array(
            output_state
                .values
                .iter()
                .filter_map(Clone::clone)
                .collect::<Vec<_>>(),
        ),
        MultiInstanceCollectionKind::Object => {
            let mut object = Map::new();
            for (slot, value) in data_binding.slots.iter().zip(&output_state.values) {
                let Some(value) = value else {
                    continue;
                };
                if let MultiInstanceCollectionKey::Key(key) = &slot.key {
                    object.insert(key.to_string(), value.clone());
                }
            }
            Value::Object(object)
        }
    }
}

fn resolve_value_path<'a>(variables: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = variables;
    for segment in path.split('.').filter(|segment| !segment.is_empty()) {
        current = current.get(segment)?;
    }
    Some(current)
}

fn assign_value_path(variables: &mut Value, path: &str, value: Value) -> Result<()> {
    let Some(object) = variables.as_object_mut() else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "assign_value_path_non_object_root",
        });
    };
    let mut segments = path
        .split('.')
        .filter(|segment| !segment.is_empty())
        .peekable();
    let Some(last_segment) = segments.next_back() else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "assign_value_path_empty_path",
        });
    };

    let mut current = object;
    for segment in segments {
        let entry = current
            .entry(segment.to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        let Some(next) = entry.as_object_mut() else {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "assign_value_path_conflicting_non_object_segment",
            });
        };
        current = next;
    }
    current.insert(last_segment.to_string(), value);
    Ok(())
}
