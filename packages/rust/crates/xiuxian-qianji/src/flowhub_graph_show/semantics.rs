use crate::contracts::FlowhubGraphNodeKind;
use crate::flowhub::FlowhubScenarioNodeIr;

use super::render_surface::render_file_list;

pub(super) fn graph_node_semantics(
    module_ref: Option<&str>,
    label: &str,
    node_contract: Option<&FlowhubScenarioNodeIr>,
) -> (Option<FlowhubGraphNodeKind>, String, String) {
    if let Some(node_contract) = node_contract {
        return (
            node_contract
                .kind
                .as_ref()
                .map(|kind| FlowhubGraphNodeKind::new(kind.as_str())),
            node_contract
                .role
                .clone()
                .unwrap_or_else(|| inferred_graph_node_role(label, node_contract)),
            node_contract
                .agent_action
                .clone()
                .unwrap_or_else(|| inferred_graph_node_action(node_contract)),
        );
    }

    if module_ref.is_some() {
        return (
            None,
            "registered Flowhub module is present in the Mermaid graph without a declared graph-node contract"
                .to_string(),
            "add a matching `[[graph.node]]` entry before relying on semantic guidance for this module node"
                .to_string(),
        );
    }

    (
        None,
        "node is outside the declared Flowhub graph contract vocabulary".to_string(),
        "do not rely on this node until the Flowhub graph contract is corrected".to_string(),
    )
}

fn inferred_graph_node_role(label: &str, node_contract: &FlowhubScenarioNodeIr) -> String {
    if node_contract.kind.as_deref() == Some("gate") || label == "done gate" {
        return "allow completion only when the declared graph-step contracts are satisfied"
            .to_string();
    }
    if label == "diagnostics" {
        return "capture blocking diagnostics for bounded-surface repair".to_string();
    }
    if !node_contract.merge_target.is_empty() {
        return "materialize localized outputs that can be merged into the persistent target surface"
            .to_string();
    }
    if !node_contract.writes.is_empty() || node_contract.checkpoint.is_some() {
        return "materialize localized bounded-work artifacts for this graph step".to_string();
    }
    "follow the declared graph-step contract".to_string()
}

fn inferred_graph_node_action(node_contract: &FlowhubScenarioNodeIr) -> String {
    let mut parts = Vec::new();
    if let Some(checkpoint) = &node_contract.checkpoint {
        parts.push(format!("write checkpoint `{checkpoint}`"));
    }
    if !node_contract.writes.is_empty() {
        parts.push(format!(
            "write localized artifacts {}",
            render_file_list(&node_contract.writes)
        ));
    }
    if !node_contract.merge_target.is_empty() {
        parts.push(format!(
            "prepare canonical merge targets {}",
            render_file_list(&node_contract.merge_target)
        ));
    }

    if parts.is_empty() {
        "follow the declared node contract".to_string()
    } else {
        parts.join("; ")
    }
}
