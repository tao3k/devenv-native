use std::fs;
use std::path::{Path, PathBuf};

use arrow::array::{Array, Float64Array, Int64Array, StringArray};
use arrow::record_batch::RecordBatch;
use serial_test::serial;
use xiuxian_wendao_julia::integration_support::probe_wendaograph_page_index_host_request_with_fixture;
use xiuxian_wendao_julia::validate_wendao_graph_page_index_reasoning_request_schema;

use crate::search::real_repo_precision::{
    MARKDOWN_SSOT_PROOF_ENV, RealRepoGoldQueryKind, RealRepoPrecisionRunOptions,
    RealRepoPrecisionRunStatus, attach_markdown_knowledge_semantic_query_evidence,
    default_real_repo_precision_catalog, evaluate_markdown_knowledge_semantic_gate,
    run_real_repo_precision_harness_with_options,
};

const RUN_WENDAOGRAPH_MARKDOWN_SSOT_PAGE_INDEX_LIVE_PROOF_TEST_ENV: &str =
    "RUN_WENDAOGRAPH_MARKDOWN_SSOT_PAGE_INDEX_LIVE_PROOF_TEST";

#[test]
fn markdown_knowledge_semantic_gate_projects_real_ssot_to_page_index() {
    let catalog = default_real_repo_precision_catalog();
    let entry = &catalog[0];
    let evaluation = evaluate_markdown_knowledge_semantic_gate(
        project_root().join("semantic").as_path(),
        entry.gold_queries.as_slice(),
    )
    .unwrap_or_else(|error| panic!("evaluate Markdown knowledge semantic gate: {error}"))
    .unwrap_or_else(|| panic!("semantic gate should apply to default Markdown gold queries"));

    let receipt = &evaluation.receipt;
    assert_eq!(
        receipt.schema,
        "xiuxian_wendao.real_repo_markdown_knowledge_semantic_gate.v1"
    );
    assert_eq!(
        receipt.linked_query_ids,
        vec![
            "repo-native-semantic-ssot-rfc".to_string(),
            "semantic-object-wendao-query-substrate".to_string(),
            "semantic-decision-repo-native-authority".to_string(),
            "semantic-decision-projections-read-models".to_string(),
            "semantic-invariant-llm-output-not-authority".to_string(),
            "semantic-relation-repo-native-governs-query-substrate".to_string(),
            "semantic-relation-projections-govern-llm-boundary".to_string(),
            "semantic-relation-llm-constrains-projections".to_string(),
        ]
    );
    assert!(
        receipt
            .required_markdown_paths
            .contains(&"docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-rfc.md".to_string())
    );
    assert!(
        receipt
            .required_markdown_paths
            .contains(&"semantic/objects/component/wendao-query-substrate.md".to_string())
    );
    assert!(
        receipt
            .required_markdown_paths
            .contains(&"semantic/objects/decision/semantic-ssot-repo-native-first.md".to_string())
    );
    assert!(receipt.required_markdown_paths.contains(
        &"semantic/objects/decision/semantic-ssot-projections-are-read-models.md".to_string()
    ));
    assert!(
        receipt
            .required_markdown_paths
            .contains(&"semantic/objects/invariant/llm-output-is-not-authority.md".to_string())
    );
    assert_eq!(
        receipt.covered_markdown_paths,
        receipt.required_markdown_paths
    );
    assert_eq!(
        receipt.required_relation_paths,
        vec![
            relation_path(
                "decision.semantic-ssot.repo-native-first",
                "governs",
                "component.wendao.query-substrate",
            ),
            relation_path(
                "decision.semantic-ssot.projections-are-read-models",
                "governs",
                "invariant.llm-output-is-not-authority",
            ),
            relation_path(
                "invariant.llm-output-is-not-authority",
                "constrains",
                "decision.semantic-ssot.projections-are-read-models",
            ),
            relation_path(
                "task.semantic-ssot.object-schema-pilot",
                "validates",
                "invariant.llm-output-is-not-authority",
            ),
        ]
    );
    for relation in &receipt.required_relation_paths {
        assert!(
            receipt.covered_relation_paths.contains(relation),
            "missing relation path {relation:?}"
        );
    }
    assert_eq!(receipt.knowledge_scenarios.len(), 3);
    assert_semantic_scenario_passed(
        receipt,
        "wendao-query-substrate-authority",
        &[
            "component.wendao.query-substrate",
            "decision.semantic-ssot.repo-native-first",
        ],
    );
    assert_semantic_scenario_passed(
        receipt,
        "projection-read-model-authority-boundary",
        &[
            "decision.semantic-ssot.projections-are-read-models",
            "invariant.llm-output-is-not-authority",
        ],
    );
    assert_semantic_scenario_passed(
        receipt,
        "llm-output-authority-validation",
        &[
            "decision.semantic-ssot.projections-are-read-models",
            "invariant.llm-output-is-not-authority",
            "task.semantic-ssot.object-schema-pilot",
        ],
    );
    assert!(
        receipt
            .semantic_object_ids
            .contains(&"component.wendao.query-substrate".to_string())
    );
    assert!(
        receipt
            .semantic_object_ids
            .contains(&"decision.semantic-ssot.repo-native-first".to_string())
    );
    assert!(
        receipt
            .semantic_object_ids
            .contains(&"decision.semantic-ssot.projections-are-read-models".to_string())
    );
    assert!(
        receipt
            .semantic_object_ids
            .contains(&"invariant.llm-output-is-not-authority".to_string())
    );
    assert!(receipt.semantic_scope_object_count >= 5);
    assert!(receipt.semantic_scope_relation_count >= 4);
    assert_eq!(
        receipt.page_index_node_count,
        evaluation.page_index.nodes.num_rows()
    );
    assert_eq!(
        receipt.page_index_edge_count,
        evaluation.page_index.edges.num_rows()
    );
    assert_eq!(
        receipt.page_index_seed_count,
        evaluation.page_index.seeds.num_rows()
    );
    assert!(receipt.page_index_seed_count >= 1);

    validate_wendao_graph_page_index_reasoning_request_schema(
        "page_index_nodes",
        evaluation.page_index.nodes.schema().as_ref(),
    )
    .unwrap_or_else(|error| panic!("validate page_index_nodes schema: {error}"));
    validate_wendao_graph_page_index_reasoning_request_schema(
        "page_index_edges",
        evaluation.page_index.edges.schema().as_ref(),
    )
    .unwrap_or_else(|error| panic!("validate page_index_edges schema: {error}"));
    validate_wendao_graph_page_index_reasoning_request_schema(
        "page_index_seeds",
        evaluation.page_index.seeds.schema().as_ref(),
    )
    .unwrap_or_else(|error| panic!("validate page_index_seeds schema: {error}"));
}

#[test]
fn markdown_knowledge_semantic_gate_marks_missing_query_evidence_failed() {
    let catalog = default_real_repo_precision_catalog();
    let entry = &catalog[0];
    let mut receipt = evaluate_markdown_knowledge_semantic_gate(
        project_root().join("semantic").as_path(),
        entry.gold_queries.as_slice(),
    )
    .unwrap_or_else(|error| panic!("evaluate Markdown knowledge semantic gate: {error}"))
    .unwrap_or_else(|| panic!("semantic gate should apply"))
    .receipt;

    attach_markdown_knowledge_semantic_query_evidence(&mut receipt, &[]);

    let scenario = receipt
        .knowledge_scenarios
        .iter()
        .find(|scenario| scenario.scenario_id == "wendao-query-substrate-authority")
        .unwrap_or_else(|| panic!("missing Wendao query substrate scenario"));
    assert!(!scenario.passed);
    assert_eq!(
        scenario.query_evidence.len(),
        scenario.linked_query_ids.len()
    );
    assert!(
        scenario
            .query_evidence
            .iter()
            .all(|evidence| !evidence.passed
                && evidence.query_kind == "missing"
                && evidence.failure_reason.as_deref() == Some("query receipt missing"))
    );
}

#[test]
fn markdown_ssot_real_repo_harness_records_semantic_gate() -> Result<(), String> {
    if !std::env::var(MARKDOWN_SSOT_PROOF_ENV).is_ok_and(|value| value.trim() == "1") {
        return Ok(());
    }

    let mut options = RealRepoPrecisionRunOptions::from_env();
    options.query_kind_filter = Some(RealRepoGoldQueryKind::LinkGraph);
    let status = run_real_repo_precision_harness_with_options(
        options,
        default_real_repo_precision_catalog(),
    )?;
    let RealRepoPrecisionRunStatus::Completed(receipt) = status else {
        panic!("Markdown SSOT proof should complete");
    };
    assert_eq!(receipt.summary.failed_query_count, 0);
    let Some(repository) = receipt.repositories.first() else {
        panic!("Markdown SSOT proof should emit one repository receipt");
    };
    let gate = repository
        .markdown_knowledge_semantic_gate
        .as_ref()
        .unwrap_or_else(|| panic!("Markdown SSOT proof should emit semantic gate receipt"));
    assert_eq!(gate.linked_query_ids.len(), 8);
    assert_eq!(gate.covered_markdown_paths, gate.required_markdown_paths);
    for relation in &gate.required_relation_paths {
        assert!(gate.covered_relation_paths.contains(relation));
    }
    assert_eq!(gate.knowledge_scenarios.len(), 3);
    assert!(
        gate.knowledge_scenarios
            .iter()
            .all(|scenario| scenario.passed)
    );
    let scenario_query_evidence_count = gate
        .knowledge_scenarios
        .iter()
        .map(|scenario| {
            assert_eq!(
                scenario.query_evidence.len(),
                scenario.linked_query_ids.len()
            );
            for evidence in &scenario.query_evidence {
                assert!(evidence.passed, "query evidence failed: {evidence:?}");
                assert_eq!(evidence.query_kind, "link_graph");
                assert!(evidence.failure_reason.is_none());
                assert!(evidence.observed_top_path.is_some());
                assert!(evidence.observed_path_count > 0);
                assert!(
                    evidence.missing_paths.is_empty(),
                    "query evidence has missing paths: {evidence:?}"
                );
            }
            scenario.query_evidence.len()
        })
        .sum::<usize>();
    assert!(scenario_query_evidence_count >= gate.linked_query_ids.len());
    assert_eq!(gate.page_index_node_count, gate.semantic_scope_object_count);
    assert!(gate.page_index_seed_count > 0);
    eprintln!(
        "markdown_ssot_real_repo_gate_summary queries={} scenario_query_evidence={} semantic_objects={} semantic_relations={} page_index_nodes={} page_index_edges={} page_index_seeds={}",
        gate.linked_query_ids.len(),
        scenario_query_evidence_count,
        gate.semantic_scope_object_count,
        gate.semantic_scope_relation_count,
        gate.page_index_node_count,
        gate.page_index_edge_count,
        gate.page_index_seed_count
    );
    Ok(())
}

fn assert_semantic_scenario_passed(
    receipt: &crate::search::real_repo_precision::types::RealRepoMarkdownKnowledgeSemanticGateReceipt,
    scenario_id: &str,
    required_object_ids: &[&str],
) {
    let scenario = receipt
        .knowledge_scenarios
        .iter()
        .find(|scenario| scenario.scenario_id == scenario_id)
        .unwrap_or_else(|| panic!("missing semantic scenario `{scenario_id}`"));
    assert!(scenario.passed, "semantic scenario `{scenario_id}` failed");
    assert_eq!(
        scenario.required_object_ids,
        required_object_ids
            .iter()
            .map(|object_id| (*object_id).to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(scenario.covered_object_ids, scenario.required_object_ids);
    assert_eq!(
        scenario.covered_relation_paths,
        scenario.required_relation_paths
    );
    assert!(
        scenario.query_evidence.is_empty(),
        "semantic-only gate should not attach query evidence"
    );
}

fn relation_path(
    source: &str,
    kind: &str,
    target: &str,
) -> crate::search::real_repo_precision::types::RealRepoMarkdownKnowledgeSemanticRelationPathReceipt
{
    crate::search::real_repo_precision::types::RealRepoMarkdownKnowledgeSemanticRelationPathReceipt {
        source: source.to_string(),
        kind: kind.to_string(),
        target: target.to_string(),
    }
}

#[test]
#[serial]
fn markdown_ssot_page_index_live_proof_runs_real_wendaograph_when_enabled() {
    if std::env::var_os(RUN_WENDAOGRAPH_MARKDOWN_SSOT_PAGE_INDEX_LIVE_PROOF_TEST_ENV).is_none() {
        eprintln!(
            "skipping Markdown SSOT PageIndex live proof; set {RUN_WENDAOGRAPH_MARKDOWN_SSOT_PAGE_INDEX_LIVE_PROOF_TEST_ENV}=1 and WENDAOGRAPH_PACKAGE_DIR"
        );
        return;
    }

    let catalog = default_real_repo_precision_catalog();
    let entry = &catalog[0];
    let evaluation = evaluate_markdown_knowledge_semantic_gate(
        project_root().join("semantic").as_path(),
        entry.gold_queries.as_slice(),
    )
    .unwrap_or_else(|error| panic!("evaluate Markdown SSOT gate: {error}"))
    .unwrap_or_else(|| panic!("Markdown SSOT gate should apply"));
    let temp = tempfile::tempdir()
        .unwrap_or_else(|error| panic!("create Markdown SSOT PageIndex fixture temp dir: {error}"));
    write_page_index_fixture(temp.path(), &evaluation.page_index);

    let report = probe_wendaograph_page_index_host_request_with_fixture(temp.path(), 2)
        .unwrap_or_else(|error| panic!("run Markdown SSOT PageIndex live proof: {error}"));

    assert_eq!(report.sample_count, 2);
    assert!(report.frontier_rows > 0);
    assert!(report.trace_rows > 0);
    assert!(report.warm_median_ms >= report.warm_min_ms);
    assert!(report.warm_p95_ms >= report.warm_median_ms);
    eprintln!(
        "wendaograph_markdown_ssot_page_index_live_proof_summary sample_count={} semantic_objects={} page_index_nodes={} page_index_edges={} page_index_seeds={} frontier_rows={} trace_rows={} first_ms={:.3} warm_median_ms={:.3} warm_p95_ms={:.3} warm_max_ms={:.3}",
        report.sample_count,
        evaluation.receipt.semantic_scope_object_count,
        evaluation.receipt.page_index_node_count,
        evaluation.receipt.page_index_edge_count,
        evaluation.receipt.page_index_seed_count,
        report.frontier_rows,
        report.trace_rows,
        report.first_ms,
        report.warm_median_ms,
        report.warm_p95_ms,
        report.warm_max_ms
    );
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")))
}

fn write_page_index_fixture(
    fixture_dir: &Path,
    bundle: &crate::link_graph::WendaoGraphPageIndexReasoningRequestBundle,
) {
    write_tsv_file(
        &fixture_dir.join("page_index_nodes.tsv"),
        &[
            "node_id",
            "page_id",
            "parent_id",
            "depth",
            "rank",
            "title",
            "summary",
            "line_start",
            "line_end",
            "token_count",
        ],
        page_index_node_rows(&bundle.nodes),
    );
    write_tsv_file(
        &fixture_dir.join("page_index_edges.tsv"),
        &["source_id", "target_id", "edge_kind", "weight"],
        page_index_edge_rows(&bundle.edges),
    );
    write_tsv_file(
        &fixture_dir.join("page_index_seeds.tsv"),
        &["node_id", "weight", "seed_kind"],
        page_index_seed_rows(&bundle.seeds),
    );
}

fn write_tsv_file(path: &Path, header: &[&str], rows: Vec<Vec<String>>) {
    let mut content = header.join("\t");
    content.push('\n');
    for row in rows {
        let cells = row
            .iter()
            .map(|cell| {
                cell.replace('\t', " ")
                    .replace('\n', " ")
                    .replace('\r', " ")
            })
            .collect::<Vec<_>>();
        content.push_str(&cells.join("\t"));
        content.push('\n');
    }
    fs::write(path, content).unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
}

fn page_index_node_rows(batch: &RecordBatch) -> Vec<Vec<String>> {
    let node_ids = string_column(batch, 0);
    let page_ids = string_column(batch, 1);
    let parent_ids = string_column(batch, 2);
    let depths = int64_column(batch, 3);
    let ranks = int64_column(batch, 4);
    let titles = string_column(batch, 5);
    let summaries = string_column(batch, 6);
    let line_starts = int64_column(batch, 7);
    let line_ends = int64_column(batch, 8);
    let token_counts = int64_column(batch, 9);

    (0..batch.num_rows())
        .map(|index| {
            vec![
                node_ids[index].clone(),
                page_ids[index].clone(),
                parent_ids[index].clone(),
                depths[index].to_string(),
                ranks[index].to_string(),
                titles[index].clone(),
                summaries[index].clone(),
                line_starts[index].to_string(),
                line_ends[index].to_string(),
                token_counts[index].to_string(),
            ]
        })
        .collect()
}

fn page_index_edge_rows(batch: &RecordBatch) -> Vec<Vec<String>> {
    let source_ids = string_column(batch, 0);
    let target_ids = string_column(batch, 1);
    let edge_kinds = string_column(batch, 2);
    let weights = float64_column(batch, 3);
    (0..batch.num_rows())
        .map(|index| {
            vec![
                source_ids[index].clone(),
                target_ids[index].clone(),
                edge_kinds[index].clone(),
                weights[index].to_string(),
            ]
        })
        .collect()
}

fn page_index_seed_rows(batch: &RecordBatch) -> Vec<Vec<String>> {
    let node_ids = string_column(batch, 0);
    let weights = float64_column(batch, 1);
    let seed_kinds = string_column(batch, 2);
    (0..batch.num_rows())
        .map(|index| {
            vec![
                node_ids[index].clone(),
                weights[index].to_string(),
                seed_kinds[index].clone(),
            ]
        })
        .collect()
}

fn string_column(batch: &RecordBatch, index: usize) -> Vec<String> {
    let column = batch
        .column(index)
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap_or_else(|| panic!("column {index} should be string"));
    (0..column.len())
        .map(|row| column.value(row).to_string())
        .collect()
}

fn int64_column(batch: &RecordBatch, index: usize) -> Vec<i64> {
    let column = batch
        .column(index)
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap_or_else(|| panic!("column {index} should be int64"));
    (0..column.len()).map(|row| column.value(row)).collect()
}

fn float64_column(batch: &RecordBatch, index: usize) -> Vec<f64> {
    let column = batch
        .column(index)
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap_or_else(|| panic!("column {index} should be float64"));
    (0..column.len()).map(|row| column.value(row)).collect()
}
