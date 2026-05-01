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
        .filter(|finding| is_blocking_modularity_finding(crate_root, finding))
        .collect::<Vec<_>>();

    assert!(
        blocking_findings.is_empty(),
        "{}",
        format_modularity_gate_report(crate_root, &findings, &blocking_findings)
    );
}

#[test]
fn enforce_no_new_relative_ancestor_visibility_gate() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let findings = collect_relative_ancestor_visibility_findings(crate_root);
    let blocking_findings = findings
        .iter()
        .filter(|finding| !is_legacy_relative_visibility_finding(finding))
        .collect::<Vec<_>>();

    assert!(
        blocking_findings.is_empty(),
        "{}",
        format_relative_visibility_gate_report(&blocking_findings)
    );
}

const LEGACY_MOD_R006_FILE_BLOAT_BASELINE: &[&str] = &[
    "src/analyzers/projection/gap_report.rs",
    "src/analyzers/service/analysis.rs",
    "src/analyzers/service/projection/docs_tool/contracts.rs",
    "src/analyzers/service/projection/index_tree.rs",
    "src/bin/wendao/execute/audit.rs",
    "src/bin/wendao/execute/gateway/command.rs",
    "src/duckdb/engine.rs",
    "src/enhancer/markdown_config/links.rs",
    "src/gateway/studio/pathing.rs",
    "src/gateway/studio/router/handlers/graph/flight.rs",
    "src/gateway/studio/router/handlers/graph/topology_flight.rs",
    "src/gateway/studio/router/handlers/repo/analysis/index_status_flight/diagnostics.rs",
    "src/gateway/studio/router/state/lifecycle.rs",
    "src/gateway/studio/router/state/search.rs",
    "src/gateway/studio/router/state/ui.rs",
    "src/gateway/studio/search/handlers/flight/repo_search.rs",
    "src/gateway/studio/search/handlers/knowledge/intent/flight.rs",
    "src/gateway/studio/types/search_index/diagnostics.rs",
    "src/gateway/studio/types/search_index/status.rs",
    "src/graph/query/tool_relevance.rs",
    "src/link_graph/index/build/cache/arrow_snapshot.rs",
    "src/link_graph/index/build/cache/duckdb.rs",
    "src/link_graph/index/search/plan/payload/policy.rs",
    "src/link_graph/index/search/plan/payload/quantum/rerank.rs",
    "src/parsers/docs_governance/api.rs",
    "src/pybindings/link_graph_py/engine/refresh/plan_apply.rs",
    "src/query_core/service.rs",
    "src/repo_index/state/coordinator/runtime/incremental.rs",
    "src/repo_index/state/task/adaptive.rs",
    "src/search/local_symbol/build/plan.rs",
    "src/search/perf_support.rs",
    "src/search/project_fingerprint.rs",
    "src/search/queries/graphql/document.rs",
    "src/search/queries/sql/registration/table.rs",
    "src/search/repo_content_chunk/build/write.rs",
    "src/search/repo_content_chunk/query/lookup/helpers.rs",
    "src/search/repo_search/ast.rs",
    "src/search/repo_search/batch.rs",
    "src/search/repo_search/search.rs",
    "src/search/service/core/repeat_work.rs",
    "src/search/service/helpers/status.rs",
    "src/skill_runtime/zhixing/resources/discovery.rs",
    "src/zhenfa_router/http.rs",
    "src/zhenfa_router/native/semantic_check/episteme.rs",
    "src/zhenfa_router/native/semantic_check/report.rs",
];

const LEGACY_RELATIVE_ANCESTOR_VISIBILITY_BASELINE: &[&str] = &[
    "src/search/queries/flightsql/discovery/catalogs.rs::pub(in super::super) const WENDAO_FLIGHTSQL_CATALOG_NAME: &str = \"wendao\";",
    "src/search/queries/flightsql/discovery/catalogs.rs::pub(in super::super) fn build_catalogs_flight_info_schema(query: CommandGetCatalogs) -> SchemaRef {",
    "src/search/queries/flightsql/discovery/catalogs.rs::pub(in super::super) fn build_catalogs_batch(",
    "src/search/queries/flightsql/discovery/schemas.rs::pub(in super::super) fn build_schemas_flight_info_schema(query: CommandGetDbSchemas) -> SchemaRef {",
    "src/search/queries/flightsql/discovery/schemas.rs::pub(in super::super) fn build_schemas_batch(",
    "src/search/queries/flightsql/discovery/schemas.rs::pub(in super::super) fn flightsql_schema_name(scope: &str) -> &str {",
    "src/search/queries/flightsql/discovery/tables.rs::pub(in super::super) fn build_tables_flight_info_schema(query: CommandGetTables) -> SchemaRef {",
    "src/search/queries/flightsql/discovery/tables.rs::pub(in super::super) fn build_tables_batch(",
    "src/search/queries/flightsql/discovery/tables.rs::pub(in super::super) fn flightsql_table_type(sql_object_kind: &str) -> &str {",
];

#[derive(Debug)]
struct RelativeVisibilityFinding {
    relative_path: String,
    line_number: usize,
    declaration: String,
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

fn collect_relative_ancestor_visibility_findings(
    crate_root: &Path,
) -> Vec<RelativeVisibilityFinding> {
    let source_root = crate_root.join("src");
    let mut files = Vec::new();
    collect_rust_source_files(source_root.as_path(), &mut files)
        .unwrap_or_else(|error| panic!("failed to collect Rust source files: {error}"));
    let mut findings = Vec::new();
    for path in files {
        let content = fs::read_to_string(path.as_path())
            .unwrap_or_else(|error| panic!("failed to read `{}`: {error}", path.display()));
        let relative_path = path.strip_prefix(crate_root).map_or_else(
            |_| path.display().to_string(),
            |relative| relative.display().to_string(),
        );
        findings.extend(
            content
                .lines()
                .enumerate()
                .filter_map(|(line_index, line)| {
                    let declaration = line.trim();
                    declaration
                        .contains("pub(in super::")
                        .then(|| RelativeVisibilityFinding {
                            relative_path: relative_path.clone(),
                            line_number: line_index + 1,
                            declaration: declaration.to_string(),
                        })
                }),
        );
    }
    findings
}

fn collect_rust_source_files(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rust_source_files(path.as_path(), files)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn is_legacy_relative_visibility_finding(finding: &RelativeVisibilityFinding) -> bool {
    let key = relative_visibility_key(finding);
    LEGACY_RELATIVE_ANCESTOR_VISIBILITY_BASELINE
        .iter()
        .any(|baseline| key == *baseline)
}

fn relative_visibility_key(finding: &RelativeVisibilityFinding) -> String {
    format!("{}::{}", finding.relative_path, finding.declaration)
}

fn format_relative_visibility_gate_report(findings: &[&RelativeVisibilityFinding]) -> String {
    let mut output = String::from(
        "relative ancestor visibility gate failed with new `pub(in super::...)` declarations:\n",
    );
    for finding in findings {
        let _ = writeln!(
            output,
            "- {}:{} :: {}",
            finding.relative_path, finding.line_number, finding.declaration
        );
    }
    output
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

fn is_blocking_modularity_finding(crate_root: &Path, finding: &ContractFinding) -> bool {
    if finding.severity >= FindingSeverity::Error {
        return true;
    }
    if finding.rule_id != "MOD-R006" {
        return false;
    }
    let path = finding_relative_path(crate_root, finding);
    !LEGACY_MOD_R006_FILE_BLOAT_BASELINE
        .iter()
        .any(|baseline| path == *baseline)
}

fn format_modularity_gate_report(
    crate_root: &Path,
    findings: &[ContractFinding],
    blocking_findings: &[&ContractFinding],
) -> String {
    let mut output = String::new();
    output.push_str(
        "modularity gate failed with blocking findings (severity >= Error or new MOD-R006):\n",
    );

    for finding in blocking_findings {
        let _ = writeln!(
            output,
            "- [{}] {} :: {}:{}",
            finding.rule_id,
            finding.summary,
            finding_relative_path(crate_root, finding),
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
    let legacy_file_bloat_count = findings
        .iter()
        .filter(|finding| {
            finding.rule_id == "MOD-R006"
                && LEGACY_MOD_R006_FILE_BLOAT_BASELINE
                    .iter()
                    .any(|baseline| finding_relative_path(crate_root, finding) == *baseline)
        })
        .count();
    if legacy_file_bloat_count > 0 {
        let _ = writeln!(
            output,
            "legacy MOD-R006 baseline entries: {legacy_file_bloat_count}"
        );
    }

    output
}

fn finding_relative_path(crate_root: &Path, finding: &ContractFinding) -> String {
    let path = finding_path(finding);
    let path = Path::new(path.as_str());
    path.strip_prefix(crate_root).map_or_else(
        |_| path.display().to_string(),
        |path| path.display().to_string(),
    )
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
