use super::{
    BpmnNodeKind, BpmnProcessSpec, DefaultReentryFlow, HashMap, HashSet, ProcessMetadata,
    incoming_edge_counts, is_host_task, source_component_entry_candidate,
};

pub(in crate::lint::bpmn::loop_risk) fn default_reentry_flows(
    process: &BpmnProcessSpec,
    metadata: &ProcessMetadata,
    component_set: &HashSet<usize>,
    gateway_ids: &[String],
) -> Vec<DefaultReentryFlow> {
    let node_indices = node_id_to_index(process);
    gateway_ids
        .iter()
        .filter_map(|gateway_id| {
            let flow_id = metadata.gateway_default_flows.get(gateway_id)?;
            let flow = metadata.sequence_flows.get(flow_id)?;
            let target_index = node_indices.get(flow.target_ref.as_str())?;
            if !component_set.contains(target_index) {
                return None;
            }
            Some(DefaultReentryFlow {
                gateway_node: gateway_id.clone(),
                sequence_flow: flow_id.clone(),
                target_node: flow.target_ref.clone(),
                suggested_exit_target: suggested_default_exit_target(
                    process,
                    component_set,
                    gateway_id,
                ),
            })
        })
        .collect()
}

pub(in crate::lint::bpmn::loop_risk) fn node_id_to_index(
    process: &BpmnProcessSpec,
) -> HashMap<&str, usize> {
    process
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.bpmn_id.as_ref(), index))
        .collect()
}

pub(in crate::lint::bpmn::loop_risk) fn suggested_default_exit_target(
    process: &BpmnProcessSpec,
    component_set: &HashSet<usize>,
    gateway_id: &str,
) -> Option<String> {
    let gateway_index = process
        .nodes
        .iter()
        .position(|node| node.bpmn_id.as_ref() == gateway_id)
        .unwrap_or_default();

    if let Some(target) = source_component_entry_candidate(process, component_set, gateway_index) {
        return Some(target);
    }

    let incoming_counts = incoming_edge_counts(process);
    let mut candidates = process
        .nodes
        .iter()
        .enumerate()
        .filter(|(index, node)| {
            !component_set.contains(index)
                && node.kind != BpmnNodeKind::StartEvent
                && incoming_counts.get(*index).copied().unwrap_or_default() == 0
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|(index, node)| {
        (
            *index <= gateway_index,
            !is_host_task(&node.kind),
            !matches!(node.kind, BpmnNodeKind::EndEvent),
            *index,
        )
    });
    candidates.first().map(|(_, node)| node.bpmn_id.to_string())
}
