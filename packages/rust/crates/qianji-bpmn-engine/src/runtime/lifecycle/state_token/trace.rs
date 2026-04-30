use crate::runtime::lifecycle::scope::{
    BpmnExecutionTraceEvent, BpmnExecutionTraceEventKind, BpmnInstanceState, BpmnNodeIndex,
    InstanceLifecycle, NodeRuntimeStatus,
};

pub(crate) fn set_node_status(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
    status: NodeRuntimeStatus,
) {
    let changed = if let Some(node_state) = instance.node_states.get_mut(node_index as usize) {
        if node_state.status == status {
            false
        } else {
            node_state.status = status.clone();
            true
        }
    } else {
        false
    };
    if changed {
        push_node_status_trace(instance, node_index, status);
    }
}

pub(crate) fn record_transition(
    instance: &mut BpmnInstanceState,
    now_ms: u64,
    lifecycle: InstanceLifecycle,
) {
    instance.sequence += 1;
    instance.lifecycle = lifecycle;
    instance.updated_at_ms = now_ms;
}

fn push_node_status_trace(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
    status: NodeRuntimeStatus,
) {
    instance.trace.push(BpmnExecutionTraceEvent {
        sequence: next_trace_sequence(instance),
        process: instance.process.clone(),
        kind: BpmnExecutionTraceEventKind::NodeStatus,
        node_index: Some(node_index),
        edge_index: None,
        status: Some(status),
    });
}

pub(super) fn push_flow_take_trace(
    instance: &mut BpmnInstanceState,
    edge_index: u32,
    target_node_index: BpmnNodeIndex,
) {
    instance.trace.push(BpmnExecutionTraceEvent {
        sequence: next_trace_sequence(instance),
        process: instance.process.clone(),
        kind: BpmnExecutionTraceEventKind::FlowTake,
        node_index: Some(target_node_index),
        edge_index: Some(edge_index),
        status: None,
    });
}

fn next_trace_sequence(instance: &BpmnInstanceState) -> u64 {
    u64::try_from(instance.trace.len())
        .unwrap_or(u64::MAX)
        .saturating_add(1)
}
