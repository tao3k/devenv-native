use crate::runtime::lifecycle::{
    scope::{
        BpmnEngineError, BpmnInstanceState, BpmnNodeIndex, BpmnNodeKind, BpmnProcessSpec,
        BpmnStandardLoopSpec, MultiInstanceCompletionConditionError, MultiInstanceCompletionCounts,
        PendingHostWorkKind, Result, evaluate_multi_instance_completion_condition,
    },
    state,
};

pub(crate) fn merge_output_data(
    variables: &mut serde_json::Value,
    output_data: &serde_json::Value,
) {
    merge_output_data_excluding(variables, output_data, &[]);
}

pub(crate) fn merge_output_data_excluding(
    variables: &mut serde_json::Value,
    output_data: &serde_json::Value,
    excluded_keys: &[String],
) {
    if let Some(obj) = output_data.as_object() {
        for (key, value) in obj {
            if excluded_keys.iter().any(|excluded| excluded == key) {
                continue;
            }
            variables[key] = value.clone();
        }
    }
}

pub(crate) fn node_matches_pending_kind(
    node_kind: &BpmnNodeKind,
    pending_kind: &PendingHostWorkKind,
) -> bool {
    matches!(
        (node_kind, pending_kind),
        (BpmnNodeKind::SendTask, PendingHostWorkKind::Send)
            | (BpmnNodeKind::ServiceTask, PendingHostWorkKind::Service)
            | (BpmnNodeKind::UserTask, PendingHostWorkKind::User)
            | (BpmnNodeKind::ManualTask, PendingHostWorkKind::Manual)
            | (
                BpmnNodeKind::BusinessRuleTask,
                PendingHostWorkKind::BusinessRule
            )
    )
}

pub(crate) fn pending_host_kind_name(kind: &PendingHostWorkKind) -> &'static str {
    match kind {
        PendingHostWorkKind::Send => "send",
        PendingHostWorkKind::Service => "service",
        PendingHostWorkKind::User => "user",
        PendingHostWorkKind::Manual => "manual",
        PendingHostWorkKind::BusinessRule => "business_rule",
    }
}

pub(crate) fn standard_loop_should_continue(
    process: &BpmnProcessSpec,
    node_index: BpmnNodeIndex,
    loop_spec: &BpmnStandardLoopSpec,
    completed_iterations: u32,
    variables: &serde_json::Value,
) -> Result<bool> {
    if let Some(loop_maximum) = loop_spec.loop_maximum
        && completed_iterations >= loop_maximum
    {
        return Ok(false);
    }

    let Some(loop_condition) = loop_spec.loop_condition.as_deref() else {
        return Ok(true);
    };
    evaluate_standard_loop_condition(process, node_index, loop_condition, variables)
}

fn evaluate_standard_loop_condition(
    process: &BpmnProcessSpec,
    node_index: BpmnNodeIndex,
    loop_condition: &str,
    variables: &serde_json::Value,
) -> Result<bool> {
    let trimmed = loop_condition.trim();
    let (negated, path) = match trimmed.strip_prefix("not ") {
        Some(path) => (true, path.trim()),
        None => (false, trimmed),
    };
    let value = resolve_boolean_variable_path(variables, path).ok_or_else(|| {
        BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: process.key.process_id.to_string(),
            node_id: process.nodes[node_index as usize].bpmn_id.to_string(),
            detail: "loop_condition_variable_unresolved",
        }
    })?;
    Ok(if negated { !value } else { value })
}

pub(crate) fn multi_instance_completion_condition_reached(
    process: &BpmnProcessSpec,
    node_index: BpmnNodeIndex,
    completion_condition: Option<&str>,
    variables: &serde_json::Value,
    counts: MultiInstanceCompletionCounts,
) -> Result<bool> {
    let Some(completion_condition) = completion_condition else {
        return Ok(false);
    };
    evaluate_multi_instance_completion_condition(completion_condition, variables, counts).map_err(
        |error| match error {
            MultiInstanceCompletionConditionError::UnresolvedVariablePath(_) => {
                BpmnEngineError::UnsupportedLoopConfiguration {
                    process_id: process.key.process_id.to_string(),
                    node_id: process.nodes[node_index as usize].bpmn_id.to_string(),
                    detail: "multi_instance_completion_condition_variable_unresolved",
                }
            }
            MultiInstanceCompletionConditionError::UnsupportedExpression => {
                BpmnEngineError::UnsupportedLoopConfiguration {
                    process_id: process.key.process_id.to_string(),
                    node_id: process.nodes[node_index as usize].bpmn_id.to_string(),
                    detail: "unsupported_multi_instance_completion_condition_expression",
                }
            }
        },
    )
}

pub(crate) fn cancel_parallel_multi_instance_siblings(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
    surviving_token_id: u64,
) {
    instance.pending_host_work.retain(|pending| {
        pending.token_id == surviving_token_id || pending.node_index != node_index
    });
    instance
        .active_tokens
        .retain(|token| token.token_id == surviving_token_id || token.node_index != node_index);
    if !state::has_pending_host_work_for_node(instance, node_index) {
        state::clear_boundary_wait_for_node(instance, node_index);
    }
}

fn resolve_boolean_variable_path(variables: &serde_json::Value, path: &str) -> Option<bool> {
    let mut current = variables;
    for segment in path.split('.') {
        current = current.get(segment)?;
    }
    current.as_bool()
}
