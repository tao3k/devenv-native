use std::collections::BTreeSet;
use std::fmt::Write as _;

use crate::contracts::FlowhubGraphTopology;

use super::model::{FlowhubGraphNodeSummary, FlowhubGraphShow};
use super::render_surface::render_label_list;

pub(super) fn render_mermaid_section_lines(show: &FlowhubGraphShow) -> Vec<String> {
    let mut lines = vec!["```mermaid".to_string()];
    lines.extend(show.mermaid.lines().map(ToString::to_string));
    lines.push("```".to_string());
    lines
}

pub(super) fn render_declared_topology_line(topology: Option<FlowhubGraphTopology>) -> String {
    match topology {
        Some(value) => format!("Declared topology: {}", value.as_str()),
        None => "Declared topology: (none)".to_string(),
    }
}

pub(super) fn render_execution_section_lines(show: &FlowhubGraphShow) -> Vec<String> {
    render_execution_summary_lines(show)
}

pub(super) fn render_node_section_lines(show: &FlowhubGraphShow) -> Vec<String> {
    if show.nodes.is_empty() {
        return vec!["- none".to_string()];
    }

    let mut lines = show
        .nodes
        .iter()
        .map(render_execution_node_line)
        .collect::<Vec<_>>();
    if !show.unknown_graph_nodes.is_empty() {
        lines.push(format!(
            "- Undeclared graph nodes: {}.",
            render_label_list(&show.unknown_graph_nodes)
        ));
    }
    lines
}

fn render_execution_summary_lines(show: &FlowhubGraphShow) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(note) = &show.declared_check_surface.note {
        lines.push(format!("- {note}"));
    }

    let entry_nodes = entry_node_labels(show);
    if !entry_nodes.is_empty() {
        lines.push(format!("- Start at {}.", render_label_list(&entry_nodes)));
    }

    let terminal_nodes = terminal_node_labels(show);
    if !terminal_nodes.is_empty() {
        lines.push(format!(
            "- Complete at {}.",
            render_label_list(&terminal_nodes)
        ));
    }

    if !show.cyclic_components.is_empty() {
        let loop_lines = show
            .cyclic_components
            .iter()
            .map(|component| render_label_list(component))
            .collect::<Vec<_>>()
            .join("; ");
        lines.push(format!("- Retry or loop components: {loop_lines}."));
    }

    lines
}

fn entry_node_labels(show: &FlowhubGraphShow) -> Vec<String> {
    let targets = show
        .edges
        .iter()
        .map(|edge| edge.to_label.as_str())
        .collect::<BTreeSet<_>>();

    show.nodes
        .iter()
        .filter(|node| !targets.contains(node.label.as_str()))
        .map(|node| node.label.clone())
        .collect()
}

fn terminal_node_labels(show: &FlowhubGraphShow) -> Vec<String> {
    show.nodes
        .iter()
        .filter(|node| node.next.is_empty())
        .map(|node| node.label.clone())
        .collect()
}

fn render_execution_node_line(node: &FlowhubGraphNodeSummary) -> String {
    let mut prefix = format!("- `{}`", node.label);
    if let Some(kind) = &node.kind {
        let _ = write!(prefix, " [`{kind}`]");
    }

    let mut detail = format!("{} Action: {}", node.role, node.agent_action);
    let _ = write!(detail, ". Next: {}", render_label_list(&node.next));
    if let Some(entry) = &node.exports_entry {
        let _ = write!(detail, ". Entry: `{entry}`");
    }
    if let Some(ready) = &node.exports_ready {
        let _ = write!(detail, ". Ready: `{ready}`");
    }

    format!("{prefix}: {detail}")
}
