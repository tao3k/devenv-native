use crate::error::{BpmnEngineError, Result};
use crate::ir_node_api::BpmnNodeKind;
use crate::parser::import::RawPackageDocument;
use std::collections::HashSet;

pub(in crate::parser) fn detect_recursive_call_activity(raw: &RawPackageDocument) -> Result<()> {
    let call_graph = raw
        .processes
        .iter()
        .map(|process| {
            let edges = process
                .nodes
                .iter()
                .filter(|node| node.kind == BpmnNodeKind::SubProcess)
                .filter_map(|node| {
                    node.called_process_ref
                        .clone()
                        .map(|called_process_id| (called_process_id, node.bpmn_id.clone()))
                })
                .collect::<Vec<_>>();
            (process.process_id.clone(), edges)
        })
        .collect::<std::collections::HashMap<_, _>>();

    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    for process_id in call_graph.keys() {
        if let Some(error) = detect_recursive_call_activity_from(
            process_id,
            &call_graph,
            &mut visiting,
            &mut visited,
        ) {
            return Err(error);
        }
    }
    Ok(())
}

fn detect_recursive_call_activity_from(
    process_id: &str,
    call_graph: &std::collections::HashMap<String, Vec<(String, String)>>,
    visiting: &mut HashSet<String>,
    visited: &mut HashSet<String>,
) -> Option<BpmnEngineError> {
    if visited.contains(process_id) {
        return None;
    }

    visiting.insert(process_id.to_string());
    if let Some(edges) = call_graph.get(process_id) {
        for (called_process_id, node_id) in edges {
            if visiting.contains(called_process_id) {
                return Some(BpmnEngineError::UnsupportedSubProcessConfiguration {
                    process_id: process_id.to_string(),
                    node_id: node_id.clone(),
                    detail: "recursive_call_activity",
                });
            }
            if let Some(error) = detect_recursive_call_activity_from(
                called_process_id,
                call_graph,
                visiting,
                visited,
            ) {
                return Some(error);
            }
        }
    }
    visiting.remove(process_id);
    visited.insert(process_id.to_string());
    None
}
