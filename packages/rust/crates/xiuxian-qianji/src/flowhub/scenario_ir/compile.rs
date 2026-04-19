use std::collections::BTreeSet;
use std::path::Path;

use crate::contracts::{
    FlowhubGraphContract, FlowhubGraphNodeContract, FlowhubGraphSurfaceContract, WorkdirCheck,
};
use crate::error::QianjiError;
use crate::flowhub::mermaid::{MermaidFlowchart, MermaidNodeKind, normalize_graph_node_label};

use super::annotations::{FlowhubGraphAnnotations, FlowhubGraphNodeAnnotations};
use super::model::{FlowhubScenarioIr, FlowhubScenarioNodeIr, FlowhubScenarioWorkdirIr};

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

/// Resolve the LLM-facing graph identity from annotations, legacy TOML, or the
/// filename stem fallback.
pub(crate) fn resolve_flowhub_graph_name(
    annotations: Option<&FlowhubGraphAnnotations>,
    declared_graph: Option<&FlowhubGraphContract>,
    fallback_graph_name: &str,
) -> String {
    if let Some(name) = annotations.and_then(|value| value.scenario.name.as_deref()) {
        return name.to_string();
    }

    declared_graph.map_or_else(
        || fallback_graph_name.to_string(),
        |graph| graph.resolved_name_or(fallback_graph_name).to_string(),
    )
}

/// Compile one Mermaid scenario-case contract from either enriched annotations
/// or the legacy TOML graph contract.
pub(crate) fn compile_flowhub_scenario_ir(
    graph_path: &Path,
    resolved_graph_name: &str,
    flowchart: &MermaidFlowchart,
    annotations: Option<&FlowhubGraphAnnotations>,
    declared_graph: Option<&FlowhubGraphContract>,
) -> Result<Option<FlowhubScenarioIr>, QianjiError> {
    if let Some(annotations) = annotations {
        return compile_enriched_scenario_ir(
            graph_path,
            resolved_graph_name,
            flowchart,
            annotations,
        )
        .map(Some);
    }

    Ok(declared_graph.map(|graph| compile_legacy_scenario_ir(resolved_graph_name, graph)))
}

fn compile_enriched_scenario_ir(
    graph_path: &Path,
    resolved_graph_name: &str,
    flowchart: &MermaidFlowchart,
    annotations: &FlowhubGraphAnnotations,
) -> Result<FlowhubScenarioIr, QianjiError> {
    let nodes = compile_enriched_nodes(graph_path, flowchart, annotations)?;
    let workdir = Some(compile_enriched_workdir(graph_path, annotations, &nodes)?);

    Ok(FlowhubScenarioIr {
        merimind_graph_name: resolved_graph_name.to_string(),
        scenario_id: annotations.scenario.id.clone(),
        description: annotations.scenario.description.clone(),
        declared_topology: annotations.scenario.topology,
        workdir,
        nodes,
    })
}

fn compile_enriched_nodes(
    graph_path: &Path,
    flowchart: &MermaidFlowchart,
    annotations: &FlowhubGraphAnnotations,
) -> Result<Vec<FlowhubScenarioNodeIr>, QianjiError> {
    let mut nodes = Vec::new();
    let mut seen_labels = BTreeSet::new();

    for (node_ref, node_annotations) in &annotations.nodes {
        let label = resolve_annotation_node_label(flowchart, node_ref.as_str()).map_err(|error| {
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
        .collect::<BTreeSet<_>>();
    nodes.sort_by_key(|node| {
        node_order
            .iter()
            .find_map(|(label, index)| {
                (label == &normalize_graph_node_label(node.label.as_str())).then_some(*index)
            })
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

fn compile_enriched_workdir(
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
        if annotations.scenario.target_paths.is_empty() && annotations.done_gate_require.is_empty() {
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
    let allowed_targets = target_paths.iter().map(String::as_str).collect::<BTreeSet<_>>();
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
    let mut require = Vec::new();
    let mut seen = BTreeSet::new();

    for entry in DEFAULT_WORKDIR_PREFIX_REQUIRE {
        push_unique(&mut require, &mut seen, entry);
    }
    for entry in &annotations.scenario.requires {
        push_unique(&mut require, &mut seen, entry);
    }
    for entry in DEFAULT_WORKDIR_STATE_REQUIRE {
        push_unique(&mut require, &mut seen, entry);
    }
    for node in nodes {
        if let Some(checkpoint) = &node.checkpoint {
            push_unique(&mut require, &mut seen, checkpoint);
        }
    }
    for node in nodes {
        for write in &node.writes {
            push_unique(&mut require, &mut seen, write);
        }
    }
    for entry in DEFAULT_WORKDIR_DIAGNOSTIC_REQUIRE {
        push_unique(&mut require, &mut seen, entry);
    }

    require
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

fn compile_legacy_scenario_ir(
    resolved_graph_name: &str,
    graph: &FlowhubGraphContract,
) -> FlowhubScenarioIr {
    FlowhubScenarioIr {
        merimind_graph_name: resolved_graph_name.to_string(),
        scenario_id: None,
        description: None,
        declared_topology: Some(graph.topology),
        workdir: graph.workdir.as_ref().map(|workdir| FlowhubScenarioWorkdirIr {
            note: workdir.note.clone(),
            root: workdir.root.clone(),
            check: workdir.check.clone(),
            target: workdir.target.clone(),
            done_gate_require: workdir
                .target
                .as_ref()
                .map(|target| target.paths.clone())
                .unwrap_or_default(),
        }),
        nodes: graph
            .node
            .iter()
            .map(compose_legacy_node_ir)
            .collect(),
    }
}

fn compose_legacy_node_ir(node: &FlowhubGraphNodeContract) -> FlowhubScenarioNodeIr {
    FlowhubScenarioNodeIr {
        label: node.label.clone(),
        kind: Some(node.kind.clone()),
        role: Some(node.role.clone()),
        agent_action: Some(node.agent_action.clone()),
        checkpoint: None,
        writes: Vec::new(),
        merge_target: Vec::new(),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/flowhub/scenario_ir/compile.rs"]
mod tests;
