use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::model::MermaidNode;
use super::{MermaidFlowchart, MermaidNodeKind};

pub(crate) fn validate_mermaid_flowchart(
    flowchart: &MermaidFlowchart,
    allowed_graph_node_labels: &BTreeSet<String>,
) -> Result<(), String> {
    validate_has_edges(flowchart)?;
    let nodes_by_id = node_labels_by_id(flowchart);
    validate_edge_endpoints(flowchart, &nodes_by_id)?;
    validate_allowed_graph_nodes(flowchart, allowed_graph_node_labels)?;
    let module_nodes = collect_module_nodes(flowchart);
    validate_has_module_nodes(&module_nodes)?;
    let module_edges = collect_module_backbone_edges(flowchart, &module_nodes);
    validate_module_backbone(flowchart, &module_nodes, &module_edges)?;
    validate_connected_module_backbone(flowchart, &module_nodes, &module_edges)?;
    Ok(())
}

fn validate_has_edges(flowchart: &MermaidFlowchart) -> Result<(), String> {
    if flowchart.edges.is_empty() {
        return Err("scenario-case graph must declare at least one edge".to_string());
    }
    Ok(())
}

fn node_labels_by_id(flowchart: &MermaidFlowchart) -> BTreeMap<&str, &str> {
    flowchart
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node.label.as_str()))
        .collect::<BTreeMap<_, _>>()
}

fn validate_edge_endpoints(
    flowchart: &MermaidFlowchart,
    nodes_by_id: &BTreeMap<&str, &str>,
) -> Result<(), String> {
    for edge in &flowchart.edges {
        if !nodes_by_id.contains_key(edge.from.as_str())
            || !nodes_by_id.contains_key(edge.to.as_str())
        {
            return Err(format!(
                "scenario-case graph contains an edge `{}` -> `{}` whose endpoint is not declared as a Mermaid node",
                edge.from, edge.to
            ));
        }
    }
    Ok(())
}

fn validate_allowed_graph_nodes(
    flowchart: &MermaidFlowchart,
    allowed_graph_node_labels: &BTreeSet<String>,
) -> Result<(), String> {
    let undeclared_graph_node_labels = flowchart
        .nodes
        .iter()
        .filter(|node| node.kind != MermaidNodeKind::Module)
        .filter(|node| {
            !scenario_graph_label_is_allowed(node.label.as_str(), allowed_graph_node_labels)
        })
        .map(|node| node.label.as_str())
        .collect::<Vec<_>>();
    if undeclared_graph_node_labels.is_empty() {
        return Ok(());
    }
    Err(format!(
        "scenario-case graph `{}` contains undeclared graph nodes: {}",
        flowchart.merimind_graph_name,
        undeclared_graph_node_labels.join(", ")
    ))
}

pub(crate) fn scenario_graph_label_is_allowed(
    label: &str,
    allowed_graph_node_labels: &BTreeSet<String>,
) -> bool {
    allowed_graph_node_labels.contains(label)
}

pub(crate) fn normalize_graph_node_label(label: &str) -> String {
    let mut normalized = String::with_capacity(label.len());
    let mut inside_angle = false;

    for character in label.chars() {
        match character {
            '<' => inside_angle = true,
            '>' => inside_angle = false,
            _ if !inside_angle => normalized.push(character),
            _ => {}
        }
    }

    normalized
}

fn collect_module_nodes(flowchart: &MermaidFlowchart) -> Vec<&MermaidNode> {
    flowchart
        .nodes
        .iter()
        .filter(|node| node.kind == MermaidNodeKind::Module)
        .collect::<Vec<_>>()
}

fn validate_has_module_nodes(module_nodes: &[&MermaidNode]) -> Result<(), String> {
    if module_nodes.is_empty() {
        return Err("scenario-case graph must expose at least one Flowhub module node".to_string());
    }
    Ok(())
}

fn collect_module_backbone_edges<'a>(
    flowchart: &'a MermaidFlowchart,
    module_nodes: &[&MermaidNode],
) -> Vec<(&'a str, &'a str)> {
    let module_node_ids = module_nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<BTreeSet<_>>();
    flowchart
        .edges
        .iter()
        .filter(|edge| {
            module_node_ids.contains(edge.from.as_str())
                && module_node_ids.contains(edge.to.as_str())
        })
        .map(|edge| (edge.from.as_str(), edge.to.as_str()))
        .collect::<Vec<_>>()
}

fn validate_module_backbone(
    _flowchart: &MermaidFlowchart,
    module_nodes: &[&MermaidNode],
    module_edges: &[(&str, &str)],
) -> Result<(), String> {
    if module_nodes.len() == 1 {
        return Ok(());
    }
    if module_edges.is_empty() {
        return Err(
            "scenario-case graph must declare at least one edge between Flowhub module nodes"
                .to_string(),
        );
    }

    let mut module_nodes_with_backbone_edge = BTreeSet::new();
    for (from, to) in module_edges {
        module_nodes_with_backbone_edge.insert(*from);
        module_nodes_with_backbone_edge.insert(*to);
    }
    let isolated_module_labels = module_nodes
        .iter()
        .filter(|node| !module_nodes_with_backbone_edge.contains(node.id.as_str()))
        .map(|node| node.label.as_str())
        .collect::<Vec<_>>();
    if isolated_module_labels.is_empty() {
        return Ok(());
    }
    Err(format!(
        "scenario-case graph contains Flowhub module nodes without a module-backbone edge: {}",
        isolated_module_labels.join(", ")
    ))
}

fn validate_connected_module_backbone(
    _flowchart: &MermaidFlowchart,
    module_nodes: &[&MermaidNode],
    module_edges: &[(&str, &str)],
) -> Result<(), String> {
    if module_nodes.len() == 1 {
        return Ok(());
    }

    let mut adjacency = BTreeMap::<&str, BTreeSet<&str>>::new();
    for (from, to) in module_edges {
        adjacency.entry(*from).or_default().insert(*to);
        adjacency.entry(*to).or_default().insert(*from);
    }

    let start = module_nodes[0].id.as_str();
    let mut queue = VecDeque::from([start]);
    let mut visited = BTreeSet::new();
    while let Some(node_id) = queue.pop_front() {
        if !visited.insert(node_id) {
            continue;
        }
        for neighbor in adjacency.get(node_id).into_iter().flatten() {
            if !visited.contains(neighbor) {
                queue.push_back(neighbor);
            }
        }
    }

    let disconnected_module_labels = module_nodes
        .iter()
        .filter(|node| !visited.contains(node.id.as_str()))
        .map(|node| node.label.as_str())
        .collect::<Vec<_>>();
    if disconnected_module_labels.is_empty() {
        return Ok(());
    }
    Err(format!(
        "scenario-case graph contains disconnected Flowhub module backbone nodes: {}",
        disconnected_module_labels.join(", ")
    ))
}

#[cfg(test)]
#[path = "../../../tests/unit/flowhub/mermaid/validate.rs"]
mod tests;
