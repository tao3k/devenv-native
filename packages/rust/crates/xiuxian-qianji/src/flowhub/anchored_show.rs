use std::path::Path;

use crate::error::QianjiError;
use crate::markdown::{MarkdownShowSection, render_show_surface};
use crate::workdir::{
    WorkdirAllowedNextIssue, WorkdirCurrentNodeIssue, load_workdir_runtime_state,
};

use super::anchor::{resolve_anchor_manifest_path, resolve_anchored_graph};
use super::graph_show::{
    display_graph_path, graph_show_sections, load_flowhub_graph_runtime_contract,
};
use super::{FlowhubGraphShow, show_flowhub_graph};

/// Render one anchor-resolved Flowhub scenario brief, optionally overlaying
/// run-local runtime state from `--dir`.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the anchor manifest cannot be
/// resolved, the requested scenario is not declared by the anchor, or the
/// underlying Mermaid graph contract cannot be loaded.
pub fn show_flowhub_anchored_scenario(
    anchor: impl AsRef<Path>,
    scenario_ref: &str,
    workdir: Option<&Path>,
) -> Result<String, QianjiError> {
    let anchor_manifest_path = resolve_anchor_manifest_path(anchor.as_ref());
    let graph_path = resolve_anchored_graph(&anchor_manifest_path, scenario_ref)?;
    let graph_show = show_flowhub_graph(&graph_path)?;

    let mut metadata = vec![
        format!(
            "Scenario: {}",
            graph_show
                .scenario_id
                .as_deref()
                .unwrap_or(graph_show.merimind_graph_name.as_str())
        ),
        format!("Anchor: {}", display_graph_path(&anchor_manifest_path)),
        format!("Graph: {}", display_graph_path(&graph_show.graph_path)),
        format!(
            "Location: {}",
            display_graph_path(
                anchor_manifest_path
                    .parent()
                    .unwrap_or(anchor_manifest_path.as_path())
            )
        ),
    ];
    if let Some(workdir) = workdir {
        metadata.push(format!("Run: {}", display_graph_path(workdir)));
    }

    let mut sections = vec![MarkdownShowSection {
        title: "Goal".into(),
        lines: render_goal_section_lines(&graph_show),
    }];
    if let Some(workdir) = workdir {
        sections.extend(build_runtime_sections(&graph_show, workdir)?);
    }
    sections.extend(graph_show_sections(&graph_show));

    Ok(render_show_surface("Execution Brief", &metadata, &sections))
}

fn build_runtime_sections(
    graph_show: &FlowhubGraphShow,
    workdir: &Path,
) -> Result<Vec<MarkdownShowSection<'static>>, QianjiError> {
    let (flowchart, scenario_ir) = load_flowhub_graph_runtime_contract(&graph_show.graph_path)?;
    let runtime_state = load_workdir_runtime_state(workdir, &flowchart)?;
    let current_node_contract = runtime_state
        .current_node
        .resolved
        .as_ref()
        .and_then(|current| {
            scenario_ir
                .as_ref()
                .and_then(|graph| graph.node_contract(current.label.as_str()))
        });

    Ok(vec![
        MarkdownShowSection {
            title: "Current State".into(),
            lines: render_current_state_section_lines(workdir, &runtime_state),
        },
        MarkdownShowSection {
            title: "Writable Surface For This Step".into(),
            lines: render_writable_surface_section_lines(current_node_contract),
        },
        MarkdownShowSection {
            title: "Merge Target For This Step".into(),
            lines: render_merge_target_section_lines(current_node_contract),
        },
        MarkdownShowSection {
            title: "Success Condition".into(),
            lines: render_success_condition_section_lines(current_node_contract),
        },
    ])
}

fn render_goal_section_lines(graph_show: &FlowhubGraphShow) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(description) = &graph_show.description {
        lines.push(format!("- {description}"));
    }
    if let Some(note) = &graph_show.declared_check_surface.note {
        lines.push(format!("- {note}"));
    }
    if lines.is_empty() {
        lines.push(format!(
            "- Execute `{}` through the Flowhub-declared graph contract.",
            graph_show.merimind_graph_name
        ));
    }
    lines
}

fn render_current_state_section_lines(
    workdir: &Path,
    runtime_state: &crate::workdir::WorkdirRuntimeState,
) -> Vec<String> {
    let mut lines = vec![format!("Workdir: {}", display_graph_path(workdir))];
    match (
        runtime_state.current_node.resolved.as_ref(),
        runtime_state.current_node.issue.as_ref(),
        runtime_state.current_node.raw_ref.as_ref(),
    ) {
        (Some(current), _, _) => lines.push(format!("Current node: {}", current.label)),
        (_, Some(WorkdirCurrentNodeIssue::MissingField), _) => lines.push(
            "Current node: invalid `state/current_node.toml` (missing `current_node`)".to_string(),
        ),
        (_, Some(WorkdirCurrentNodeIssue::UnknownNode(_)), Some(raw_ref)) => lines.push(format!(
            "Current node: invalid `{raw_ref}` (not present in graph)"
        )),
        _ => lines.push("Current node: (not set)".to_string()),
    }

    if runtime_state.allowed_next.raw_refs.is_some() {
        match runtime_state.allowed_next.issue.as_ref() {
            Some(WorkdirAllowedNextIssue::InvalidJson(_)) => lines.push(
                "Allowed next: invalid `state/allowed_next.json` (expected JSON string array)"
                    .to_string(),
            ),
            Some(WorkdirAllowedNextIssue::UnknownNode(next_ref)) => lines.push(format!(
                "Allowed next: invalid `{next_ref}` (not present in graph)"
            )),
            None => lines.push(format!(
                "Allowed next: {}",
                render_label_list(runtime_state.allowed_next.resolved_labels.as_slice())
            )),
        }
    } else if !runtime_state.allowed_next.expected_labels.is_empty() {
        lines.push(format!(
            "Graph next: {}",
            render_label_list(runtime_state.allowed_next.expected_labels.as_slice())
        ));
    } else {
        lines.push("Allowed next: (not set)".to_string());
    }

    if runtime_state.allowed_next.raw_refs.is_some()
        && runtime_state.allowed_next.issue.is_none()
        && runtime_state.allowed_next.expected_labels != runtime_state.allowed_next.resolved_labels
    {
        lines.push(format!(
            "Allowed-next drift: state declares {}, graph allows {}",
            render_label_list(runtime_state.allowed_next.resolved_labels.as_slice()),
            render_label_list(runtime_state.allowed_next.expected_labels.as_slice()),
        ));
    }

    lines
}

fn render_writable_surface_section_lines(
    current_node_contract: Option<&super::scenario_ir::FlowhubScenarioNodeIr>,
) -> Vec<String> {
    let Some(current_node_contract) = current_node_contract else {
        return vec![
            "- Set `state/current_node.toml` to a Mermaid node id or label to narrow the writable surface."
                .to_string(),
        ];
    };

    let mut lines = Vec::new();
    if let Some(checkpoint) = &current_node_contract.checkpoint {
        lines.push(format!("- {checkpoint}"));
    }
    lines.extend(
        current_node_contract
            .writes
            .iter()
            .map(|write| format!("- {write}")),
    );
    if lines.is_empty() {
        lines.push("- Current node does not declare localized writes.".to_string());
    }
    lines
}

fn render_merge_target_section_lines(
    current_node_contract: Option<&super::scenario_ir::FlowhubScenarioNodeIr>,
) -> Vec<String> {
    let Some(current_node_contract) = current_node_contract else {
        return vec!["- No active node contract is selected yet.".to_string()];
    };
    if current_node_contract.merge_target.is_empty() {
        return vec!["- Current node does not declare canonical merge targets.".to_string()];
    }
    current_node_contract
        .merge_target
        .iter()
        .map(|target| format!("- {target}"))
        .collect()
}

fn render_success_condition_section_lines(
    current_node_contract: Option<&super::scenario_ir::FlowhubScenarioNodeIr>,
) -> Vec<String> {
    let Some(current_node_contract) = current_node_contract else {
        return vec![
            "- Initialize the localized runtime state before deriving step-level success conditions."
                .to_string(),
        ];
    };

    let mut lines = Vec::new();
    if let Some(checkpoint) = &current_node_contract.checkpoint {
        lines.push(format!("- `{checkpoint}` exists"));
    }
    for write in &current_node_contract.writes {
        lines.push(format!("- `{write}` exists"));
    }
    if lines.is_empty() {
        lines.push("- Current node has no declared checkpoint or writes.".to_string());
    }
    lines
}

fn render_label_list(labels: &[String]) -> String {
    if labels.is_empty() {
        "`none`".to_string()
    } else {
        labels
            .iter()
            .map(|label| format!("`{label}`"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}
