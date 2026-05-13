//! Public entry points for `Flowhub` graph-show previews.

use std::collections::BTreeMap;
use std::path::Path;

use crate::error::QianjiError;
use crate::flowhub::{FlowhubScenarioIr, MermaidFlowchart};

use super::load::LoadedFlowhubGraphContext;
use super::model::FlowhubGraphShow;
use super::{build, load, render, render_surface};

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
