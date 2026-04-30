use super::{
    BTreeSet, BpmnGatewayKind, BpmnNodeKind, BpmnProcessSpec, GatewayConditionSummary,
    ProcessMetadata, outgoing_edge_indices, parse_gateway_condition_summary,
};

pub(super) fn route_variables(process: &BpmnProcessSpec, component: &[usize]) -> BTreeSet<String> {
    component
        .iter()
        .filter(|node_index| is_gateway(process.nodes[**node_index].gateway_kind.as_ref()))
        .flat_map(|node_index| {
            let Some(edge_indices) = outgoing_edge_indices(process, *node_index) else {
                return Vec::new();
            };
            edge_indices
                .iter()
                .filter_map(|edge_index| {
                    process.edges[*edge_index as usize]
                        .condition_expression
                        .as_deref()
                        .and_then(gateway_condition_variable_path)
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

pub(super) fn gateway_condition_variable_path(condition: &str) -> Option<String> {
    match parse_gateway_condition_summary(condition)? {
        GatewayConditionSummary::BooleanPath { path, .. } => Some(path),
        GatewayConditionSummary::NumericComparison { lhs, .. } => Some(lhs),
    }
}

pub(super) fn task_node_ids(process: &BpmnProcessSpec, component: &[usize]) -> Vec<String> {
    component
        .iter()
        .filter(|node_index| is_host_task(&process.nodes[**node_index].kind))
        .map(|node_index| process.nodes[*node_index].bpmn_id.to_string())
        .collect()
}

pub(super) fn gateway_node_ids(process: &BpmnProcessSpec, component: &[usize]) -> Vec<String> {
    component
        .iter()
        .filter(|node_index| is_gateway(process.nodes[**node_index].gateway_kind.as_ref()))
        .map(|node_index| process.nodes[*node_index].bpmn_id.to_string())
        .collect()
}

pub(super) fn updated_variables(
    metadata: &ProcessMetadata,
    task_node_ids: &[String],
) -> BTreeSet<String> {
    task_node_ids
        .iter()
        .flat_map(|node_id| metadata.task_outputs.get(node_id).into_iter().flatten())
        .cloned()
        .collect()
}

pub(super) fn user_task_outputs(
    process: &BpmnProcessSpec,
    metadata: &ProcessMetadata,
    component: &[usize],
) -> BTreeSet<String> {
    component
        .iter()
        .filter(|node_index| process.nodes[**node_index].kind == BpmnNodeKind::UserTask)
        .flat_map(|node_index| {
            let node_id = process.nodes[*node_index].bpmn_id.as_ref();
            metadata.task_outputs.get(node_id).into_iter().flatten()
        })
        .cloned()
        .collect()
}

pub(super) fn worker_task_inputs(
    process: &BpmnProcessSpec,
    metadata: &ProcessMetadata,
    component: &[usize],
) -> BTreeSet<String> {
    component
        .iter()
        .filter(|node_index| is_state_worker_task(&process.nodes[**node_index].kind))
        .flat_map(|node_index| {
            let node_id = process.nodes[*node_index].bpmn_id.as_ref();
            metadata.task_inputs.get(node_id).into_iter().flatten()
        })
        .cloned()
        .collect()
}

pub(super) fn worker_task_outputs(
    process: &BpmnProcessSpec,
    metadata: &ProcessMetadata,
    component: &[usize],
) -> BTreeSet<String> {
    component
        .iter()
        .filter(|node_index| is_state_worker_task(&process.nodes[**node_index].kind))
        .flat_map(|node_index| {
            let node_id = process.nodes[*node_index].bpmn_id.as_ref();
            metadata.task_outputs.get(node_id).into_iter().flatten()
        })
        .cloned()
        .collect()
}

pub(super) fn undeclared_variables<'a>(
    declared: &BTreeSet<String>,
    variables: impl Iterator<Item = &'a str>,
) -> BTreeSet<String> {
    variables
        .filter(|variable| !declares_variable(declared, variable))
        .map(ToString::to_string)
        .collect()
}

pub(super) fn declares_variable(declared: &BTreeSet<String>, variable_path: &str) -> bool {
    let root = variable_path.split('.').next().unwrap_or(variable_path);
    declared.contains(variable_path) || declared.contains(root)
}

pub(super) fn sorted_node_ids(process: &BpmnProcessSpec, component: &[usize]) -> Vec<String> {
    component
        .iter()
        .map(|node_index| process.nodes[*node_index].bpmn_id.to_string())
        .collect()
}

pub(super) fn sorted_set_values(values: &BTreeSet<String>) -> Vec<String> {
    values.iter().cloned().collect()
}

pub(super) fn is_host_task(kind: &BpmnNodeKind) -> bool {
    matches!(
        kind,
        BpmnNodeKind::ServiceTask
            | BpmnNodeKind::ScriptTask
            | BpmnNodeKind::UserTask
            | BpmnNodeKind::ManualTask
            | BpmnNodeKind::BusinessRuleTask
            | BpmnNodeKind::SendTask
            | BpmnNodeKind::ReceiveTask
    )
}

pub(super) fn is_state_worker_task(kind: &BpmnNodeKind) -> bool {
    matches!(
        kind,
        BpmnNodeKind::ServiceTask | BpmnNodeKind::ScriptTask | BpmnNodeKind::BusinessRuleTask
    )
}

pub(super) fn is_gateway(kind: Option<&BpmnGatewayKind>) -> bool {
    matches!(
        kind,
        Some(
            BpmnGatewayKind::Exclusive
                | BpmnGatewayKind::Inclusive
                | BpmnGatewayKind::Parallel
                | BpmnGatewayKind::EventBased
        )
    )
}

pub(super) fn is_prompt_output(output: &str) -> bool {
    let normalized = output.to_ascii_lowercase();
    [
        "question",
        "questions",
        "choice",
        "choices",
        "prompt",
        "clarif",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}
