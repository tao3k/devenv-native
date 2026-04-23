use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::error::QianjiError;
use crate::flowhub::{MermaidFlowchart, normalize_graph_node_label};

use super::scenario_ir_annotations::FlowhubGraphAnnotations;
use super::scenario_ir_model::FlowhubScenarioNodeIr;

pub(super) fn compile_enriched_nodes(
    graph_path: &Path,
    flowchart: &MermaidFlowchart,
    annotations: &FlowhubGraphAnnotations,
) -> Result<Vec<FlowhubScenarioNodeIr>, QianjiError> {
    let mut nodes = Vec::new();
    let mut seen_labels = BTreeSet::new();

    for (node_ref, node_annotations) in &annotations.nodes {
        let label =
            resolve_annotation_node_label(flowchart, node_ref.as_str()).map_err(|error| {
                QianjiError::Topology(format!(
                    "Failed to compile Flowhub Mermaid contract `{}`: {error}",
                    graph_path.display()
                ))
            })?;
        let normalized_label = normalize_graph_node_label(label.as_str());
        if !seen_labels.insert(normalized_label.clone()) {
            return Err(QianjiError::Topology(format!(
                "Failed to compile Flowhub Mermaid contract `{}`: duplicate node contract for `{}`",
                graph_path.display(),
                label
            )));
        }

        nodes.push(FlowhubScenarioNodeIr {
            label,
            kind: node_annotations.kind.clone(),
            role: node_annotations.role.clone(),
            agent_action: node_annotations.agent_action.clone(),
            checkpoint: node_annotations.checkpoint.clone(),
            writes: node_annotations.writes.clone(),
            merge_target: node_annotations.merge_target.clone(),
        });
    }

    let node_order = flowchart
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (normalize_graph_node_label(node.label.as_str()), index))
        .collect::<BTreeMap<_, _>>();
    nodes.sort_by_key(|node| {
        node_order
            .get(&normalize_graph_node_label(node.label.as_str()))
            .copied()
            .unwrap_or(usize::MAX)
    });

    Ok(nodes)
}

fn resolve_annotation_node_label(
    flowchart: &MermaidFlowchart,
    node_ref: &str,
) -> Result<String, String> {
    let exact_matches = flowchart
        .nodes
        .iter()
        .filter(|node| node.id == node_ref || node.label == node_ref)
        .map(|node| node.label.clone())
        .collect::<BTreeSet<_>>();
    if let Some(label) = single_match(exact_matches) {
        return Ok(label);
    }

    let normalized_ref = normalize_graph_node_label(node_ref);
    let normalized_matches = flowchart
        .nodes
        .iter()
        .filter(|node| normalize_graph_node_label(node.label.as_str()) == normalized_ref)
        .map(|node| node.label.clone())
        .collect::<BTreeSet<_>>();
    if let Some(label) = single_match(normalized_matches) {
        return Ok(label);
    }

    Err(format!(
        "node annotation `{node_ref}` does not match any Mermaid node id or label"
    ))
}

fn single_match(matches: BTreeSet<String>) -> Option<String> {
    if matches.len() == 1 {
        matches.into_iter().next()
    } else {
        None
    }
}
