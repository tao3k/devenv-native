//! Terminal rendering for Orgize read-model commands.

use crate::ClientContext;
use std::collections::BTreeMap;
use xiuxian_graph_core::{CompactMermaidGraph, GraphEdge, GraphNode, GraphProjection};

use super::archive::archive_target_for_row;
use super::model::{AgentOrgTaskListRow, ResolvedReadModelSettings};
use super::row_view::{display_source_path, property_value, task_repeater_labels};
use super::section_lens::TaskSectionLens;

pub(super) fn render_task_list_row(
    index: usize,
    row: &AgentOrgTaskListRow,
    context: &ClientContext,
) {
    println!();
    println!("[TASK{index:03}] {}", row.title);
    println!("orgid: {}", row.orgid);
    if let Some(todo_state) = row.todo_state.as_deref() {
        println!("state: {todo_state}");
    }
    if !row.effective_tags.is_empty() {
        println!("tags: {}", row.effective_tags.join(":"));
    }
    println!(
        "source: {}:{}",
        display_source_path(&row.source_path, context),
        row.source_line
    );
    if let Some(scheduled) = row.scheduled.as_deref() {
        println!("scheduled: {scheduled}");
    }
    if let Some(deadline) = row.deadline.as_deref() {
        println!("deadline: {deadline}");
    }
    let repeaters = task_repeater_labels(row);
    if !repeaters.is_empty() {
        println!("repeat: {}", repeaters.join(", "));
    }
    if let Some(closed) = row.closed.as_deref() {
        println!("closed: {closed}");
    }
    if let Some(next_action) = property_value(row, "NEXT_ACTION") {
        println!("next: {next_action}");
    }
    if let Some(resume_query) = property_value(row, "RESUME_QUERY") {
        println!("resume: {resume_query}");
    }
    println!(
        "show: wendao-client orgize ogrid-show --cached --id {}",
        row.orgid
    );
}

pub(super) fn render_ogrid_show_row(
    row: &AgentOrgTaskListRow,
    section: &str,
    context: &ClientContext,
    full: bool,
) {
    println!("title: {}", row.title);
    println!(
        "source: {}:{}",
        display_source_path(&row.source_path, context),
        row.source_line
    );
    if row.outline_path.len() > 1 {
        println!("outline: {}", row.outline_path.join(" / "));
    }
    if let Some(next_action) = property_value(row, "NEXT_ACTION") {
        println!("next: {next_action}");
    }
    println!();
    if full {
        println!("section:");
        println!("{section}");
    } else {
        render_ogrid_recovery_view(section);
        println!();
        println!(
            "full: wendao-client orgize ogrid-show --cached --id {} --full",
            row.orgid
        );
    }
}

fn render_ogrid_recovery_view(section: &str) {
    let view = TaskSectionLens::from_section(section);
    if let Some(progress) = view.progress_label() {
        println!("checklist-progress: {progress}");
    }
    if let Some(next_unchecked) = view.next_unchecked.as_deref() {
        println!("next-unchecked: {next_unchecked}");
    }
    if !view.checkboxes.is_empty() {
        println!("checklist:");
        render_limited_lines(&view.checkboxes, 24);
    }
    if !view.direct_children.is_empty() {
        println!("children:");
        render_limited_lines(&view.direct_children, 24);
    }
    if view.checkboxes.is_empty() && view.direct_children.is_empty() {
        println!("summary: no direct checklist or child headings");
    }
}

fn render_limited_lines(lines: &[String], limit: usize) {
    for line in lines.iter().take(limit) {
        println!("{line}");
    }
    if lines.len() > limit {
        println!("... omitted {} more lines", lines.len() - limit);
    }
}

pub(super) fn render_recovery_candidate_row(
    index: usize,
    row: &AgentOrgTaskListRow,
    context: &ClientContext,
) {
    if index > 1 {
        println!();
    }
    println!("title: {}", row.title);
    println!(
        "source: {}:{}",
        display_source_path(&row.source_path, context),
        row.source_line
    );
    if !row.effective_tags.is_empty() {
        println!("tags: {}", row.effective_tags.join(":"));
    }
    render_probe_properties(row);
    println!(
        "show: wendao-client orgize ogrid-show --cached --id {}",
        row.orgid
    );
}

pub(super) fn render_probe_candidate_row(
    index: usize,
    row: &AgentOrgTaskListRow,
    context: &ClientContext,
) {
    if index > 1 {
        println!();
    }
    println!("title: {}", row.title);
    println!(
        "source: {}:{}",
        display_source_path(&row.source_path, context),
        row.source_line
    );
    render_probe_properties(row);
    render_probe_recovery_evidence(row);
    println!(
        "show: wendao-client orgize ogrid-show --cached --id {}",
        row.orgid
    );
}

fn render_probe_recovery_evidence(row: &AgentOrgTaskListRow) {
    let Some(view) = task_row_section_lens(row) else {
        return;
    };
    if let Some(progress) = view.progress_label() {
        println!("checklist-progress: {progress}");
    }
    if let Some(next_unchecked) = view.next_unchecked.as_deref() {
        println!("next-unchecked: {next_unchecked}");
    }
}

fn render_probe_properties(row: &AgentOrgTaskListRow) {
    for key in ["PACKAGE", "SLICE"] {
        render_probe_property(row, key);
    }
    if let Some(sdd) = real_sdd_value(row) {
        println!(
            "{}",
            compact_task_sdd_mermaid(row, &[TaskSddEdge::new("sdd", "SDD", sdd)])
        );
    }
    render_probe_property(row, "NEXT_ACTION");
}

fn render_probe_property(row: &AgentOrgTaskListRow, key: &str) {
    if let Some(value) = property_value(row, key)
        && !value.trim().is_empty()
        && value.trim() != "none"
    {
        println!("{}: {value}", probe_property_label(key));
    }
}

fn probe_property_label(key: &str) -> &'static str {
    match key {
        "NEXT_ACTION" => "next",
        "PACKAGE" => "package",
        "SLICE" => "slice",
        _ => "property",
    }
}

fn real_sdd_value(row: &AgentOrgTaskListRow) -> Option<&str> {
    property_value(row, "SDD")
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "none")
}

fn task_row_section_lens(row: &AgentOrgTaskListRow) -> Option<TaskSectionLens> {
    let source = std::fs::read_to_string(row.source_path.as_str()).ok()?;
    let start = usize::try_from(row.source_range_start).ok()?;
    let end = usize::try_from(row.source_range_end).ok()?;
    let section = source.get(start..end)?;
    Some(TaskSectionLens::from_section(section))
}

pub(super) fn render_task_sdd_graph(row: &AgentOrgTaskListRow, context: &ClientContext) {
    let edges = task_sdd_edges(row);

    println!("task: {}", row.title);
    println!(
        "source: {}:{}",
        display_source_path(&row.source_path, context),
        row.source_line
    );
    println!("{}", compact_task_sdd_mermaid(row, edges.as_slice()));
    for edge in &edges {
        if edge.property == "SDD" {
            println!(
                "inspect-sdd: wendao-client orgize sdd status {}",
                edge.value
            );
        }
    }
}

struct TaskSddEdge<'a> {
    label: &'static str,
    property: &'static str,
    value: &'a str,
}

impl<'a> TaskSddEdge<'a> {
    const fn new(label: &'static str, property: &'static str, value: &'a str) -> Self {
        Self {
            label,
            property,
            value,
        }
    }
}

fn task_sdd_edges(row: &AgentOrgTaskListRow) -> Vec<TaskSddEdge<'_>> {
    [("sdd", "SDD"), ("package", "PACKAGE"), ("slice", "SLICE")]
        .into_iter()
        .filter_map(|(label, property)| {
            property_value(row, property)
                .map(str::trim)
                .filter(|value| !value.is_empty() && *value != "none")
                .map(|value| TaskSddEdge::new(label, property, value))
        })
        .collect()
}

fn compact_task_sdd_mermaid(row: &AgentOrgTaskListRow, edges: &[TaskSddEdge<'_>]) -> String {
    let projection = task_sdd_graph_projection(row, edges);
    let result = CompactMermaidGraph::new().render(&projection);
    debug_assert!(
        result.is_ok(),
        "generated task SDD Mermaid graph should parse through xiuxian-graph-core: {:?}",
        result.as_ref().err()
    );
    let diagram = result.unwrap_or_else(|_| fallback_task_sdd_diagram(row, edges));
    format!("graph: {diagram}")
}

fn task_sdd_graph_projection(
    row: &AgentOrgTaskListRow,
    edges: &[TaskSddEdge<'_>],
) -> GraphProjection {
    let mut projection = GraphProjection::new();
    projection.push_node(GraphNode::new("T", format!("task:{}", row.orgid)));
    for (index, edge) in edges.iter().enumerate() {
        let node_id = format!("N{index}");
        projection.push_node(GraphNode::new(
            node_id.clone(),
            format!("{}:{}", edge.label, edge.value),
        ));
        projection.push_edge(GraphEdge::new("T", node_id));
    }
    projection
}

fn fallback_task_sdd_diagram(row: &AgentOrgTaskListRow, edges: &[TaskSddEdge<'_>]) -> String {
    let mut diagram = format!(
        "flowchart LR;T[\"task:{}\"]",
        escape_mermaid_label(row.orgid.as_str())
    );
    for (index, edge) in edges.iter().enumerate() {
        diagram.push_str(
            format!(
                "-->N{index}[\"{}:{}\"]",
                edge.label,
                escape_mermaid_label(edge.value)
            )
            .as_str(),
        );
        if index + 1 < edges.len() {
            diagram.push_str(";T");
        }
    }
    diagram
}

fn escape_mermaid_label(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

pub(super) fn render_tag_counts(rows: &[&AgentOrgTaskListRow]) {
    let mut counts = BTreeMap::<String, usize>::new();
    for row in rows {
        for tag in &row.effective_tags {
            *counts.entry(tag.clone()).or_default() += 1;
        }
    }
    if counts.is_empty() {
        return;
    }

    println!();
    println!("Tags");
    for (tag, count) in counts {
        println!("{tag}: {count}");
    }
}

pub(super) fn render_report_section(
    title: &str,
    rows: &[&AgentOrgTaskListRow],
    limit: usize,
    context: &ClientContext,
) {
    println!();
    println!("{title}: {}", rows.len());
    for (index, row) in rows.iter().take(limit).enumerate() {
        render_task_list_row(index + 1, row, context);
    }
}

pub(super) fn render_archive_plan_row(
    index: usize,
    row: &AgentOrgTaskListRow,
    settings: &ResolvedReadModelSettings,
    context: &ClientContext,
) {
    println!();
    println!("[ARCHIVE{index:03}] {}", row.title);
    println!(
        "source: {}:{}",
        display_source_path(&row.source_path, context),
        row.source_line
    );
    println!(
        "range: {}..{}",
        row.source_range_start, row.source_range_end
    );
    println!(
        "target: {}",
        display_source_path(
            archive_target_for_row(row, settings, context)
                .to_string_lossy()
                .as_ref(),
            context
        )
    );
    if !row.effective_tags.is_empty() {
        println!("tags: {}", row.effective_tags.join(":"));
    }
}

pub(super) fn render_archive_target_summary(
    rows: &[&AgentOrgTaskListRow],
    settings: &ResolvedReadModelSettings,
    context: &ClientContext,
) {
    let mut counts = BTreeMap::<String, usize>::new();
    for row in rows {
        let target = archive_target_for_row(row, settings, context);
        let target = display_source_path(target.to_string_lossy().as_ref(), context);
        *counts.entry(target).or_default() += 1;
    }
    if counts.is_empty() {
        return;
    }

    println!();
    println!("Archive Targets");
    for (target, count) in counts {
        println!("{target}: {count}");
    }
}
