use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::contracts::{FlowhubGraphContract, FlowhubGraphTopology};
use crate::error::QianjiError;
use crate::flowhub::discover::{
    FlowhubDiscoveredModule, discover_all_flowhub_module_refs, find_flowhub_root_for_module_dir,
    load_flowhub_module_candidate, module_candidate_from_dir, module_candidate_from_ref,
};
use crate::flowhub::{
    FlowhubScenarioIr, MermaidFlowchart, analyze_mermaid_flowchart_topology,
    compile_flowhub_scenario_ir, parse_flowhub_graph_annotations, parse_mermaid_flowchart,
    resolve_flowhub_graph_name,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ModuleExports {
    pub(super) entry: String,
    pub(super) ready: String,
}

#[derive(Debug)]
pub(super) struct LoadedFlowhubGraphContext {
    pub(super) owning_module: FlowhubDiscoveredModule,
    pub(super) flowhub_root: PathBuf,
    pub(super) module_exports: BTreeMap<String, ModuleExports>,
    pub(super) owning_module_manifest_toml: String,
    pub(super) source: String,
    pub(super) flowchart: MermaidFlowchart,
    pub(super) topology: FlowhubGraphTopology,
    pub(super) cyclic_components: Vec<Vec<String>>,
    pub(super) scenario_ir: Option<FlowhubScenarioIr>,
    pub(super) declared_topology: Option<FlowhubGraphTopology>,
    pub(super) allowed_graph_node_labels: BTreeSet<String>,
}

pub(super) fn validate_graph_path(graph_path: &Path) -> Result<(), QianjiError> {
    if !graph_path.is_file() {
        return Err(QianjiError::Topology(format!(
            "`{}` is not a Mermaid graph file",
            graph_path.display()
        )));
    }
    if graph_path
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("mmd")
    {
        return Err(QianjiError::Topology(format!(
            "`{}` is not a `.mmd` graph file",
            graph_path.display()
        )));
    }
    Ok(())
}

pub(super) fn load_flowhub_graph_context(
    graph_path: &Path,
) -> Result<LoadedFlowhubGraphContext, QianjiError> {
    let module_dir = graph_path.parent().ok_or_else(|| {
        QianjiError::Topology(format!(
            "Flowhub Mermaid graph `{}` has no parent module directory",
            graph_path.display()
        ))
    })?;
    let module_candidate = module_candidate_from_dir(module_dir)?;
    let owning_module = load_flowhub_module_candidate(&module_candidate)?;
    let flowhub_root = find_flowhub_root_for_module_dir(module_dir)?;
    let registered_modules = discover_all_flowhub_module_refs(&flowhub_root)?;
    let module_exports = load_registered_module_exports(&flowhub_root, &registered_modules)?;
    let owning_module_manifest_toml =
        fs::read_to_string(&owning_module.manifest_path).map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to read Flowhub module manifest `{}`: {error}",
                owning_module.manifest_path.display()
            ))
        })?;
    let source = fs::read_to_string(graph_path).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to read Flowhub Mermaid graph `{}`: {error}",
            graph_path.display()
        ))
    })?;
    let fallback_graph_name = graph_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| {
            QianjiError::Topology(format!(
                "Failed to derive Mermaid graph name from `{}`",
                graph_path.display()
            ))
        })?;
    let declared_graph = declared_graph_contract(&owning_module, graph_path).cloned();
    let annotations = parse_flowhub_graph_annotations(&source).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to parse Flowhub Mermaid annotations from `{}`: {error}",
            graph_path.display()
        ))
    })?;
    let merimind_graph_name = resolve_flowhub_graph_name(
        annotations.as_ref(),
        declared_graph.as_ref(),
        fallback_graph_name,
    );
    let flowchart =
        parse_mermaid_flowchart(&source, merimind_graph_name.as_str(), &registered_modules)
            .map_err(|error| {
                QianjiError::Topology(format!(
                    "Failed to parse Flowhub Mermaid graph `{}`: {error}",
                    graph_path.display()
                ))
            })?;
    let scenario_ir = compile_flowhub_scenario_ir(
        graph_path,
        merimind_graph_name.as_str(),
        &flowchart,
        annotations.as_ref(),
        declared_graph.as_ref(),
    )?;
    let topology_analysis = analyze_mermaid_flowchart_topology(&flowchart);
    let declared_topology = scenario_ir
        .as_ref()
        .and_then(|graph| graph.declared_topology);
    let allowed_graph_node_labels = scenario_ir
        .as_ref()
        .map_or_else(BTreeSet::new, FlowhubScenarioIr::allowed_graph_node_labels);

    Ok(LoadedFlowhubGraphContext {
        owning_module,
        flowhub_root,
        module_exports,
        owning_module_manifest_toml,
        source,
        flowchart,
        topology: topology_analysis.topology,
        cyclic_components: topology_analysis.cyclic_components,
        scenario_ir,
        declared_topology,
        allowed_graph_node_labels,
    })
}

fn load_registered_module_exports(
    flowhub_root: &Path,
    registered_modules: &[String],
) -> Result<BTreeMap<String, ModuleExports>, QianjiError> {
    registered_modules
        .iter()
        .map(|module_ref| {
            let module = load_flowhub_module_candidate(&module_candidate_from_ref(
                flowhub_root,
                module_ref,
            ))?;
            Ok((
                module_ref.clone(),
                ModuleExports {
                    entry: module.manifest.exports.entry,
                    ready: module.manifest.exports.ready,
                },
            ))
        })
        .collect()
}

fn declared_graph_contract<'a>(
    owning_module: &'a FlowhubDiscoveredModule,
    graph_path: &Path,
) -> Option<&'a FlowhubGraphContract> {
    let file_name = graph_path.file_name()?.to_str()?;
    owning_module
        .manifest
        .graph
        .iter()
        .find(|graph| graph.path == file_name)
}
