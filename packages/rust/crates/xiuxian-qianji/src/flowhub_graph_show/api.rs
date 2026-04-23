use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::contracts::FlowhubGraphTopology;
use crate::error::QianjiError;
use crate::flowhub::{FlowhubScenarioIr, MermaidFlowchart};

use super::load::LoadedFlowhubGraphContext;
use super::{build, load, render, render_surface};

/// One Flowhub Mermaid graph contract preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubGraphShow {
    /// Mermaid graph file on disk.
    pub graph_path: PathBuf,
    /// Stable graph identity resolved from `[[graph]].name` or the filename
    /// stem fallback.
    pub merimind_graph_name: String,
    /// Optional scenario id declared in Mermaid annotations.
    pub scenario_id: Option<String>,
    /// Optional scenario description declared in Mermaid annotations.
    pub description: Option<String>,
    /// Resolved topology from petgraph analysis.
    pub topology: FlowhubGraphTopology,
    /// Optional module-owned declared topology.
    pub declared_topology: Option<FlowhubGraphTopology>,
    /// Raw Mermaid source.
    pub mermaid: String,
    /// Owning Flowhub module reference.
    pub owning_module_ref: String,
    /// Flowhub root containing the owning module.
    pub flowhub_root: PathBuf,
    /// Declared Mermaid direction such as `LR`.
    pub direction: String,
    /// Parsed nodes with semantic guidance in declaration order.
    pub nodes: Vec<FlowhubGraphNodeSummary>,
    /// Parsed edges in declaration order.
    pub edges: Vec<FlowhubGraphEdgeSummary>,
    /// Registered Flowhub modules that are missing from the Mermaid graph.
    pub missing_registered_modules: Vec<String>,
    /// Mermaid nodes outside the registered-module set and allowed graph vocabulary.
    pub unknown_graph_nodes: Vec<String>,
    /// Node labels grouped by cyclic SCC when the graph loops.
    pub cyclic_components: Vec<Vec<String>>,
    /// Static Flowhub module-owned contract entries for the graph source.
    pub module_contract_surface: Vec<String>,
    /// Declared bounded check surface for executor guidance.
    pub declared_check_surface: FlowhubGraphCheckSurface,
    /// Owning module manifest source.
    pub owning_module_manifest_toml: String,
}

/// Declared bounded check surface derived from one graph contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubGraphCheckSurface {
    /// Optional note that explains how the localized surface should be used.
    pub note: Option<String>,
    /// Optional localized run root declared by the Flowhub contract.
    pub root: Option<String>,
    /// Raw `check.require` paths or globs declared by Flowhub.
    pub required_paths: Vec<String>,
    /// Raw `check.flowchart` surfaces declared by Flowhub.
    pub flowchart_surfaces: Vec<String>,
    /// Optional persistent canonical target tree for validated merges.
    pub persistent_target_surface_tree: Vec<String>,
    /// Optional declared done-gate paths over the persistent target surface.
    pub done_gate_require: Vec<String>,
}

/// One parsed Flowhub Mermaid node summary with semantic guidance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubGraphNodeSummary {
    /// Stable Mermaid node id.
    pub id: String,
    /// Visible Mermaid label.
    pub label: String,
    /// Contract-owned node semantic kind when declared.
    pub kind: Option<String>,
    /// Stable role description for Codex.
    pub role: String,
    /// Stable agent action guidance for the node.
    pub agent_action: String,
    /// Visible next-node labels in edge order.
    pub next: Vec<String>,
    /// Resolved Flowhub module ref when the node represents a registered module.
    pub module_ref: Option<String>,
    /// Stable module entry export when available.
    pub exports_entry: Option<String>,
    /// Stable module ready export when available.
    pub exports_ready: Option<String>,
}

/// One extracted graph edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubGraphEdgeSummary {
    /// Edge source label.
    pub from_label: String,
    /// Edge destination label.
    pub to_label: String,
}

/// Load and summarize one Flowhub Mermaid graph file.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the target is not a Mermaid file
/// owned by a Flowhub module or when the Flowhub manifests cannot be loaded.
pub fn show_flowhub_graph(graph_path: impl AsRef<Path>) -> Result<FlowhubGraphShow, QianjiError> {
    let graph_path = graph_path.as_ref();
    load::validate_graph_path(graph_path)?;
    let LoadedFlowhubGraphContext {
        owning_module,
        flowhub_root,
        module_exports,
        owning_module_manifest_toml,
        source,
        flowchart,
        topology,
        cyclic_components,
        scenario_ir,
        declared_topology,
        allowed_graph_node_labels,
    } = load::load_flowhub_graph_context(graph_path)?;

    let nodes_by_id = flowchart
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node.label.as_str()))
        .collect::<BTreeMap<_, _>>();
    let unknown_graph_nodes =
        build::collect_unknown_graph_nodes(&flowchart, &allowed_graph_node_labels);
    let next_by_node_id = build::build_next_labels_by_node_id(&flowchart.edges, &nodes_by_id);
    let nodes = build::build_graph_node_summaries(
        &flowchart,
        &module_exports,
        &next_by_node_id,
        scenario_ir.as_ref(),
    );
    let edges = build::build_graph_edge_summaries(&flowchart, &nodes_by_id);
    let module_contract_surface = build::module_contract_surface(&owning_module);
    let declared_check_surface = render_surface::declared_check_surface(scenario_ir.as_ref());

    Ok(FlowhubGraphShow {
        graph_path: graph_path.to_path_buf(),
        merimind_graph_name: flowchart.merimind_graph_name,
        scenario_id: scenario_ir
            .as_ref()
            .and_then(|graph| graph.scenario_id.clone()),
        description: scenario_ir
            .as_ref()
            .and_then(|graph| graph.description.clone()),
        topology,
        declared_topology,
        mermaid: source,
        owning_module_ref: owning_module.module_ref,
        flowhub_root,
        direction: flowchart.direction,
        nodes,
        edges,
        missing_registered_modules: Vec::new(),
        unknown_graph_nodes,
        cyclic_components,
        module_contract_surface,
        declared_check_surface,
        owning_module_manifest_toml,
    })
}

pub(crate) fn load_flowhub_graph_runtime_contract(
    graph_path: &Path,
) -> Result<(MermaidFlowchart, Option<FlowhubScenarioIr>), QianjiError> {
    let LoadedFlowhubGraphContext {
        flowchart,
        scenario_ir,
        ..
    } = load::load_flowhub_graph_context(graph_path)?;
    Ok((flowchart, scenario_ir))
}

/// Render one Flowhub Mermaid graph contract preview into markdown.
#[must_use]
pub fn render_flowhub_graph_show(show: &FlowhubGraphShow) -> String {
    render::render_flowhub_graph_show_impl(show)
}
