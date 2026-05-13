use std::path::Path;

use crate::contracts::FlowhubGraphContract;
use crate::error::QianjiError;
use crate::flowhub::discover::FlowhubDiscoveredModule;
use crate::flowhub::load::load_flowhub_root_manifest;
use crate::flowhub::parse_mermaid_flowchart;

use super::model::{FlowhubModuleKind, FlowhubModuleSummary, FlowhubScenarioCaseSummary};
use crate::flowhub::{parse_flowhub_graph_annotations, resolve_flowhub_graph_name};

pub(super) fn module_summary(
    module: &FlowhubDiscoveredModule,
    known_module_names: &[String],
) -> Result<FlowhubModuleSummary, QianjiError> {
    let child_modules = module
        .manifest
        .contract
        .as_ref()
        .map(|contract| {
            contract
                .register
                .iter()
                .map(|child_ref| {
                    resolve_child_module_ref(&module.module_dir, &module.module_ref, child_ref)
                })
                .collect()
        })
        .unwrap_or_default();
    let scenario_cases = discover_immediate_scenario_cases(
        &module.module_dir,
        &module.manifest.graph,
        known_module_names,
    )?;

    Ok(FlowhubModuleSummary {
        module_ref: module.module_ref.clone(),
        module_name: module.manifest.module.name.clone(),
        module_dir: module.module_dir.clone(),
        kind: if module_owns_child_graphs(module) {
            FlowhubModuleKind::Composite
        } else {
            FlowhubModuleKind::Leaf
        },
        exports_entry: module.manifest.exports.entry.clone(),
        exports_ready: module.manifest.exports.ready.clone(),
        child_modules,
        scenario_cases,
    })
}

fn module_owns_child_graphs(module: &FlowhubDiscoveredModule) -> bool {
    module
        .manifest
        .contract
        .as_ref()
        .is_some_and(|contract| !contract.register.is_empty())
        || module.manifest.template.is_some()
}

fn resolve_child_module_ref(
    parent_module_dir: &Path,
    parent_module_ref: &str,
    child_module_ref: &str,
) -> String {
    if parent_module_dir.join(child_module_ref).is_dir() {
        return format!("{parent_module_ref}/{child_module_ref}");
    }
    child_module_ref.to_string()
}

pub(super) fn discover_immediate_scenario_cases(
    module_dir: &Path,
    graph_contracts: &[FlowhubGraphContract],
    known_module_names: &[String],
) -> Result<Vec<FlowhubScenarioCaseSummary>, QianjiError> {
    let mut scenario_cases = std::fs::read_dir(module_dir)
        .map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to inspect Flowhub module directory `{}`: {error}",
                module_dir.display()
            ))
        })?
        .map(|entry| {
            let entry = entry.map_err(|error| {
                QianjiError::Topology(format!(
                    "Failed to inspect Flowhub module directory `{}`: {error}",
                    module_dir.display()
                ))
            })?;
            Ok(entry.path())
        })
        .collect::<Result<Vec<_>, QianjiError>>()?
        .into_iter()
        .filter(|path| path.is_file())
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("mmd"))
        .filter_map(|path| summarize_scenario_case(&path, graph_contracts, known_module_names))
        .collect::<Vec<_>>();
    scenario_cases.sort_by(|left, right| left.file_name.cmp(&right.file_name));
    Ok(scenario_cases)
}

fn summarize_scenario_case(
    path: &Path,
    graph_contracts: &[FlowhubGraphContract],
    known_module_names: &[String],
) -> Option<FlowhubScenarioCaseSummary> {
    let file_name = path.file_name()?.to_str()?.to_string();
    let file_stem = path.file_stem()?.to_str()?.to_string();
    let declared_graph = graph_contracts.iter().find(|graph| graph.path == file_name);
    let merimind_graph_name = std::fs::read_to_string(path)
        .ok()
        .and_then(|source| {
            let annotations = parse_flowhub_graph_annotations(&source).ok().flatten();
            let graph_name = resolve_flowhub_graph_name(
                annotations.as_ref(),
                declared_graph,
                file_stem.as_str(),
            );
            parse_mermaid_flowchart(&source, graph_name.as_str(), known_module_names).ok()
        })
        .map_or_else(
            || resolve_flowhub_graph_name(None, declared_graph, file_stem.as_str()),
            |flowchart| flowchart.merimind_graph_name,
        );

    Some(FlowhubScenarioCaseSummary {
        file_name,
        merimind_graph_name,
    })
}

pub(super) fn load_known_module_names_for_show(
    module_dir: &Path,
) -> Result<Vec<String>, QianjiError> {
    let Some(root_dir) = module_dir.parent() else {
        return Ok(Vec::new());
    };
    let root_manifest_path = root_dir.join("qianji.toml");
    if !root_manifest_path.is_file() {
        return Ok(Vec::new());
    }

    match load_flowhub_root_manifest(&root_manifest_path) {
        Ok(manifest) => Ok(manifest.contract.register),
        Err(error) => Err(QianjiError::Topology(format!(
            "Failed to load Flowhub root manifest `{}` while summarizing scenario cases: {error}",
            root_manifest_path.display()
        ))),
    }
}
