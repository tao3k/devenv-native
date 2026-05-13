use std::collections::BTreeSet;
use std::path::Path;

use crate::contracts::{FlowhubGraphSurfaceContract, WorkdirCheck};
use crate::error::QianjiError;

use super::scenario_ir_annotations::FlowhubGraphAnnotations;
use super::scenario_ir_model::{FlowhubScenarioNodeIr, FlowhubScenarioWorkdirIr};

const DEFAULT_WORKDIR_PREFIX_REQUIRE: [&str; 2] = ["qianji.toml", "flowchart.mmd"];
const DEFAULT_WORKDIR_STATE_REQUIRE: [&str; 3] = [
    "state/current_node.toml",
    "state/trace.jsonl",
    "state/allowed_next.json",
];
const DEFAULT_WORKDIR_DIAGNOSTIC_REQUIRE: [&str; 4] = [
    "diagnostics/latest_check.md",
    "diagnostics/blocked.json",
    "diagnostics/failed.json",
    "outputs/response_preview.md",
];
const DEFAULT_FLOWCHART_SURFACES: [&str; 3] = ["state", "checkpoints", "staging"];

pub(super) fn compile_enriched_workdir(
    graph_path: &Path,
    annotations: &FlowhubGraphAnnotations,
    nodes: &[FlowhubScenarioNodeIr],
) -> Result<FlowhubScenarioWorkdirIr, QianjiError> {
    let root = annotations
        .scenario
        .workdir_root
        .as_ref()
        .ok_or_else(|| {
            QianjiError::Topology(format!(
                "Failed to compile Flowhub Mermaid contract `{}`: missing `qianji.scenario.workdir_root`",
                graph_path.display()
            ))
        })?
        .clone();

    let target = compile_target_surface(graph_path, annotations)?;
    let target_paths = target
        .as_ref()
        .map(|surface| surface.paths.as_slice())
        .unwrap_or_default();
    validate_target_ownership(graph_path, annotations, nodes, target_paths)?;

    let require = derive_enriched_required_paths(annotations, nodes);
    let flowchart = derive_flowchart_surfaces(&require);

    Ok(FlowhubScenarioWorkdirIr {
        note: annotations.scenario.note.clone(),
        root,
        check: WorkdirCheck { require, flowchart },
        target,
        done_gate_require: annotations.done_gate_require.clone(),
    })
}

fn compile_target_surface(
    graph_path: &Path,
    annotations: &FlowhubGraphAnnotations,
) -> Result<Option<FlowhubGraphSurfaceContract>, QianjiError> {
    let Some(root) = annotations.scenario.target_root.as_ref() else {
        if annotations.scenario.target_paths.is_empty() && annotations.done_gate_require.is_empty()
        {
            return Ok(None);
        }
        return Err(QianjiError::Topology(format!(
            "Failed to compile Flowhub Mermaid contract `{}`: target paths require `qianji.scenario.target_root`",
            graph_path.display()
        )));
    };

    Ok(Some(FlowhubGraphSurfaceContract {
        root: root.clone(),
        paths: annotations.scenario.target_paths.clone(),
    }))
}

fn validate_target_ownership(
    graph_path: &Path,
    annotations: &FlowhubGraphAnnotations,
    nodes: &[FlowhubScenarioNodeIr],
    target_paths: &[String],
) -> Result<(), QianjiError> {
    let allowed_targets = target_paths
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    for required in &annotations.done_gate_require {
        if !allowed_targets.contains(required.as_str()) {
            return Err(QianjiError::Topology(format!(
                "Failed to compile Flowhub Mermaid contract `{}`: `qianji.done_gate.require` path `{required}` is outside `qianji.scenario.target_paths`",
                graph_path.display()
            )));
        }
    }

    for node in nodes {
        for target in &node.merge_target {
            if !allowed_targets.contains(target.as_str()) {
                return Err(QianjiError::Topology(format!(
                    "Failed to compile Flowhub Mermaid contract `{}`: node `{}` merge target `{target}` is outside `qianji.scenario.target_paths`",
                    graph_path.display(),
                    node.label
                )));
            }
        }
    }

    Ok(())
}

fn derive_enriched_required_paths(
    annotations: &FlowhubGraphAnnotations,
    nodes: &[FlowhubScenarioNodeIr],
) -> Vec<String> {
    DEFAULT_WORKDIR_PREFIX_REQUIRE
        .iter()
        .copied()
        .chain(annotations.scenario.requires.iter().map(String::as_str))
        .chain(DEFAULT_WORKDIR_STATE_REQUIRE.iter().copied())
        .chain(nodes.iter().filter_map(|node| node.checkpoint.as_deref()))
        .chain(
            nodes
                .iter()
                .flat_map(|node| node.writes.iter().map(String::as_str)),
        )
        .chain(DEFAULT_WORKDIR_DIAGNOSTIC_REQUIRE.iter().copied())
        .fold(
            (Vec::new(), BTreeSet::new()),
            |(mut require, mut seen), entry| {
                push_unique(&mut require, &mut seen, entry);
                (require, seen)
            },
        )
        .0
}

fn derive_flowchart_surfaces(require: &[String]) -> Vec<String> {
    let surfaces = require
        .iter()
        .filter_map(|entry| entry.split('/').next())
        .filter(|entry| *entry != "qianji.toml" && *entry != "flowchart.mmd")
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();

    let mut ordered = DEFAULT_FLOWCHART_SURFACES
        .iter()
        .filter(|entry| surfaces.contains(**entry))
        .map(|entry| (*entry).to_string())
        .collect::<Vec<_>>();

    if ordered.is_empty() {
        ordered.extend(surfaces);
    }

    ordered
}

fn push_unique(require: &mut Vec<String>, seen: &mut BTreeSet<String>, entry: impl AsRef<str>) {
    let entry = entry.as_ref().trim();
    if entry.is_empty() {
        return;
    }
    if seen.insert(entry.to_string()) {
        require.push(entry.to_string());
    }
}
