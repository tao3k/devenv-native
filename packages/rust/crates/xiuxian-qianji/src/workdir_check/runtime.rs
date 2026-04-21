use std::collections::BTreeSet;
use std::path::Path;

use crate::error::QianjiError;
use crate::flowhub::parse_mermaid_flowchart;
use crate::flowhub::{
    compile_flowhub_scenario_ir, parse_flowhub_graph_annotations, resolve_flowhub_graph_name,
};
use crate::workdir::{
    WorkdirAllowedNextIssue, WorkdirCurrentNodeIssue, WorkdirRuntimeState,
    load_workdir_runtime_state,
};

use super::api::WorkdirDiagnostic;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct WorkdirStepAwareContext {
    pub(super) required_paths: Vec<String>,
    pub(super) allowed_next_validation: Option<WorkdirAllowedNextValidation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct WorkdirAllowedNextValidation {
    pub(super) current_node_label: String,
    pub(super) expected_next_labels: Vec<String>,
    pub(super) actual_next_labels: Vec<String>,
}

pub(super) fn step_aware_required_paths(
    manifest_require: &[String],
    step_aware: Option<&WorkdirStepAwareContext>,
) -> Vec<String> {
    step_aware.map_or_else(
        || manifest_require.to_vec(),
        |context| context.required_paths.clone(),
    )
}

pub(super) fn derive_step_aware_context(
    workdir: &Path,
    flowchart: Option<&str>,
    flowchart_path: &Path,
    manifest_require: &[String],
    diagnostics: &mut Vec<WorkdirDiagnostic>,
) -> Result<Option<WorkdirStepAwareContext>, QianjiError> {
    let Some(flowchart) = flowchart else {
        return Ok(None);
    };
    let annotations = parse_flowhub_graph_annotations(flowchart).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to parse bounded work-surface Mermaid annotations from `{}`: {error}",
            flowchart_path.display()
        ))
    })?;
    let Some(annotations) = annotations else {
        return Ok(None);
    };

    let fallback_graph_name = flowchart_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| {
            QianjiError::Topology(format!(
                "Failed to derive Mermaid graph name from `{}`",
                flowchart_path.display()
            ))
        })?;
    let graph_name = resolve_flowhub_graph_name(Some(&annotations), None, fallback_graph_name);
    let registered_modules = Vec::<String>::new();
    let parsed_flowchart =
        parse_mermaid_flowchart(flowchart, graph_name.as_str(), &registered_modules).map_err(
            |error| {
                QianjiError::Topology(format!(
                    "Failed to parse bounded work-surface Mermaid graph `{}`: {error}",
                    flowchart_path.display()
                ))
            },
        )?;
    let scenario_ir = compile_flowhub_scenario_ir(
        flowchart_path,
        graph_name.as_str(),
        &parsed_flowchart,
        Some(&annotations),
        None,
    )?
    .ok_or_else(|| {
        QianjiError::Topology(format!(
            "Failed to compile bounded work-surface Mermaid graph `{}` into a scenario contract",
            flowchart_path.display()
        ))
    })?;

    let node_owned_paths = scenario_ir
        .nodes
        .iter()
        .flat_map(|node| {
            node.checkpoint
                .iter()
                .cloned()
                .chain(node.writes.iter().cloned())
        })
        .collect::<BTreeSet<_>>();
    let mut required_paths = manifest_require
        .iter()
        .filter(|path| !node_owned_paths.contains(path.as_str()))
        .cloned()
        .collect::<Vec<_>>();

    let mut allowed_next_validation = None;
    let runtime_state = load_workdir_runtime_state(workdir, &parsed_flowchart)?;
    emit_runtime_state_diagnostics(workdir, &runtime_state, diagnostics);
    if let Some(current_node) = runtime_state.current_node.resolved.as_ref() {
        if let Some(node_contract) = scenario_ir.node_contract(current_node.label.as_str()) {
            if let Some(checkpoint) = &node_contract.checkpoint {
                push_unique_string(&mut required_paths, checkpoint);
            }
            for write in &node_contract.writes {
                push_unique_string(&mut required_paths, write);
            }
        }
        if runtime_state.allowed_next.issue.is_none() {
            allowed_next_validation = runtime_state.allowed_next.raw_refs.as_ref().map(|_| {
                WorkdirAllowedNextValidation {
                    current_node_label: current_node.label.clone(),
                    expected_next_labels: runtime_state.allowed_next.expected_labels.clone(),
                    actual_next_labels: runtime_state.allowed_next.resolved_labels.clone(),
                }
            });
        }
    }

    Ok(Some(WorkdirStepAwareContext {
        required_paths,
        allowed_next_validation,
    }))
}

fn emit_runtime_state_diagnostics(
    workdir: &Path,
    runtime_state: &WorkdirRuntimeState,
    diagnostics: &mut Vec<WorkdirDiagnostic>,
) {
    let current_node_path = workdir.join("state/current_node.toml");
    match (
        runtime_state.current_node.raw_ref.as_ref(),
        runtime_state.current_node.issue.as_ref(),
    ) {
        (_, Some(WorkdirCurrentNodeIssue::MissingField)) => diagnostics.push(WorkdirDiagnostic {
            title: "Invalid current node state".to_string(),
            location: current_node_path,
            problem: "`state/current_node.toml` must declare `current_node = \"<node>\"`"
                .to_string(),
            why_it_blocks: "the localized run state cannot identify which graph step is active"
                .to_string(),
            fix: "write `current_node = \"<mermaid node id or label>\"` to `state/current_node.toml`"
                .to_string(),
            follow_up_surfaces: Vec::new(),
        }),
        (Some(raw_ref), Some(WorkdirCurrentNodeIssue::UnknownNode(_))) => diagnostics.push(
            WorkdirDiagnostic {
                title: "Invalid current node state".to_string(),
                location: current_node_path,
                problem: format!(
                    "`state/current_node.toml` selects `{raw_ref}`, but that node is not present in `flowchart.mmd`"
                ),
                why_it_blocks:
                    "the localized run state points at a step outside the declared graph"
                        .to_string(),
                fix: "rewrite `state/current_node.toml` so `current_node` matches a Mermaid node id or label"
                    .to_string(),
                follow_up_surfaces: Vec::new(),
            },
        ),
        _ => {}
    }

    let allowed_next_path = workdir.join("state/allowed_next.json");
    match runtime_state.allowed_next.issue.as_ref() {
        Some(WorkdirAllowedNextIssue::InvalidJson(error)) => {
            diagnostics.push(WorkdirDiagnostic {
                title: "Invalid allowed-next state".to_string(),
                location: allowed_next_path,
                problem: format!(
                    "`state/allowed_next.json` must be a JSON string array of Mermaid node ids or labels: {error}"
                ),
                why_it_blocks:
                    "the localized run state cannot prove which transitions remain legal"
                        .to_string(),
                fix: "rewrite `state/allowed_next.json` as a JSON array of Mermaid node ids or labels"
                    .to_string(),
                follow_up_surfaces: Vec::new(),
            });
        }
        Some(WorkdirAllowedNextIssue::UnknownNode(next_ref)) => {
            diagnostics.push(WorkdirDiagnostic {
                title: "Invalid allowed-next state".to_string(),
                location: allowed_next_path,
                problem: format!(
                    "`state/allowed_next.json` references `{next_ref}`, but that node is not present in `flowchart.mmd`"
                ),
                why_it_blocks:
                    "the localized run state points at an undeclared next-step boundary"
                        .to_string(),
                fix: "rewrite `state/allowed_next.json` so every entry matches a Mermaid node id or label"
                    .to_string(),
                follow_up_surfaces: Vec::new(),
            });
        }
        None => {}
    }
}

fn push_unique_string(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|entry| entry == value) {
        values.push(value.to_string());
    }
}
