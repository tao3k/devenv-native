use std::path::Path;

use crate::contracts::FlowhubGraphContract;
use crate::error::QianjiError;
use crate::flowhub::MermaidFlowchart;

use super::scenario_ir_annotations::FlowhubGraphAnnotations;
use super::scenario_ir_compile_legacy::compile_legacy_scenario_ir;
use super::scenario_ir_compile_nodes::compile_enriched_nodes;
use super::scenario_ir_compile_workdir::compile_enriched_workdir;
use super::scenario_ir_model::FlowhubScenarioIr;

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

#[cfg(test)]
#[path = "../../../tests/unit/flowhub/scenario_ir/compile.rs"]
mod tests;
