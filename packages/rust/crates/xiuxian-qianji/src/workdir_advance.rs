//! Localized workdir step advancement.
//!
//! This module advances the current Mermaid node for a bounded run root,
//! rewrites runtime state, scaffolds node-owned surfaces, and rolls back when
//! `qianji check --dir` rejects the advanced state.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::QianjiError;
use crate::flowhub::{
    FlowhubScenarioIr, compile_flowhub_scenario_ir, parse_flowhub_graph_annotations,
    resolve_flowhub_graph_name,
};
use crate::flowhub::{MermaidFlowchart, parse_mermaid_flowchart};
use crate::markdown::{MarkdownShowSection, render_show_surface};

use super::check_workdir;
use super::load::load_workdir_manifest;
use super::render_workdir_check_markdown;
use super::{
    WorkdirAllowedNextIssue, WorkdirCurrentNodeIssue, WorkdirRuntimeNode, WorkdirRuntimeState,
    expected_next_labels, load_workdir_runtime_state, resolve_runtime_node,
};

/// Summary of one localized step advance over a bounded workdir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkdirAdvance {
    /// Stable plan name from the root manifest.
    pub plan_name: String,
    /// Advanced bounded workdir root.
    pub workdir: PathBuf,
    /// Previous current-node label.
    pub previous_node: String,
    /// New current-node label.
    pub current_node: String,
    /// Allowed next labels after the advance.
    pub allowed_next: Vec<String>,
    /// Current-step checkpoint/write paths scaffolded for the new node.
    pub current_step_surface: Vec<String>,
    /// Trace log path updated by the advance.
    pub trace_path: PathBuf,
}

#[derive(Debug, Serialize)]
struct WorkdirTraceRecord<'a> {
    event: &'static str,
    from: &'a str,
    to: &'a str,
    allowed_next: &'a [String],
}

/// Advance one localized run root to one allowed next Mermaid node.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the localized run root is missing
/// runtime state, the target node is undeclared or not currently allowed, the
/// state files cannot be rewritten, or the advanced workdir fails validation.
pub fn advance_workdir_step(
    workdir: impl AsRef<Path>,
    target_node_ref: &str,
) -> Result<WorkdirAdvance, QianjiError> {
    let workdir = workdir.as_ref();
    let plan = plan_workdir_advance(workdir, target_node_ref)?;
    let snapshot = read_runtime_state_snapshot(workdir)?;

    write_runtime_state(
        &snapshot.current_node_path,
        &snapshot.allowed_next_path,
        &snapshot.trace_path,
        plan.current_node.label.as_str(),
        &plan.target_node,
        plan.next_allowed.as_slice(),
        snapshot.original_trace.as_str(),
    )?;
    let created_step_surface =
        scaffold_current_step_surface(workdir, plan.current_step_surface.as_slice())?;
    validate_advanced_workdir_or_restore(workdir, &snapshot, created_step_surface.as_slice())?;

    Ok(plan.into_advance(workdir, snapshot.trace_path))
}

struct WorkdirAdvancePlan {
    plan_name: String,
    current_node: WorkdirRuntimeNode,
    target_node: WorkdirRuntimeNode,
    next_allowed: Vec<String>,
    current_step_surface: Vec<String>,
}

impl WorkdirAdvancePlan {
    fn into_advance(self, workdir: &Path, trace_path: PathBuf) -> WorkdirAdvance {
        WorkdirAdvance {
            plan_name: self.plan_name,
            workdir: workdir.to_path_buf(),
            previous_node: self.current_node.label,
            current_node: self.target_node.label,
            allowed_next: self.next_allowed,
            current_step_surface: self.current_step_surface,
            trace_path,
        }
    }
}

struct RuntimeStateSnapshot {
    current_node_path: PathBuf,
    allowed_next_path: PathBuf,
    trace_path: PathBuf,
    original_current_node: String,
    original_allowed_next: String,
    original_trace: String,
}

fn plan_workdir_advance(
    workdir: &Path,
    target_node_ref: &str,
) -> Result<WorkdirAdvancePlan, QianjiError> {
    let manifest = load_workdir_manifest(workdir.join("qianji.toml"))?;
    let contract = load_runtime_contract(&workdir.join("flowchart.mmd"))?;
    let runtime_state = load_workdir_runtime_state(workdir, &contract.flowchart)?;
    let current_node = current_runtime_node(workdir, &runtime_state)?;
    let allowed_next = current_allowed_next(workdir, &runtime_state)?;
    let target_node = resolve_runtime_node(&contract.flowchart, target_node_ref).ok_or_else(|| {
        QianjiError::Topology(format!(
            "localized run root `{}` does not resolve target node `{target_node_ref}` inside `flowchart.mmd`",
            workdir.display()
        ))
    })?;

    if !allowed_next
        .iter()
        .any(|label| label == target_node.label.as_str())
    {
        return Err(QianjiError::Topology(format!(
            "localized run root `{}` cannot advance to `{}` because it is not present in `state/allowed_next.json`: {}",
            workdir.display(),
            target_node.label,
            render_label_list(allowed_next.as_slice())
        )));
    }

    let mut next_allowed = expected_next_labels(&contract.flowchart, target_node.id.as_str());
    next_allowed.sort();
    next_allowed.dedup();
    let current_step_surface =
        target_node_owned_paths(&contract.scenario_ir, target_node.label.as_str());

    Ok(WorkdirAdvancePlan {
        plan_name: manifest.plan.name,
        current_node,
        target_node,
        next_allowed,
        current_step_surface,
    })
}

fn read_runtime_state_snapshot(workdir: &Path) -> Result<RuntimeStateSnapshot, QianjiError> {
    let current_node_path = workdir.join("state/current_node.toml");
    let allowed_next_path = workdir.join("state/allowed_next.json");
    let trace_path = workdir.join("state/trace.jsonl");
    let original_current_node =
        read_required_runtime_file(&current_node_path, "localized current-node state")?;
    let original_allowed_next =
        read_required_runtime_file(&allowed_next_path, "localized allowed-next state")?;
    let original_trace = read_required_runtime_file(&trace_path, "localized trace state")?;

    Ok(RuntimeStateSnapshot {
        current_node_path,
        allowed_next_path,
        trace_path,
        original_current_node,
        original_allowed_next,
        original_trace,
    })
}

fn validate_advanced_workdir_or_restore(
    workdir: &Path,
    snapshot: &RuntimeStateSnapshot,
    created_step_surface: &[PathBuf],
) -> Result<(), QianjiError> {
    let report = check_workdir(workdir)?;
    if !report.is_valid() {
        restore_runtime_state(
            &snapshot.current_node_path,
            snapshot.original_current_node.as_str(),
            &snapshot.allowed_next_path,
            snapshot.original_allowed_next.as_str(),
            &snapshot.trace_path,
            snapshot.original_trace.as_str(),
        )?;
        restore_current_step_surface(workdir, created_step_surface)?;
        return Err(QianjiError::Topology(format!(
            "Advanced work surface `{}` failed validation:\n{}",
            workdir.display(),
            render_workdir_check_markdown(&report)
        )));
    }
    Ok(())
}

/// Render one localized step advance into a compact markdown summary.
#[must_use]
pub fn render_workdir_advance(advance: &WorkdirAdvance) -> String {
    render_show_surface(
        "Advanced Workdir Step",
        &[
            format!("Plan: {}", advance.plan_name),
            format!("Location: {}", advance.workdir.display()),
        ],
        &[
            MarkdownShowSection {
                title: "Transition".into(),
                lines: vec![
                    format!("Previous node: {}", advance.previous_node),
                    format!("Current node: {}", advance.current_node),
                ],
            },
            MarkdownShowSection {
                title: "Allowed Next".into(),
                lines: vec![format!(
                    "Nodes: {}",
                    render_label_list(advance.allowed_next.as_slice())
                )],
            },
            MarkdownShowSection {
                title: "Current Step Surface".into(),
                lines: if advance.current_step_surface.is_empty() {
                    vec!["- Current node does not declare localized checkpoint or writes.".into()]
                } else {
                    advance
                        .current_step_surface
                        .iter()
                        .map(|path| format!("- {path}"))
                        .collect()
                },
            },
            MarkdownShowSection {
                title: "Trace".into(),
                lines: vec![format!("Path: {}", advance.trace_path.display())],
            },
        ],
    )
}

struct RuntimeContract {
    flowchart: MermaidFlowchart,
    scenario_ir: FlowhubScenarioIr,
}

fn load_runtime_contract(flowchart_path: &Path) -> Result<RuntimeContract, QianjiError> {
    let flowchart_source = fs::read_to_string(flowchart_path).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to read localized workdir flowchart `{}`: {error}",
            flowchart_path.display()
        ))
    })?;
    let annotations = parse_flowhub_graph_annotations(&flowchart_source).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to parse localized workdir Mermaid annotations from `{}`: {error}",
            flowchart_path.display()
        ))
    })?;
    let fallback_graph_name = flowchart_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| {
            QianjiError::Topology(format!(
                "Failed to derive Mermaid graph name from `{}`",
                flowchart_path.display()
            ))
        })?;
    let graph_name = resolve_flowhub_graph_name(annotations.as_ref(), None, fallback_graph_name);

    let flowchart = parse_mermaid_flowchart(
        &flowchart_source,
        graph_name.as_str(),
        &Vec::<String>::new(),
    )
    .map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to parse localized workdir Mermaid graph `{}`: {error}",
            flowchart_path.display()
        ))
    })?;
    let scenario_ir = compile_flowhub_scenario_ir(
        flowchart_path,
        graph_name.as_str(),
        &flowchart,
        annotations.as_ref(),
        None,
    )?
    .ok_or_else(|| {
        QianjiError::Topology(format!(
            "Failed to compile localized workdir Mermaid graph `{}` into a scenario contract",
            flowchart_path.display()
        ))
    })?;

    Ok(RuntimeContract {
        flowchart,
        scenario_ir,
    })
}

fn current_runtime_node(
    workdir: &Path,
    runtime_state: &WorkdirRuntimeState,
) -> Result<WorkdirRuntimeNode, QianjiError> {
    if let Some(issue) = runtime_state.current_node.issue.as_ref() {
        let detail = match issue {
            WorkdirCurrentNodeIssue::MissingField => {
                "`state/current_node.toml` must declare `current_node = \"<node>\"`".to_string()
            }
            WorkdirCurrentNodeIssue::UnknownNode(node_ref) => format!(
                "`state/current_node.toml` selects `{node_ref}`, but that node is not present in `flowchart.mmd`"
            ),
        };
        return Err(QianjiError::Topology(format!(
            "localized run root `{}` cannot advance because {detail}",
            workdir.display()
        )));
    }

    runtime_state.current_node.resolved.clone().ok_or_else(|| {
        QianjiError::Topology(format!(
            "localized run root `{}` cannot advance because `state/current_node.toml` is not set",
            workdir.display()
        ))
    })
}

fn current_allowed_next(
    workdir: &Path,
    runtime_state: &WorkdirRuntimeState,
) -> Result<Vec<String>, QianjiError> {
    if runtime_state.allowed_next.raw_refs.is_none() {
        return Err(QianjiError::Topology(format!(
            "localized run root `{}` cannot advance because `state/allowed_next.json` is not set",
            workdir.display()
        )));
    }

    if let Some(issue) = runtime_state.allowed_next.issue.as_ref() {
        let detail = match issue {
            WorkdirAllowedNextIssue::InvalidJson(error) => format!(
                "`state/allowed_next.json` must be a JSON string array of Mermaid node ids or labels: {error}"
            ),
            WorkdirAllowedNextIssue::UnknownNode(node_ref) => format!(
                "`state/allowed_next.json` references `{node_ref}`, but that node is not present in `flowchart.mmd`"
            ),
        };
        return Err(QianjiError::Topology(format!(
            "localized run root `{}` cannot advance because {detail}",
            workdir.display()
        )));
    }

    if runtime_state.allowed_next.expected_labels != runtime_state.allowed_next.resolved_labels {
        return Err(QianjiError::Topology(format!(
            "localized run root `{}` cannot advance because `state/allowed_next.json` drifts from the current node adjacency: declared {}, expected {}",
            workdir.display(),
            render_label_list(runtime_state.allowed_next.resolved_labels.as_slice()),
            render_label_list(runtime_state.allowed_next.expected_labels.as_slice())
        )));
    }

    Ok(runtime_state.allowed_next.resolved_labels.clone())
}

fn read_required_runtime_file(path: &Path, surface_name: &str) -> Result<String, QianjiError> {
    fs::read_to_string(path).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to read {surface_name} `{}`: {error}",
            path.display()
        ))
    })
}

fn write_runtime_state(
    current_node_path: &Path,
    allowed_next_path: &Path,
    trace_path: &Path,
    previous_node_label: &str,
    target_node: &WorkdirRuntimeNode,
    next_allowed: &[String],
    original_trace: &str,
) -> Result<(), QianjiError> {
    let current_node_toml = format!("current_node = {:?}\n", target_node.label);
    let allowed_next_json = serde_json::to_string_pretty(next_allowed)
        .map(|json| format!("{json}\n"))
        .map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to serialize localized allowed-next state: {error}"
            ))
        })?;
    let trace_record = WorkdirTraceRecord {
        event: "step_advance",
        from: previous_node_label,
        to: target_node.label.as_str(),
        allowed_next: next_allowed,
    };
    let trace_line = serde_json::to_string(&trace_record).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to serialize localized trace record for `{}`: {error}",
            trace_path.display()
        ))
    })?;
    let trace_jsonl = append_jsonl_record(original_trace, trace_line.as_str());

    fs::write(current_node_path, current_node_toml).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to write localized current-node state `{}`: {error}",
            current_node_path.display()
        ))
    })?;
    fs::write(allowed_next_path, allowed_next_json).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to write localized allowed-next state `{}`: {error}",
            allowed_next_path.display()
        ))
    })?;
    fs::write(trace_path, trace_jsonl).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to write localized trace state `{}`: {error}",
            trace_path.display()
        ))
    })?;

    Ok(())
}

fn target_node_owned_paths(
    scenario_ir: &FlowhubScenarioIr,
    target_node_label: &str,
) -> Vec<String> {
    scenario_ir
        .node_contract(target_node_label)
        .map_or_else(Vec::new, |node| {
            node.checkpoint
                .iter()
                .cloned()
                .chain(node.writes.iter().cloned())
                .collect()
        })
}

fn scaffold_current_step_surface(
    workdir: &Path,
    current_step_surface: &[String],
) -> Result<Vec<PathBuf>, QianjiError> {
    current_step_surface
        .iter()
        .try_fold(Vec::new(), |mut created, relative_path| {
            let target_path = workdir.join(relative_path);
            if !target_path.exists() {
                write_scaffold_file(&target_path, scaffold_file_content(relative_path))?;
                created.push(target_path);
            }
            Ok(created)
        })
}

fn restore_current_step_surface(
    workdir: &Path,
    created_step_surface: &[PathBuf],
) -> Result<(), QianjiError> {
    for created_path in created_step_surface {
        if !created_path.starts_with(workdir) {
            return Err(QianjiError::Topology(format!(
                "Refusing to restore scaffolded path `{}` outside localized run root `{}`",
                created_path.display(),
                workdir.display()
            )));
        }

        if created_path.is_file() {
            fs::remove_file(created_path).map_err(|error| {
                QianjiError::Topology(format!(
                    "Failed to remove scaffolded current-step file `{}`: {error}",
                    created_path.display()
                ))
            })?;
        }
    }

    Ok(())
}

fn restore_runtime_state(
    current_node_path: &Path,
    original_current_node: &str,
    allowed_next_path: &Path,
    original_allowed_next: &str,
    trace_path: &Path,
    original_trace: &str,
) -> Result<(), QianjiError> {
    fs::write(current_node_path, original_current_node).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to restore localized current-node state `{}`: {error}",
            current_node_path.display()
        ))
    })?;
    fs::write(allowed_next_path, original_allowed_next).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to restore localized allowed-next state `{}`: {error}",
            allowed_next_path.display()
        ))
    })?;
    fs::write(trace_path, original_trace).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to restore localized trace state `{}`: {error}",
            trace_path.display()
        ))
    })?;

    Ok(())
}

fn append_jsonl_record(existing: &str, record: &str) -> String {
    let mut jsonl = existing.to_string();
    if !jsonl.is_empty() && !jsonl.ends_with('\n') {
        jsonl.push('\n');
    }
    jsonl.push_str(record);
    jsonl.push('\n');
    jsonl
}

fn scaffold_file_content(path: &str) -> &'static str {
    if has_extension(path, "json") {
        "{}\n"
    } else {
        ""
    }
}

fn has_extension(path: &str, extension: &str) -> bool {
    Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
}

fn write_scaffold_file(path: &Path, content: &str) -> Result<(), QianjiError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to create localized parent directory `{}`: {error}",
                parent.display()
            ))
        })?;
    }

    fs::write(path, content).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to write localized current-step scaffold `{}`: {error}",
            path.display()
        ))
    })
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
