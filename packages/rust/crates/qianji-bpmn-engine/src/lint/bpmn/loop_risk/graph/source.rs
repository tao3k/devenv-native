use super::{BpmnNodeKind, BpmnProcessSpec, HashSet, is_host_task, strongly_connected_components};

pub(in crate::lint::bpmn::loop_risk) fn source_component_entry_candidate(
    process: &BpmnProcessSpec,
    current_component_set: &HashSet<usize>,
    gateway_index: usize,
) -> Option<String> {
    let mut candidates = strongly_connected_components(process)
        .into_iter()
        .filter(|component| {
            !component
                .iter()
                .any(|index| current_component_set.contains(index))
        })
        .filter(|component| {
            !component
                .iter()
                .any(|index| process.nodes[*index].kind == BpmnNodeKind::StartEvent)
        })
        .filter(|component| component_has_no_external_incoming(process, component))
        .filter_map(|component| source_component_entry(process, &component, gateway_index))
        .collect::<Vec<_>>();
    candidates.sort_by_key(|candidate| {
        (
            candidate.index <= gateway_index,
            !candidate.is_host_task,
            !candidate.is_end_event,
            candidate.index,
        )
    });
    candidates
        .first()
        .map(|candidate| candidate.node_id.clone())
}

pub(in crate::lint::bpmn::loop_risk) struct SourceComponentEntry {
    node_id: String,
    index: usize,
    is_host_task: bool,
    is_end_event: bool,
}

pub(in crate::lint::bpmn::loop_risk) fn component_has_no_external_incoming(
    process: &BpmnProcessSpec,
    component: &[usize],
) -> bool {
    let component_set = component.iter().copied().collect::<HashSet<_>>();
    !process.edges.iter().any(|edge| {
        let Ok(source) = usize::try_from(edge.from) else {
            return false;
        };
        let Ok(target) = usize::try_from(edge.to) else {
            return false;
        };
        component_set.contains(&target) && !component_set.contains(&source)
    })
}

pub(in crate::lint::bpmn::loop_risk) fn source_component_entry(
    process: &BpmnProcessSpec,
    component: &[usize],
    gateway_index: usize,
) -> Option<SourceComponentEntry> {
    component
        .iter()
        .map(|index| {
            let node = &process.nodes[*index];
            SourceComponentEntry {
                node_id: (node.bpmn_id.to_string()),
                index: *index,
                is_host_task: is_host_task(&node.kind),
                is_end_event: node.kind == BpmnNodeKind::EndEvent,
            }
        })
        .min_by_key(|candidate| {
            (
                candidate.index <= gateway_index,
                !candidate.is_host_task,
                !candidate.is_end_event,
                candidate.index,
            )
        })
}

pub(in crate::lint::bpmn::loop_risk) fn incoming_edge_counts(
    process: &BpmnProcessSpec,
) -> Vec<usize> {
    let mut counts = vec![0; process.nodes.len()];
    for edge in &process.edges {
        if let Ok(index) = usize::try_from(edge.to)
            && let Some(count) = counts.get_mut(index)
        {
            *count += 1;
        }
    }
    counts
}
