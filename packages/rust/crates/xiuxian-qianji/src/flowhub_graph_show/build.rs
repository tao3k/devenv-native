use std::collections::{BTreeMap, BTreeSet};

use crate::flowhub::discover::FlowhubDiscoveredModule;
use crate::flowhub::mermaid_model::MermaidEdge;
use crate::flowhub::{
    FlowhubScenarioIr, MermaidFlowchart, MermaidNodeKind, scenario_graph_label_is_allowed,
};

use super::api::{FlowhubGraphEdgeSummary, FlowhubGraphNodeSummary};
use super::load::ModuleExports;
use super::semantics::graph_node_semantics;

pub(super) fn collect_unknown_graph_nodes(
    flowchart: &MermaidFlowchart,
    allowed_graph_node_labels: &BTreeSet<String>,
) -> Vec<String> {
    flowchart
        .nodes
        .iter()
        .filter(|node| node.kind != MermaidNodeKind::Module)
        .filter(|node| {
            !scenario_graph_label_is_allowed(node.label.as_str(), allowed_graph_node_labels)
        })
        .map(|node| node.label.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn build_next_labels_by_node_id<'a>(
    edges: &'a [MermaidEdge],
    nodes_by_id: &BTreeMap<&'a str, &'a str>,
) -> BTreeMap<&'a str, Vec<String>> {
    let mut next_by_node_id = BTreeMap::<&str, Vec<String>>::new();

    for edge in edges {
        let next_label = nodes_by_id
            .get(edge.to.as_str())
            .copied()
            .unwrap_or(edge.to.as_str())
            .to_string();
        let entry = next_by_node_id.entry(edge.from.as_str()).or_default();
        if !entry.contains(&next_label) {
            entry.push(next_label);
        }
    }

    next_by_node_id
}

pub(super) fn build_graph_node_summaries(
    flowchart: &MermaidFlowchart,
    module_exports: &BTreeMap<String, ModuleExports>,
    next_by_node_id: &BTreeMap<&str, Vec<String>>,
    scenario_ir: Option<&FlowhubScenarioIr>,
) -> Vec<FlowhubGraphNodeSummary> {
    flowchart
        .nodes
        .iter()
        .map(|node| {
            let module_ref = match node.kind {
                MermaidNodeKind::Module => Some(node.label.clone()),
                MermaidNodeKind::Scenario => None,
            };
            let exports = module_ref
                .as_deref()
                .and_then(|module_name| module_exports.get(module_name));
            let node_contract =
                scenario_ir.and_then(|graph| graph.node_contract(node.label.as_str()));
            let (kind, role, agent_action) =
                graph_node_semantics(module_ref.as_deref(), node.label.as_str(), node_contract);

            FlowhubGraphNodeSummary {
                id: node.id.clone(),
                label: node.label.clone(),
                kind,
                role,
                agent_action,
                next: next_by_node_id
                    .get(node.id.as_str())
                    .cloned()
                    .unwrap_or_default(),
                module_ref,
                exports_entry: exports.map(|value| value.entry.clone()),
                exports_ready: exports.map(|value| value.ready.clone()),
            }
        })
        .collect()
}

pub(super) fn build_graph_edge_summaries(
    flowchart: &MermaidFlowchart,
    nodes_by_id: &BTreeMap<&str, &str>,
) -> Vec<FlowhubGraphEdgeSummary> {
    flowchart
        .edges
        .iter()
        .map(|edge| FlowhubGraphEdgeSummary {
            from_label: nodes_by_id
                .get(edge.from.as_str())
                .copied()
                .unwrap_or(edge.from.as_str())
                .to_string(),
            to_label: nodes_by_id
                .get(edge.to.as_str())
                .copied()
                .unwrap_or(edge.to.as_str())
                .to_string(),
        })
        .collect()
}

pub(super) fn module_contract_surface(owning_module: &FlowhubDiscoveredModule) -> Vec<String> {
    let mut entries = vec!["qianji.toml".to_string()];
    if let Some(contract) = &owning_module.manifest.contract {
        entries.extend(contract.required.iter().cloned());
    }
    entries
}
