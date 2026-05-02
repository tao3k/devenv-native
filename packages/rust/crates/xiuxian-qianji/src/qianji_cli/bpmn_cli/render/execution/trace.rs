use crate::qianji_cli::bpmn_cli::deps::{
    BpmnExecutionTraceEvent, BpmnExecutionTraceEventKind, BpmnProcessSpec, QianjiBpmnSession,
};

use crate::qianji_cli::bpmn_cli::render::support::{
    bpmn_node_kind_label, node_runtime_status_label,
};

pub(super) fn render_bpmn_execution_trace(session: &QianjiBpmnSession) -> String {
    serde_json::to_string_pretty(&bpmn_execution_trace_values(
        session,
        &session.instance().trace,
    ))
    .unwrap_or_else(|error| format!("[{{\"serialization_error\":\"{error}\"}}]"))
}

pub(crate) fn render_bpmn_execution_trace_stream_lines(
    session: &QianjiBpmnSession,
    events: &[BpmnExecutionTraceEvent],
) -> Vec<String> {
    bpmn_execution_trace_values(session, events)
        .into_iter()
        .filter_map(|event| serde_json::to_string(&event).ok())
        .map(|event| format!("@@QIANJI_TRACE {event}"))
        .collect()
}

fn bpmn_execution_trace_values(
    session: &QianjiBpmnSession,
    events: &[BpmnExecutionTraceEvent],
) -> Vec<serde_json::Value> {
    events
        .iter()
        .map(|event| {
            let process = session
                .package()
                .find_process_position(event.process.process_id.as_ref())
                .map(|(_, process)| process);
            match &event.kind {
                BpmnExecutionTraceEventKind::NodeStatus => {
                    let node = node_by_optional_index(process, event.node_index);
                    let node_id = node_id_by_index(node, event.node_index);
                    serde_json::json!({
                        "sequence": event.sequence,
                        "kind": "node_status",
                        "process_id": event.process.process_id.as_ref(),
                        "node_id": node_id,
                        "node_kind": node.map(|node| bpmn_node_kind_label(&node.kind)),
                        "status": event.status.as_ref().map_or("unknown", node_runtime_status_label),
                    })
                }
                BpmnExecutionTraceEventKind::FlowTake => {
                    let (source_id, target_id) =
                        flow_endpoint_ids(process, event.edge_index, event.node_index);
                    serde_json::json!({
                        "sequence": event.sequence,
                        "kind": "flow_take",
                        "process_id": event.process.process_id.as_ref(),
                        "source_id": source_id,
                        "target_id": target_id,
                    })
                }
            }
        })
        .collect::<Vec<_>>()
}

fn node_by_optional_index(
    process: Option<&BpmnProcessSpec>,
    node_index: Option<qianji_bpmn_engine::BpmnNodeIndex>,
) -> Option<&qianji_bpmn_engine::BpmnNodeSpec> {
    let node_index = node_index?;
    process.and_then(|process| node_by_index(process, node_index))
}

fn node_id_by_index(
    node: Option<&qianji_bpmn_engine::BpmnNodeSpec>,
    node_index: Option<qianji_bpmn_engine::BpmnNodeIndex>,
) -> String {
    let Some(node_index) = node_index else {
        return String::new();
    };
    node.map_or_else(|| node_index.to_string(), |node| node.bpmn_id.to_string())
}

fn flow_endpoint_ids(
    process: Option<&BpmnProcessSpec>,
    edge_index: Option<u32>,
    fallback_target_node_index: Option<qianji_bpmn_engine::BpmnNodeIndex>,
) -> (String, String) {
    let fallback_target = fallback_target_node_index
        .map(|index| index.to_string())
        .unwrap_or_default();
    let (Some(process), Some(edge_index)) = (process, edge_index) else {
        return (String::new(), fallback_target);
    };
    let Some(edge) = edge_by_index(process, edge_index) else {
        return (String::new(), fallback_target);
    };
    let source_id = node_by_index(process, edge.from)
        .map_or_else(|| edge.from.to_string(), |node| node.bpmn_id.to_string());
    let target_id = node_by_index(process, edge.to)
        .map_or_else(|| edge.to.to_string(), |node| node.bpmn_id.to_string());
    (source_id, target_id)
}

fn node_by_index(
    process: &BpmnProcessSpec,
    node_index: qianji_bpmn_engine::BpmnNodeIndex,
) -> Option<&qianji_bpmn_engine::BpmnNodeSpec> {
    process.nodes.get(node_index as usize)
}

fn edge_by_index(
    process: &BpmnProcessSpec,
    edge_index: u32,
) -> Option<&qianji_bpmn_engine::BpmnEdgeSpec> {
    process.edges.get(edge_index as usize)
}
