//! Test-structure policy gate for xiuxian-wendao.

use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use xiuxian_testing::{
    CollectionContext, ContractFinding, FindingSeverity, ModularityRulePack, RulePack,
    assert_crate_test_policy_with_workspace_config,
};

#[cfg(not(feature = "performance"))]
#[path = "integration/support/mod.rs"]
mod support;

#[cfg(not(feature = "performance"))]
#[path = "integration/coactivation_multihop_diffusion.rs"]
mod coactivation_multihop_diffusion;

#[cfg(not(feature = "performance"))]
#[path = "integration/coactivation_weighted_propagation.rs"]
mod coactivation_weighted_propagation;

#[cfg(all(not(feature = "performance"), feature = "vector-store"))]
#[path = "integration/planned_search_semantic_ignition.rs"]
mod planned_search_semantic_ignition;

#[cfg(not(feature = "performance"))]
#[path = "integration/ppr_weight_precision.rs"]
mod ppr_weight_precision;

#[cfg(all(not(feature = "performance"), feature = "vector-store"))]
#[path = "integration/quantum_fusion_openai_ignition.rs"]
mod quantum_fusion_openai_ignition;

#[cfg(all(not(feature = "performance"), feature = "vector-store"))]
#[path = "integration/quantum_fusion_saliency_budget.rs"]
mod quantum_fusion_saliency_budget;

#[cfg(all(not(feature = "performance"), feature = "vector-store"))]
#[path = "integration/quantum_fusion_saliency_window.rs"]
mod quantum_fusion_saliency_window;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_doc_coverage.rs"]
mod repo_doc_coverage;

#[cfg(not(feature = "performance"))]
#[path = "integration/docs_markdown_documents.rs"]
mod docs_markdown_documents;

#[cfg(not(feature = "performance"))]
#[path = "integration/docs_search.rs"]
mod docs_search;

#[cfg(not(feature = "performance"))]
#[path = "integration/docs_retrieval.rs"]
mod docs_retrieval;

#[cfg(not(feature = "performance"))]
#[path = "integration/docs_retrieval_context.rs"]
mod docs_retrieval_context;

#[cfg(not(feature = "performance"))]
#[path = "integration/docs_retrieval_hit.rs"]
mod docs_retrieval_hit;

#[cfg(not(feature = "performance"))]
#[path = "integration/docs_planner_item.rs"]
mod docs_planner_item;

#[cfg(not(feature = "performance"))]
#[path = "integration/docs_planner_queue.rs"]
mod docs_planner_queue;

#[cfg(not(feature = "performance"))]
#[path = "integration/docs_planner_rank.rs"]
mod docs_planner_rank;

#[cfg(not(feature = "performance"))]
#[path = "integration/docs_planner_search.rs"]
mod docs_planner_search;

#[cfg(not(feature = "performance"))]
#[path = "integration/docs_planner_workset.rs"]
mod docs_planner_workset;

#[cfg(not(feature = "performance"))]
#[path = "integration/docs_navigation_search.rs"]
mod docs_navigation_search;

#[cfg(not(feature = "performance"))]
#[path = "integration/docs_projected_gap_report.rs"]
mod docs_projected_gap_report;

#[cfg(not(feature = "performance"))]
#[path = "integration/docs_navigation.rs"]
mod docs_navigation;

#[cfg(not(feature = "performance"))]
#[path = "integration/docs_family_search.rs"]
mod docs_family_search;

#[cfg(not(feature = "performance"))]
#[path = "integration/docs_family_context.rs"]
mod docs_family_context;

#[cfg(not(feature = "performance"))]
#[path = "integration/docs_family_cluster.rs"]
mod docs_family_cluster;

#[cfg(not(feature = "performance"))]
#[path = "integration/docs_page.rs"]
mod docs_page;

#[cfg(not(feature = "performance"))]
#[path = "integration/docs_page_index_tree.rs"]
mod docs_page_index_tree;

#[cfg(not(feature = "performance"))]
#[path = "integration/docs_page_index_documents.rs"]
mod docs_page_index_documents;

#[cfg(not(feature = "performance"))]
#[path = "integration/docs_page_index_trees.rs"]
mod docs_page_index_trees;

#[cfg(not(feature = "performance"))]
#[path = "integration/docs_page_index_tree_search.rs"]
mod docs_page_index_tree_search;

#[cfg(not(feature = "performance"))]
#[path = "integration/docs_page_index_node.rs"]
mod docs_page_index_node;

#[cfg(not(feature = "performance"))]
#[path = "integration/docs_tool_service.rs"]
mod docs_tool_service;

#[cfg(not(feature = "performance"))]
#[path = "integration/dependency_indexer_pyproject.rs"]
mod dependency_indexer_pyproject;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_example_search.rs"]
mod repo_example_search;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_projected_gap_report.rs"]
mod repo_projected_gap_report;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_intelligence_registry.rs"]
mod repo_intelligence_registry;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_module_search.rs"]
mod repo_module_search;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_overview.rs"]
mod repo_overview;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_projected_page.rs"]
mod repo_projected_page;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_projected_page_family_cluster.rs"]
mod repo_projected_page_family_cluster;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_projected_page_family_context.rs"]
mod repo_projected_page_family_context;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_projected_page_family_search.rs"]
mod repo_projected_page_family_search;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_projected_page_index_documents.rs"]
mod repo_projected_page_index_documents;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_projected_page_index_node.rs"]
mod repo_projected_page_index_node;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_projected_page_index_tree.rs"]
mod repo_projected_page_index_tree;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_projected_page_index_tree_search.rs"]
mod repo_projected_page_index_tree_search;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_projected_page_index_trees.rs"]
mod repo_projected_page_index_trees;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_projected_page_navigation.rs"]
mod repo_projected_page_navigation;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_projected_page_navigation_search.rs"]
mod repo_projected_page_navigation_search;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_projected_page_search.rs"]
mod repo_projected_page_search;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_projected_pages.rs"]
mod repo_projected_pages;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_projected_retrieval.rs"]
mod repo_projected_retrieval;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_projected_retrieval_context.rs"]
mod repo_projected_retrieval_context;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_projected_retrieval_hit.rs"]
mod repo_projected_retrieval_hit;

#[cfg(not(feature = "performance"))]
#[path = "unit/link_graph_agentic/mod.rs"]
mod link_graph_agentic;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_projection_inputs.rs"]
mod repo_projection_inputs;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_relations.rs"]
mod repo_relations;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_symbol_search.rs"]
mod repo_symbol_search;

#[cfg(not(feature = "performance"))]
#[path = "integration/repo_sync.rs"]
mod repo_sync;

#[cfg(not(feature = "performance"))]
#[path = "integration/scenarios.rs"]
mod scenarios;

#[cfg(not(feature = "performance"))]
#[path = "integration/studio_search_index_api.rs"]
mod studio_search_index_api;

#[cfg(not(feature = "performance"))]
#[path = "integration/pybindings_feature_smoke.rs"]
mod pybindings_feature_smoke;

#[cfg(feature = "performance")]
#[path = "performance/mod.rs"]
mod performance;

#[cfg(feature = "performance-stress")]
#[path = "performance/stress/mod.rs"]
mod performance_stress;

#[test]
fn enforce_crate_test_policy_gate() {
    assert_crate_test_policy_with_workspace_config(Path::new(env!("CARGO_MANIFEST_DIR")));
}

#[test]
fn enforce_modularity_contract_gate() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let findings = collect_modularity_findings(crate_root);
    let blocking_findings = findings
        .iter()
        .filter(|finding| finding.severity >= FindingSeverity::Error)
        .collect::<Vec<_>>();

    assert!(
        blocking_findings.is_empty(),
        "{}",
        format_modularity_gate_report(&findings, &blocking_findings)
    );
}

fn collect_modularity_findings(crate_root: &Path) -> Vec<ContractFinding> {
    let Some(crate_name) = crate_root.file_name().and_then(|value| value.to_str()) else {
        panic!("failed to derive crate name from {}", crate_root.display());
    };
    let context = CollectionContext {
        suite_id: "xiuxian-testing-gate".to_string(),
        crate_name: Some(crate_name.to_string()),
        workspace_root: Some(resolve_workspace_root(crate_root)),
        labels: BTreeMap::new(),
    };
    let pack = ModularityRulePack;
    let artifacts = pack
        .collect(&context)
        .unwrap_or_else(|error| panic!("failed to collect modularity artifacts: {error}"));
    pack.evaluate(&artifacts)
        .unwrap_or_else(|error| panic!("failed to evaluate modularity artifacts: {error}"))
}

fn resolve_workspace_root(crate_root: &Path) -> PathBuf {
    crate_root
        .ancestors()
        .find_map(|candidate| {
            let manifest_path = candidate.join("Cargo.toml");
            let content = fs::read_to_string(manifest_path).ok()?;
            if content.contains("[workspace]") {
                return Some(candidate.to_path_buf());
            }
            None
        })
        .unwrap_or_else(|| {
            panic!(
                "failed to resolve workspace root from crate root {}",
                crate_root.display()
            )
        })
}

fn format_modularity_gate_report(
    findings: &[ContractFinding],
    blocking_findings: &[&ContractFinding],
) -> String {
    let mut output = String::new();
    output.push_str("modularity gate failed with blocking findings (severity >= Error):\n");

    for finding in blocking_findings {
        let _ = writeln!(
            output,
            "- [{}] {} :: {}:{}",
            finding.rule_id,
            finding.summary,
            finding_path(finding),
            finding_locator(finding)
        );
    }

    let warning_count = findings
        .iter()
        .filter(|finding| finding.severity == FindingSeverity::Warning)
        .count();
    if warning_count > 0 {
        let _ = writeln!(output, "non-blocking warnings: {warning_count}");
    }

    output
}

fn finding_path(finding: &ContractFinding) -> String {
    if let Some(path) = finding
        .evidence
        .iter()
        .find_map(|evidence| evidence.path.as_ref())
    {
        return path.display().to_string();
    }
    finding
        .labels
        .get("path")
        .cloned()
        .unwrap_or_else(|| "<unknown-path>".to_string())
}

fn finding_locator(finding: &ContractFinding) -> String {
    finding
        .evidence
        .iter()
        .find_map(|evidence| evidence.locator.as_deref())
        .unwrap_or("<unknown-locator>")
        .to_string()
}
