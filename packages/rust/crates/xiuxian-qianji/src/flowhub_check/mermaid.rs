use std::collections::BTreeSet;
use std::path::Path;

use crate::contracts::{FlowhubGraphContract, FlowhubStructureContract};
use crate::error::QianjiError;
use crate::flowhub::discover::FlowhubDiscoveredModule;
use crate::flowhub::{
    FlowhubGraphAnnotations, FlowhubScenarioIr, MermaidFlowchart,
    analyze_mermaid_flowchart_topology, compile_flowhub_scenario_ir,
    parse_flowhub_graph_annotations, parse_mermaid_flowchart, resolve_flowhub_graph_name,
    validate_mermaid_flowchart,
};

use super::api::FlowhubDiagnostic;
use super::contract::mermaid_file_is_contracted;

struct MermaidCaseValidation<'a> {
    module_ref: &'a str,
    scenario_case: &'a Path,
    file_name: &'a str,
    merimind_graph_name: &'a str,
    declared_graph: Option<&'a FlowhubGraphContract>,
    annotations: Option<&'a FlowhubGraphAnnotations>,
}

pub(super) fn validate_mermaid_case_files(
    module: &FlowhubDiscoveredModule,
    contract: Option<&FlowhubStructureContract>,
    known_module_names: &[String],
    diagnostics: &mut Vec<FlowhubDiagnostic>,
) -> Result<(), QianjiError> {
    for scenario_case in super::filesystem::discover_immediate_mermaid_files(&module.module_dir)? {
        let Some(file_name) = scenario_case.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if !mermaid_file_is_contracted(file_name, contract) {
            diagnostics.push(FlowhubDiagnostic {
                title: "Uncontracted scenario-case graph".to_string(),
                location: scenario_case.clone(),
                problem: format!(
                    "module `{}` contains Mermaid scenario-case `{file_name}`, but the file is not declared by `contract.required`",
                    module.module_ref
                ),
                why_it_blocks: "scenario-case graphs must be owned by the node contract"
                    .to_string(),
                fix: format!(
                    "add `{file_name}` to `contract.required` or remove the uncontracted Mermaid file"
                ),
            });
            continue;
        }

        validate_contracted_mermaid_case(
            module,
            &scenario_case,
            file_name,
            known_module_names,
            diagnostics,
        )?;
    }

    Ok(())
}

fn validate_contracted_mermaid_case(
    module: &FlowhubDiscoveredModule,
    scenario_case: &Path,
    file_name: &str,
    known_module_names: &[String],
    diagnostics: &mut Vec<FlowhubDiagnostic>,
) -> Result<(), QianjiError> {
    let source = std::fs::read_to_string(scenario_case).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to read Mermaid scenario-case `{}`: {error}",
            scenario_case.display()
        ))
    })?;
    let fallback_graph_name = scenario_case
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(file_name);
    let declared_graph = declared_graph_contract(module, file_name);
    let annotations = match parse_flowhub_graph_annotations(&source) {
        Ok(annotations) => annotations,
        Err(error) => {
            diagnostics.push(FlowhubDiagnostic {
                title: "Invalid scenario-case contract".to_string(),
                location: scenario_case.to_path_buf(),
                problem: error.to_string(),
                why_it_blocks: "Qianji cannot trust the Mermaid-owned graph contract metadata"
                    .to_string(),
                fix: "repair the `%% qianji.*` annotations so the graph contract is well formed"
                    .to_string(),
            });
            return Ok(());
        }
    };
    let merimind_graph_name =
        resolve_flowhub_graph_name(annotations.as_ref(), declared_graph, fallback_graph_name);
    match parse_mermaid_flowchart(&source, &merimind_graph_name, known_module_names) {
        Ok(flowchart) => {
            validate_parsed_mermaid_case(
                &flowchart,
                &MermaidCaseValidation {
                    module_ref: &module.module_ref,
                    scenario_case,
                    file_name,
                    merimind_graph_name: &merimind_graph_name,
                    declared_graph,
                    annotations: annotations.as_ref(),
                },
                diagnostics,
            );
            Ok(())
        }
        Err(error) => {
            diagnostics.push(FlowhubDiagnostic {
                title: "Invalid scenario-case Mermaid".to_string(),
                location: scenario_case.to_path_buf(),
                problem: error.to_string(),
                why_it_blocks: "Qianji cannot parse the scenario-case graph into nodes and edges"
                    .to_string(),
                fix: "repair the Mermaid flowchart syntax so node ids, labels, and edges are well formed".to_string(),
            });
            Ok(())
        }
    }
}

fn validate_parsed_mermaid_case(
    flowchart: &MermaidFlowchart,
    validation: &MermaidCaseValidation<'_>,
    diagnostics: &mut Vec<FlowhubDiagnostic>,
) {
    let scenario_ir = match compile_flowhub_scenario_ir(
        validation.scenario_case,
        validation.merimind_graph_name,
        flowchart,
        validation.annotations,
        validation.declared_graph,
    ) {
        Ok(contract) => contract,
        Err(error) => {
            diagnostics.push(FlowhubDiagnostic {
                title: "Invalid scenario-case contract".to_string(),
                location: validation.scenario_case.to_path_buf(),
                problem: error.to_string(),
                why_it_blocks: "Qianji cannot trust the compiled scenario-case contract surface"
                    .to_string(),
                fix: "repair the Mermaid annotations or legacy graph contract so the scenario surface compiles".to_string(),
            });
            return;
        }
    };
    let allowed_graph_node_labels = scenario_ir
        .as_ref()
        .map_or_else(BTreeSet::new, FlowhubScenarioIr::allowed_graph_node_labels);
    if let Err(problem) = validate_mermaid_flowchart(flowchart, &allowed_graph_node_labels) {
        diagnostics.push(FlowhubDiagnostic {
            title: "Invalid scenario-case graph".to_string(),
            location: validation.scenario_case.to_path_buf(),
            problem,
            why_it_blocks:
                "Qianji cannot trust the scenario-case graph as a valid modular assembly surface"
                    .to_string(),
            fix: "repair the Mermaid node and edge graph so required module nodes are valid"
                .to_string(),
        });
        return;
    }

    let topology = analyze_mermaid_flowchart_topology(flowchart);
    if let Some(declared_topology) = scenario_ir
        .as_ref()
        .and_then(|graph| graph.declared_topology)
        && topology.topology != declared_topology
    {
        diagnostics.push(FlowhubDiagnostic {
            title: "Invalid scenario-case topology".to_string(),
            location: validation.scenario_case.to_path_buf(),
            problem: format!(
                "module `{}` expects scenario-case `{}` to resolve topology `{}`, but petgraph analysis resolved `{}`",
                validation.module_ref,
                validation.file_name,
                declared_topology.as_str(),
                topology.topology.as_str(),
            ),
            why_it_blocks:
                "Qianji cannot trust the scenario-case graph as a correctly typed Flowhub topology surface"
                    .to_string(),
            fix: format!(
                "repair `{}` so it matches `{}`, or update the owning graph contract to the analyzed graph shape",
                validation.file_name,
                declared_topology.as_str(),
            ),
        });
    }
}

fn declared_graph_contract<'a>(
    module: &'a FlowhubDiscoveredModule,
    file_name: &str,
) -> Option<&'a FlowhubGraphContract> {
    module
        .manifest
        .graph
        .iter()
        .find(|graph| graph.path == file_name)
}
