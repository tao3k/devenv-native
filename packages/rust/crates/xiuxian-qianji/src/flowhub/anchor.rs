use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::contracts::FlowhubGraphContract;
use crate::error::QianjiError;

use super::load::load_flowhub_module_manifest;

pub(crate) fn resolve_anchor_manifest_path(anchor: &Path) -> PathBuf {
    if anchor.is_dir() {
        anchor.join("qianji.toml")
    } else {
        anchor.to_path_buf()
    }
}

pub(crate) fn resolve_anchored_graph(
    anchor_manifest_path: &Path,
    scenario_ref: &str,
) -> Result<PathBuf, QianjiError> {
    let module_manifest = load_flowhub_module_manifest(anchor_manifest_path)?;
    if module_manifest.graph.is_empty() {
        return Err(QianjiError::Topology(format!(
            "Flowhub anchor `{}` does not declare any `[[graph]]` scenarios",
            anchor_manifest_path.display()
        )));
    }

    let anchor_dir = anchor_manifest_path.parent().ok_or_else(|| {
        QianjiError::Topology(format!(
            "Flowhub anchor `{}` has no parent directory",
            anchor_manifest_path.display()
        ))
    })?;
    let normalized_ref = normalize_identifier(scenario_ref);

    let direct_matches = module_manifest
        .graph
        .iter()
        .filter(|graph| direct_graph_ref_matches(graph, normalized_ref.as_str()))
        .collect::<Vec<_>>();
    if let Some(graph_path) = resolve_unique_graph_match(
        anchor_manifest_path,
        scenario_ref,
        anchor_dir,
        &direct_matches,
        "direct graph refs",
    )? {
        return Ok(graph_path);
    }

    let annotation_matches = module_manifest
        .graph
        .iter()
        .filter(|graph| {
            graph_annotation_scenario_id(anchor_dir, graph).is_some_and(|scenario_id| {
                normalize_identifier(scenario_id.as_str()) == normalized_ref
            })
        })
        .collect::<Vec<_>>();
    if let Some(graph_path) = resolve_unique_graph_match(
        anchor_manifest_path,
        scenario_ref,
        anchor_dir,
        &annotation_matches,
        "annotation scenario ids",
    )? {
        return Ok(graph_path);
    }

    Err(QianjiError::Topology(format!(
        "Flowhub anchor `{}` does not declare scenario `{scenario_ref}`; available refs: {}",
        anchor_manifest_path.display(),
        available_scenario_refs(anchor_dir, &module_manifest.graph).join(", ")
    )))
}

fn resolve_unique_graph_match(
    anchor_manifest_path: &Path,
    scenario_ref: &str,
    anchor_dir: &Path,
    matches: &[&FlowhubGraphContract],
    match_kind: &str,
) -> Result<Option<PathBuf>, QianjiError> {
    match matches {
        [] => Ok(None),
        [graph] => Ok(Some(anchor_dir.join(&graph.path))),
        graphs => Err(QianjiError::Topology(format!(
            "Flowhub anchor `{}` resolves scenario `{scenario_ref}` ambiguously through {match_kind}: {}",
            anchor_manifest_path.display(),
            graphs
                .iter()
                .map(|graph| graph.path.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

fn direct_graph_ref_matches(graph: &FlowhubGraphContract, normalized_ref: &str) -> bool {
    if normalize_identifier(graph.path.as_str()) == normalized_ref {
        return true;
    }

    let file_stem = Path::new(graph.path.as_str())
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(graph.path.as_str());
    normalize_identifier(file_stem) == normalized_ref
        || graph
            .name
            .as_deref()
            .is_some_and(|name| normalize_identifier(name) == normalized_ref)
}

fn graph_annotation_scenario_id(anchor_dir: &Path, graph: &FlowhubGraphContract) -> Option<String> {
    let source = fs::read_to_string(anchor_dir.join(&graph.path)).ok()?;
    super::parse_flowhub_graph_annotations(&source)
        .ok()
        .flatten()
        .and_then(|annotations| annotations.scenario.id)
}

fn available_scenario_refs(anchor_dir: &Path, graphs: &[FlowhubGraphContract]) -> Vec<String> {
    let mut refs = BTreeSet::new();
    for graph in graphs {
        if let Some(name) = &graph.name {
            refs.insert(name.clone());
        }
        refs.insert(graph.path.clone());
        if let Some(file_stem) = Path::new(graph.path.as_str())
            .file_stem()
            .and_then(|stem| stem.to_str())
        {
            refs.insert(file_stem.to_string());
        }
        if let Some(scenario_id) = graph_annotation_scenario_id(anchor_dir, graph) {
            refs.insert(scenario_id);
        }
    }
    refs.into_iter().collect()
}

fn normalize_identifier(value: &str) -> String {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .filter(char::is_ascii_alphanumeric)
        .collect()
}
