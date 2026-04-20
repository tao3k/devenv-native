use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::contracts::{WorkdirManifest, WorkdirPlan};
use crate::error::QianjiError;
use crate::flowhub::MermaidFlowchart;
use crate::markdown::{MarkdownShowSection, render_show_surface};
use crate::workdir::{check_workdir, render_workdir_check_markdown};

use super::materialize_safety::ensure_output_dir_is_safe;
use crate::flowhub::FlowhubScenarioIr;
use crate::flowhub::anchor::{resolve_anchor_manifest_path, resolve_anchored_graph};
use crate::flowhub::graph_show::{display_graph_path, load_flowhub_graph_runtime_contract};
use crate::workdir::{WorkdirRuntimeNode, resolve_runtime_node};

/// Summary of one localized run root materialized from an anchor-owned Mermaid
/// scenario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchoredMaterializedWorkdir {
    /// Stable localized plan/scenario id.
    pub plan_name: String,
    /// Owning module anchor manifest.
    pub anchor_manifest_path: PathBuf,
    /// Resolved Mermaid graph path.
    pub graph_path: PathBuf,
    /// Materialized run-root location.
    pub output_dir: PathBuf,
    /// Ordered visible top-level surfaces.
    pub visible_surfaces: Vec<String>,
    /// Bootstrap current-node label.
    pub current_node: String,
    /// Bootstrap allowed-next labels.
    pub allowed_next: Vec<String>,
    /// Current-step checkpoint/write paths scaffolded for the selected node.
    pub current_step_surface: Vec<String>,
}

/// Materialize one anchor-resolved Mermaid scenario into a minimal localized
/// run root that immediately participates in step-aware `qianji check --dir`.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the anchor/scenario cannot be
/// resolved, the graph does not expose a localized workdir contract, the
/// output directory is unsafe, or the generated run root fails bounded-work
/// validation.
pub fn materialize_flowhub_anchored_scenario(
    anchor: impl AsRef<Path>,
    scenario_ref: &str,
    output_dir: impl AsRef<Path>,
) -> Result<AnchoredMaterializedWorkdir, QianjiError> {
    materialize_flowhub_anchored_scenario_at_node(anchor, scenario_ref, output_dir, None)
}

/// Materialize one anchor-resolved Mermaid scenario into a localized run root
/// scaffolded for either the default start node or one selected current node.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the anchor/scenario cannot be
/// resolved, the graph does not expose a localized workdir contract, the
/// requested current node is not declared by the Mermaid graph, the output
/// directory is unsafe, or the generated run root fails bounded-work
/// validation.
pub fn materialize_flowhub_anchored_scenario_at_node(
    anchor: impl AsRef<Path>,
    scenario_ref: &str,
    output_dir: impl AsRef<Path>,
    current_node_ref: Option<&str>,
) -> Result<AnchoredMaterializedWorkdir, QianjiError> {
    let anchor_manifest_path = resolve_anchor_manifest_path(anchor.as_ref());
    let graph_path = resolve_anchored_graph(&anchor_manifest_path, scenario_ref)?;
    let output_dir = output_dir.as_ref();
    let graph_source = fs::read_to_string(&graph_path).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to read anchored Mermaid graph `{}`: {error}",
            graph_path.display()
        ))
    })?;
    let (flowchart, scenario_ir) = load_flowhub_graph_runtime_contract(&graph_path)?;
    let scenario_ir = scenario_ir.ok_or_else(|| {
        QianjiError::Topology(format!(
            "Flowhub graph `{}` does not compile into a localized scenario contract",
            graph_path.display()
        ))
    })?;
    let workdir_contract = scenario_ir.workdir.as_ref().ok_or_else(|| {
        QianjiError::Topology(format!(
            "Flowhub graph `{}` does not declare a localized workdir contract",
            graph_path.display()
        ))
    })?;

    ensure_output_dir_is_safe(output_dir)?;
    fs::create_dir_all(output_dir).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to create anchored materialize target `{}`: {error}",
            output_dir.display()
        ))
    })?;

    let visible_surfaces = derive_visible_surfaces(&workdir_contract.check.require);
    materialize_surface_dirs(output_dir, &visible_surfaces)?;

    let plan_name = scenario_ir
        .scenario_id
        .clone()
        .unwrap_or_else(|| scenario_ir.merimind_graph_name.clone());
    let manifest = WorkdirManifest {
        version: 1,
        plan: WorkdirPlan {
            name: plan_name.clone(),
            surface: visible_surfaces.clone(),
        },
        check: workdir_contract.check.clone(),
    };
    let manifest_toml = toml::to_string_pretty(&manifest).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to serialize anchored workdir manifest for `{}`: {error}",
            graph_path.display()
        ))
    })?;
    write_file(&output_dir.join("qianji.toml"), manifest_toml.as_str())?;
    write_file(&output_dir.join("flowchart.mmd"), graph_source.as_str())?;

    let bootstrap = derive_bootstrap_state(&flowchart, &scenario_ir, current_node_ref)?;
    materialize_bootstrap_files(output_dir, &scenario_ir, &bootstrap)?;
    let current_step_surface = current_node_owned_paths(&scenario_ir, &bootstrap)
        .into_iter()
        .collect::<Vec<_>>();

    let report = check_workdir(output_dir)?;
    if !report.is_valid() {
        return Err(QianjiError::Topology(format!(
            "Generated anchored work surface `{}` failed validation:\n{}",
            output_dir.display(),
            render_workdir_check_markdown(&report)
        )));
    }

    Ok(AnchoredMaterializedWorkdir {
        plan_name,
        anchor_manifest_path,
        graph_path,
        output_dir: output_dir.to_path_buf(),
        visible_surfaces,
        current_node: bootstrap.current_node_label,
        allowed_next: bootstrap.allowed_next_labels,
        current_step_surface,
    })
}

/// Render one anchored materialization result into a markdown control-plane
/// surface.
#[must_use]
pub fn render_anchored_materialized_workdir(materialized: &AnchoredMaterializedWorkdir) -> String {
    render_show_surface(
        "Materialized Work Surface",
        &[
            format!("Scenario: {}", materialized.plan_name),
            format!(
                "Anchor: {}",
                display_graph_path(&materialized.anchor_manifest_path)
            ),
            format!("Graph: {}", display_graph_path(&materialized.graph_path)),
            format!("Run: {}", display_graph_path(&materialized.output_dir)),
        ],
        &[
            MarkdownShowSection {
                title: "Current State".into(),
                lines: vec![
                    format!("Current node: {}", materialized.current_node),
                    format!(
                        "Allowed next: {}",
                        render_label_list(materialized.allowed_next.as_slice())
                    ),
                ],
            },
            MarkdownShowSection {
                title: "Visible Surface".into(),
                lines: materialized
                    .visible_surfaces
                    .iter()
                    .map(|surface| format!("- {surface}"))
                    .collect(),
            },
            MarkdownShowSection {
                title: "Current Step Surface".into(),
                lines: if materialized.current_step_surface.is_empty() {
                    vec!["- Current node does not declare localized checkpoint or writes.".into()]
                } else {
                    materialized
                        .current_step_surface
                        .iter()
                        .map(|path| format!("- {path}"))
                        .collect()
                },
            },
        ],
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BootstrapState {
    current_node_id: String,
    current_node_label: String,
    allowed_next_labels: Vec<String>,
}

fn derive_visible_surfaces(require: &[String]) -> Vec<String> {
    let mut surfaces = vec!["flowchart.mmd".to_string()];
    let mut seen = BTreeSet::from(["flowchart.mmd".to_string()]);

    for path in require {
        let trimmed = path.trim();
        if trimmed.is_empty() || trimmed == "qianji.toml" || trimmed == "flowchart.mmd" {
            continue;
        }
        let surface = trimmed
            .split('/')
            .next()
            .unwrap_or(trimmed)
            .trim_end_matches('/');
        if surface.is_empty() {
            continue;
        }
        if seen.insert(surface.to_string()) {
            surfaces.push(surface.to_string());
        }
    }

    surfaces
}

fn materialize_surface_dirs(
    output_dir: &Path,
    visible_surfaces: &[String],
) -> Result<(), QianjiError> {
    for surface in visible_surfaces {
        if surface == "flowchart.mmd" {
            continue;
        }
        fs::create_dir_all(output_dir.join(surface)).map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to create localized surface `{}` under `{}`: {error}",
                surface,
                output_dir.display()
            ))
        })?;
    }
    Ok(())
}

fn derive_bootstrap_state(
    flowchart: &MermaidFlowchart,
    scenario_ir: &FlowhubScenarioIr,
    current_node_ref: Option<&str>,
) -> Result<BootstrapState, QianjiError> {
    let current_node = match current_node_ref {
        Some(node_ref) => resolve_selected_current_node(flowchart, scenario_ir, node_ref)?,
        None => derive_start_node(flowchart, scenario_ir)?,
    };

    let mut allowed_next_labels = flowchart
        .edges
        .iter()
        .filter(|edge| edge.from == current_node.id)
        .filter_map(|edge| {
            flowchart
                .nodes
                .iter()
                .find(|node| node.id == edge.to)
                .map(|node| node.label.clone())
        })
        .collect::<Vec<_>>();
    allowed_next_labels.sort();
    allowed_next_labels.dedup();

    Ok(BootstrapState {
        current_node_id: current_node.id,
        current_node_label: current_node.label,
        allowed_next_labels,
    })
}

fn resolve_selected_current_node(
    flowchart: &MermaidFlowchart,
    scenario_ir: &FlowhubScenarioIr,
    current_node_ref: &str,
) -> Result<WorkdirRuntimeNode, QianjiError> {
    resolve_runtime_node(flowchart, current_node_ref).ok_or_else(|| {
        QianjiError::Topology(format!(
            "Flowhub graph `{}` does not resolve current node `{current_node_ref}` to one Mermaid node",
            scenario_ir.merimind_graph_name
        ))
    })
}

fn derive_start_node(
    flowchart: &MermaidFlowchart,
    scenario_ir: &FlowhubScenarioIr,
) -> Result<WorkdirRuntimeNode, QianjiError> {
    let mut inbound = BTreeSet::new();
    for edge in &flowchart.edges {
        inbound.insert(edge.to.as_str());
    }

    let start_nodes = flowchart
        .nodes
        .iter()
        .filter(|node| !inbound.contains(node.id.as_str()))
        .collect::<Vec<_>>();
    let start_node = match start_nodes.as_slice() {
        [start] => WorkdirRuntimeNode {
            id: start.id.clone(),
            label: start.label.clone(),
        },
        [] => {
            return Err(QianjiError::Topology(format!(
                "Flowhub graph `{}` does not expose a unique start node for materialization",
                scenario_ir.merimind_graph_name
            )));
        }
        starts => {
            return Err(QianjiError::Topology(format!(
                "Flowhub graph `{}` exposes multiple start nodes for materialization: {}",
                scenario_ir.merimind_graph_name,
                starts
                    .iter()
                    .map(|node| node.label.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
    };

    Ok(start_node)
}

fn materialize_bootstrap_files(
    output_dir: &Path,
    scenario_ir: &FlowhubScenarioIr,
    bootstrap: &BootstrapState,
) -> Result<(), QianjiError> {
    let workdir = scenario_ir.workdir.as_ref().ok_or_else(|| {
        QianjiError::Topology(format!(
            "Flowhub graph `{}` does not declare a localized workdir contract",
            scenario_ir.merimind_graph_name
        ))
    })?;
    let current_node_owned_paths = current_node_owned_paths(scenario_ir, bootstrap);
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

    for required_path in &workdir.check.require {
        if required_path == "qianji.toml" || required_path == "flowchart.mmd" {
            continue;
        }
        if node_owned_paths.contains(required_path.as_str())
            && !current_node_owned_paths.contains(required_path.as_str())
        {
            continue;
        }
        let target_path = output_dir.join(required_path);
        if required_path.ends_with('/') {
            fs::create_dir_all(&target_path).map_err(|error| {
                QianjiError::Topology(format!(
                    "Failed to create localized bootstrap directory `{}`: {error}",
                    target_path.display()
                ))
            })?;
            continue;
        }

        let content = bootstrap_file_content(required_path, bootstrap)?;
        write_file(&target_path, content.as_str())?;
    }

    Ok(())
}

fn current_node_owned_paths(
    scenario_ir: &FlowhubScenarioIr,
    bootstrap: &BootstrapState,
) -> BTreeSet<String> {
    scenario_ir
        .node_contract(bootstrap.current_node_label.as_str())
        .map_or_else(BTreeSet::new, |node| {
            node.checkpoint
                .iter()
                .cloned()
                .chain(node.writes.iter().cloned())
                .collect()
        })
}

fn bootstrap_file_content(
    required_path: &str,
    bootstrap: &BootstrapState,
) -> Result<String, QianjiError> {
    match required_path {
        "state/current_node.toml" => Ok(format!(
            "current_node = {:?}\n",
            bootstrap.current_node_label
        )),
        "state/allowed_next.json" => serde_json::to_string_pretty(&bootstrap.allowed_next_labels)
            .map(|json| format!("{json}\n"))
            .map_err(|error| {
                QianjiError::Topology(format!(
                    "Failed to serialize bootstrap allowed-next state: {error}"
                ))
            }),
        "state/trace.jsonl" => Ok(String::new()),
        "diagnostics/latest_check.md" => Ok(format!(
            "# Materialized Run Root\n\nBootstrap current node: `{}` (`{}`).\n",
            bootstrap.current_node_label, bootstrap.current_node_id
        )),
        path if has_extension(path, "json") => Ok("{}\n".to_string()),
        path if has_extension(path, "jsonl") => Ok(String::new()),
        path if has_extension(path, "md") => Ok(String::new()),
        _ => Ok(String::new()),
    }
}

fn has_extension(path: &str, extension: &str) -> bool {
    Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
}

fn write_file(path: &Path, content: &str) -> Result<(), QianjiError> {
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
            "Failed to write localized bootstrap file `{}`: {error}",
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
