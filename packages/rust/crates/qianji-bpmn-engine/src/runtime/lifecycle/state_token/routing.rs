use crate::runtime::lifecycle::scope::{
    BpmnEngineError, BpmnNodeIndex, BpmnNodeKind, BpmnProcessSpec, Result,
};

pub(crate) fn find_single_start_node(process: &BpmnProcessSpec) -> Result<BpmnNodeIndex> {
    let mut start_nodes = process
        .nodes
        .iter()
        .filter(|node| node.kind == BpmnNodeKind::StartEvent)
        .map(|node| node.index);
    let Some(start_node_index) = start_nodes.next() else {
        return Err(BpmnEngineError::MissingRequiredProcessElement {
            process_id: (process.key.process_id.to_string()).into(),
            element: "start event",
        });
    };
    if start_nodes.next().is_some() {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "advance_instance_multiple_start_events",
        });
    }
    Ok(start_node_index)
}

pub(crate) fn resolve_single_outgoing_edge(
    process: &BpmnProcessSpec,
    node_index: BpmnNodeIndex,
    operation: &'static str,
) -> Result<u32> {
    let outgoing = process.outgoing_edge_indices(node_index);
    if outgoing.len() != 1 {
        return Err(BpmnEngineError::UnsupportedOperation { operation });
    }
    Ok(outgoing[0])
}
