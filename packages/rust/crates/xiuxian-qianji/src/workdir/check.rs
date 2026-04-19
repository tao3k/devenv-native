use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use globset::Glob;
use regex::Regex;
use walkdir::WalkDir;

use crate::error::QianjiError;
use crate::flowhub::mermaid::parse_mermaid_flowchart;
use crate::flowhub::scenario_ir::{
    compile_flowhub_scenario_ir, parse_flowhub_graph_annotations, resolve_flowhub_graph_name,
};
use crate::markdown::{
    MarkdownDiagnostic, render_follow_up_query_section, render_validation_failed,
    render_validation_pass,
};

use super::load::load_workdir_manifest;
use super::query::build_workdir_check_follow_up_query;
use super::{
    WorkdirAllowedNextIssue, WorkdirCurrentNodeIssue, WorkdirRuntimeState,
    load_workdir_runtime_state,
};

/// One bounded markdown retrieval surface supported by the compact workdir.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorkdirMarkdownSurface {
    /// The `blueprint/` markdown surface.
    Blueprint,
    /// The `plan/` markdown surface.
    Plan,
}

impl WorkdirMarkdownSurface {
    /// Return the stable SQL-visible surface name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Blueprint => "blueprint",
            Self::Plan => "plan",
        }
    }

    fn from_top_level_name(surface: &str) -> Option<Self> {
        match surface {
            "blueprint" => Some(Self::Blueprint),
            "plan" => Some(Self::Plan),
            _ => None,
        }
    }
}

/// One user-facing validation diagnostic for a bounded work surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkdirDiagnostic {
    /// Short diagnostic title.
    pub title: String,
    /// On-disk location of the failing surface.
    pub location: PathBuf,
    /// Concrete failing condition.
    pub problem: String,
    /// Why the issue blocks continued bounded work.
    pub why_it_blocks: String,
    /// Concrete next action for repairing the surface.
    pub fix: String,
    /// Bounded markdown surfaces that should be queried during repair follow-up.
    pub follow_up_surfaces: Vec<WorkdirMarkdownSurface>,
}

/// Structural validation result for one bounded work surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkdirCheckReport {
    /// Stable plan name from the root manifest.
    pub plan_name: String,
    /// Checked bounded workdir root.
    pub workdir: PathBuf,
    /// Collected blocking diagnostics.
    pub diagnostics: Vec<WorkdirDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkdirStepAwareContext {
    required_paths: Vec<String>,
    allowed_next_validation: Option<WorkdirAllowedNextValidation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkdirAllowedNextValidation {
    current_node_label: String,
    expected_next_labels: Vec<String>,
    actual_next_labels: Vec<String>,
}

impl WorkdirCheckReport {
    /// Returns `true` when no blocking diagnostics were emitted.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

/// Validate the bounded work-surface contract on disk.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the root manifest cannot be loaded,
/// the filesystem cannot be inspected, or the flowchart companion cannot be
/// read.
pub fn check_workdir(workdir: impl AsRef<Path>) -> Result<WorkdirCheckReport, QianjiError> {
    let workdir = workdir.as_ref();
    let manifest = load_workdir_manifest(workdir.join("qianji.toml"))?;
    let mut diagnostics = Vec::new();
    let flowchart_path = workdir.join("flowchart.mmd");
    let flowchart = load_optional_flowchart(&flowchart_path)?;
    let step_aware = derive_step_aware_context(
        workdir,
        flowchart.as_deref(),
        &flowchart_path,
        &manifest.check.require,
        &mut diagnostics,
    )?;
    let required_paths = step_aware_required_paths(&manifest.check.require, step_aware.as_ref());
    validate_required_paths(workdir, &required_paths, &mut diagnostics)?;
    validate_flowchart_alignment(
        workdir,
        &flowchart_path,
        flowchart.as_deref(),
        &manifest.check.flowchart,
        step_aware.as_ref(),
        &mut diagnostics,
    )?;

    Ok(WorkdirCheckReport {
        plan_name: manifest.plan.name,
        workdir: workdir.to_path_buf(),
        diagnostics,
    })
}

fn load_optional_flowchart(flowchart_path: &Path) -> Result<Option<String>, QianjiError> {
    if !flowchart_path.is_file() {
        return Ok(None);
    }

    fs::read_to_string(flowchart_path)
        .map(Some)
        .map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to read bounded work-surface flowchart `{}`: {error}",
                flowchart_path.display()
            ))
        })
}

fn step_aware_required_paths(
    manifest_require: &[String],
    step_aware: Option<&WorkdirStepAwareContext>,
) -> Vec<String> {
    step_aware.map_or_else(
        || manifest_require.to_vec(),
        |context| context.required_paths.clone(),
    )
}

fn validate_required_paths(
    workdir: &Path,
    required_paths: &[String],
    diagnostics: &mut Vec<WorkdirDiagnostic>,
) -> Result<(), QianjiError> {
    for requirement in required_paths {
        if is_glob_pattern(requirement) {
            emit_missing_glob_diagnostic(workdir, requirement, diagnostics)?;
        } else if !workdir.join(requirement).exists() {
            diagnostics.push(missing_required_path_diagnostic(workdir, requirement));
        }
    }

    Ok(())
}

fn emit_missing_glob_diagnostic(
    workdir: &Path,
    requirement: &str,
    diagnostics: &mut Vec<WorkdirDiagnostic>,
) -> Result<(), QianjiError> {
    if count_glob_matches(workdir, requirement)? == 0 {
        diagnostics.push(WorkdirDiagnostic {
            title: "Missing required glob matches".to_string(),
            location: workdir.to_path_buf(),
            problem: format!(
                "bounded work-surface contract requires at least one match for `{requirement}`, but none were found"
            ),
            why_it_blocks: "the bounded surface is structurally incomplete".to_string(),
            fix: format!("create at least one file matching `{requirement}` or relax `check.require`"),
            follow_up_surfaces: follow_up_surfaces_for_requirement(requirement),
        });
    }

    Ok(())
}

fn missing_required_path_diagnostic(workdir: &Path, requirement: &str) -> WorkdirDiagnostic {
    WorkdirDiagnostic {
        title: "Missing required path".to_string(),
        location: workdir.join(requirement),
        problem: format!(
            "bounded work-surface contract requires `{requirement}`, but the path is absent"
        ),
        why_it_blocks: "Codex cannot rely on the declared bounded surface".to_string(),
        fix: format!("create `{requirement}` or relax `check.require`"),
        follow_up_surfaces: follow_up_surfaces_for_requirement(requirement),
    }
}

fn validate_flowchart_alignment(
    workdir: &Path,
    flowchart_path: &Path,
    flowchart: Option<&str>,
    flowchart_entries: &[String],
    step_aware: Option<&WorkdirStepAwareContext>,
    diagnostics: &mut Vec<WorkdirDiagnostic>,
) -> Result<(), QianjiError> {
    let Some(flowchart) = flowchart else {
        diagnostics.push(missing_flowchart_companion_diagnostic(
            flowchart_path,
            flowchart_entries,
        ));
        return Ok(());
    };

    if step_aware.is_none() {
        validate_declared_flowchart_surfaces(
            flowchart,
            flowchart_path,
            flowchart_entries,
            diagnostics,
        )?;
    }
    validate_allowed_next_alignment(workdir, step_aware, diagnostics);

    Ok(())
}

fn validate_declared_flowchart_surfaces(
    flowchart: &str,
    flowchart_path: &Path,
    flowchart_entries: &[String],
    diagnostics: &mut Vec<WorkdirDiagnostic>,
) -> Result<(), QianjiError> {
    for surface in flowchart_entries {
        if !flowchart_contains_surface(flowchart, surface)? {
            diagnostics.push(missing_flowchart_surface_diagnostic(
                flowchart_path,
                surface,
                flowchart_entries,
            ));
        }
    }

    for pair in flowchart_entries.windows(2) {
        let from = &pair[0];
        let to = &pair[1];
        if !flowchart_contains_backbone(flowchart, from, to)? {
            diagnostics.push(missing_flowchart_backbone_diagnostic(
                flowchart_path,
                from,
                to,
                flowchart_entries,
            ));
        }
    }

    Ok(())
}

fn validate_allowed_next_alignment(
    workdir: &Path,
    step_aware: Option<&WorkdirStepAwareContext>,
    diagnostics: &mut Vec<WorkdirDiagnostic>,
) {
    let Some(allowed_next) =
        step_aware.and_then(|context| context.allowed_next_validation.as_ref())
    else {
        return;
    };
    if allowed_next.expected_next_labels == allowed_next.actual_next_labels {
        return;
    }

    diagnostics.push(WorkdirDiagnostic {
        title: "Allowed-next drift".to_string(),
        location: workdir.join("state/allowed_next.json"),
        problem: format!(
            "`state/allowed_next.json` declares {}, but the current node `{}` allows {}",
            render_label_list(allowed_next.actual_next_labels.as_slice()),
            allowed_next.current_node_label,
            render_label_list(allowed_next.expected_next_labels.as_slice()),
        ),
        why_it_blocks:
            "the localized run state no longer matches the declared graph transition boundary"
                .to_string(),
        fix: format!(
            "rewrite `state/allowed_next.json` so it matches the current node `{}` adjacency",
            allowed_next.current_node_label
        ),
        follow_up_surfaces: Vec::new(),
    });
}

fn missing_flowchart_companion_diagnostic(
    flowchart_path: &Path,
    flowchart_entries: &[String],
) -> WorkdirDiagnostic {
    WorkdirDiagnostic {
        title: "Missing flowchart companion".to_string(),
        location: flowchart_path.to_path_buf(),
        problem:
            "`flowchart.mmd` is required for flowchart alignment checks, but the file is absent"
                .to_string(),
        why_it_blocks: "the bounded work surface has no direct graph companion".to_string(),
        fix: "create `flowchart.mmd` at the work-surface root".to_string(),
        follow_up_surfaces: follow_up_surfaces_for_flowchart(flowchart_entries),
    }
}

fn missing_flowchart_surface_diagnostic(
    flowchart_path: &Path,
    surface: &str,
    flowchart_entries: &[String],
) -> WorkdirDiagnostic {
    WorkdirDiagnostic {
        title: "Missing flowchart surface".to_string(),
        location: flowchart_path.to_path_buf(),
        problem: format!(
            "`flowchart.mmd` does not visibly contain the principal surface `{surface}`"
        ),
        why_it_blocks: "the graph companion no longer aligns with the bounded work surface"
            .to_string(),
        fix: format!("add a visible `{surface}` node or label to `flowchart.mmd`"),
        follow_up_surfaces: follow_up_surfaces_for_flowchart(flowchart_entries),
    }
}

fn missing_flowchart_backbone_diagnostic(
    flowchart_path: &Path,
    from: &str,
    to: &str,
    flowchart_entries: &[String],
) -> WorkdirDiagnostic {
    WorkdirDiagnostic {
        title: "Missing flowchart backbone".to_string(),
        location: flowchart_path.to_path_buf(),
        problem: format!("`flowchart.mmd` does not visibly express the backbone `{from} --> {to}`"),
        why_it_blocks: "Codex cannot trust the visible backbone direction of the bounded work"
            .to_string(),
        fix: format!("add a visible `{from} --> {to}` relation to `flowchart.mmd`"),
        follow_up_surfaces: follow_up_surfaces_for_flowchart(flowchart_entries),
    }
}

fn derive_step_aware_context(
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

/// Render a bounded work-surface validation report into markdown diagnostics.
#[must_use]
pub fn render_workdir_check_markdown(report: &WorkdirCheckReport) -> String {
    if report.is_valid() {
        return render_validation_pass(&[
            format!("Plan: {}", report.plan_name),
            format!("Location: {}", report.workdir.display()),
        ]);
    }

    let diagnostics = report
        .diagnostics
        .iter()
        .map(|diagnostic| MarkdownDiagnostic {
            title: diagnostic.title.as_str(),
            location: diagnostic.location.display().to_string().into(),
            problem: diagnostic.problem.as_str(),
            why_it_blocks: diagnostic.why_it_blocks.as_str(),
            fix: diagnostic.fix.as_str(),
        })
        .collect::<Vec<_>>();

    let mut rendered = render_validation_failed(&[], &diagnostics);
    if let Some(follow_up_query) = build_workdir_check_follow_up_query(report) {
        let surface_names = follow_up_query
            .surfaces
            .iter()
            .map(|surface| surface.as_str())
            .collect::<Vec<_>>()
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        rendered.push_str("\n\n");
        rendered.push_str(&render_follow_up_query_section(
            &surface_names,
            &follow_up_query.query_text,
        ));
    }

    rendered
}

fn count_glob_matches(workdir: &Path, pattern: &str) -> Result<usize, QianjiError> {
    let matcher = Glob::new(pattern)
        .map_err(|error| {
            QianjiError::Topology(format!(
                "invalid `check.require` glob pattern `{pattern}`: {error}"
            ))
        })?
        .compile_matcher();

    let mut match_count = 0_usize;
    for entry in WalkDir::new(workdir) {
        let entry = entry.map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to walk bounded work surface `{}`: {error}",
                workdir.display()
            ))
        })?;
        if entry.path() == workdir {
            continue;
        }
        let relative = entry.path().strip_prefix(workdir).map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to relativize bounded work-surface path `{}` against `{}`: {error}",
                entry.path().display(),
                workdir.display()
            ))
        })?;
        let normalized = relative.to_string_lossy().replace('\\', "/");
        if matcher.is_match(normalized.as_str()) {
            match_count += 1;
        }
    }

    Ok(match_count)
}

fn flowchart_contains_surface(flowchart: &str, surface: &str) -> Result<bool, QianjiError> {
    let regex = surface_regex(surface)?;
    Ok(regex.is_match(flowchart))
}

fn flowchart_contains_backbone(flowchart: &str, from: &str, to: &str) -> Result<bool, QianjiError> {
    let from_regex = surface_regex(from)?;
    let to_regex = surface_regex(to)?;

    for line in flowchart.lines().filter(|line| line.contains("-->")) {
        let Some(arrow_index) = line.find("-->") else {
            continue;
        };
        let from_match = from_regex
            .find_iter(line)
            .find(|capture| capture.start() < arrow_index);
        let to_match = to_regex
            .find_iter(line)
            .find(|capture| capture.start() > arrow_index);
        if from_match.is_some() && to_match.is_some() {
            return Ok(true);
        }
    }

    Ok(false)
}

fn surface_regex(surface: &str) -> Result<Regex, QianjiError> {
    Regex::new(&format!(
        r"(^|[^A-Za-z0-9_-]){}([^A-Za-z0-9_-]|$)",
        regex::escape(surface)
    ))
    .map_err(|error| {
        QianjiError::Topology(format!(
            "failed to build flowchart surface matcher for `{surface}`: {error}"
        ))
    })
}

fn is_glob_pattern(value: &str) -> bool {
    value
        .chars()
        .any(|character| matches!(character, '*' | '?' | '[' | ']'))
}

fn follow_up_surfaces_for_requirement(requirement: &str) -> Vec<WorkdirMarkdownSurface> {
    let mut surfaces = Vec::new();
    if requirement.starts_with("blueprint") {
        surfaces.push(WorkdirMarkdownSurface::Blueprint);
    }
    if requirement.starts_with("plan") {
        surfaces.push(WorkdirMarkdownSurface::Plan);
    }
    surfaces
}

fn follow_up_surfaces_for_flowchart(entries: &[String]) -> Vec<WorkdirMarkdownSurface> {
    let mut surfaces = entries
        .iter()
        .filter_map(|entry| WorkdirMarkdownSurface::from_top_level_name(entry))
        .collect::<Vec<_>>();
    if surfaces.is_empty() {
        surfaces.push(WorkdirMarkdownSurface::Blueprint);
        surfaces.push(WorkdirMarkdownSurface::Plan);
    }
    surfaces
}

fn render_label_list(values: &[String]) -> String {
    if values.is_empty() {
        return "`none`".to_string();
    }

    values
        .iter()
        .map(|value| format!("`{value}`"))
        .collect::<Vec<_>>()
        .join(", ")
}
