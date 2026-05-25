use super::{BpmnProcessSpec, HashSet, outgoing_edge_indices};

pub(in crate::lint::bpmn::loop_risk) fn is_cyclic_component(
    process: &BpmnProcessSpec,
    component: &[usize],
) -> bool {
    if component.len() > 1 {
        return true;
    }
    let Some(node_index) = component.first().copied() else {
        return false;
    };
    outgoing_edge_indices(process, node_index).is_some_and(|edge_indices| {
        edge_indices
            .iter()
            .any(|edge_index| process.edges[*edge_index as usize].to as usize == node_index)
    })
}

pub(in crate::lint::bpmn::loop_risk) fn component_has_exit_path(
    process: &BpmnProcessSpec,
    component_set: &HashSet<usize>,
) -> bool {
    component_set.iter().any(|node_index| {
        outgoing_edge_indices(process, *node_index).is_some_and(|edge_indices| {
            edge_indices.iter().any(|edge_index| {
                let target_index = process.edges[*edge_index as usize].to as usize;
                !component_set.contains(&target_index)
            })
        })
    })
}
