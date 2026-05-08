use std::fs;

use xiuxian_wendao_core::repo_intelligence::{RegisteredRepository, RepositoryRefreshPolicy};

use crate::link_graph::LinkGraphIndex;
use crate::search::real_repo_precision::{
    DOCS_CORPUS_PROOF_ENV, PREWARM_PROOF_ENV, RESIDENT_PROOF_ENV, RealRepoGoldQuery,
    RealRepoGoldQueryKind, RealRepoPrecisionCatalogEntry, RealRepoPrecisionRunOptions,
    RealRepoPrecisionRunStatus, RealRepoPrecisionSyncMode, default_real_repo_precision_catalog,
    evaluate_gold_query_paths, run_real_repo_precision_harness,
    run_real_repo_precision_harness_with_options,
};

#[path = "docs_page_index.rs"]
mod docs_page_index;
#[path = "scenario_matrix.rs"]
mod scenario_matrix;
#[path = "semantic_gate.rs"]
mod semantic_gate;

#[test]
fn default_catalog_uses_managed_repo_contracts() {
    let catalog = default_real_repo_precision_catalog();
    assert_eq!(catalog.len(), 1);
    let entry = &catalog[0];

    assert_eq!(entry.repository.id, "xiuxian-artisan-workshop");
    assert!(entry.repository.path.is_none());
    assert_eq!(
        entry.repository.url.as_deref(),
        Some("https://github.com/tao3k/xiuxian-artisan-workshop.git")
    );
    assert_eq!(entry.repository.refresh, RepositoryRefreshPolicy::Manual);
    assert!(
        entry
            .include_dirs
            .iter()
            .any(|path| path == "packages/rust/crates/xiuxian-wendao")
    );
    assert!(
        entry
            .include_dirs
            .iter()
            .any(|path| path == "packages/rust/crates/xiuxian-wendao/src/link_graph")
    );
    assert!(entry.include_dirs.iter().any(|path| path == "semantic"));
    assert!(
        entry
            .gold_queries
            .iter()
            .any(|query| query.id == "repo-native-semantic-ssot-rfc")
    );
    assert!(entry.gold_queries.iter().any(|query| query.id
        == "semantic-object-wendao-query-substrate"
        && matches!(query.kind, RealRepoGoldQueryKind::LinkGraph)
        && query.must_hit_paths
            == vec!["semantic/objects/component/wendao-query-substrate.md".to_string()]));
    assert_semantic_object_gold_query(
        &entry.gold_queries,
        "semantic-decision-repo-native-authority",
        "semantic/objects/decision/semantic-ssot-repo-native-first.md",
    );
    assert_semantic_object_gold_query(
        &entry.gold_queries,
        "semantic-decision-projections-read-models",
        "semantic/objects/decision/semantic-ssot-projections-are-read-models.md",
    );
    assert_semantic_object_gold_query(
        &entry.gold_queries,
        "semantic-invariant-llm-output-not-authority",
        "semantic/objects/invariant/llm-output-is-not-authority.md",
    );
    assert_semantic_object_gold_query(
        &entry.gold_queries,
        "semantic-relation-repo-native-governs-query-substrate",
        "semantic/objects/decision/semantic-ssot-repo-native-first.md",
    );
    assert_semantic_object_gold_query(
        &entry.gold_queries,
        "semantic-relation-projections-govern-llm-boundary",
        "semantic/objects/decision/semantic-ssot-projections-are-read-models.md",
    );
    assert_semantic_object_gold_query(
        &entry.gold_queries,
        "semantic-relation-llm-constrains-projections",
        "semantic/objects/invariant/llm-output-is-not-authority.md",
    );
    assert_docs_gold_query(
        &entry.gold_queries,
        "docs-documentation-hierarchy-standard",
        "docs/02_dev/standards/DOCUMENTATION_HIERARCHY.md",
    );
    assert_docs_gold_query(
        &entry.gold_queries,
        "docs-documentation-hierarchy-standard-paraphrase",
        "docs/02_dev/standards/DOCUMENTATION_HIERARCHY.md",
    );
    assert_docs_gold_query(
        &entry.gold_queries,
        "docs-wendao-agentic-retrieval",
        "docs/03_features/wendao-agentic-retrieval.md",
    );
    assert_docs_gold_query(
        &entry.gold_queries,
        "docs-wendao-agentic-retrieval-paraphrase",
        "docs/03_features/wendao-agentic-retrieval.md",
    );
    assert_docs_gold_query(
        &entry.gold_queries,
        "docs-memory-architecture",
        "docs/01_core/memory/architecture.md",
    );
    assert_docs_gold_query(
        &entry.gold_queries,
        "docs-llm-routing-guide",
        "docs/99_llm/routing-guide.md",
    );
    assert_docs_gold_query(
        &entry.gold_queries,
        "docs-polyglot-compute-orchestrator-rfc",
        "docs/rfcs/2026-05-04-polyglot-compute-orchestrator-rfc.md",
    );
    assert_docs_gold_query(
        &entry.gold_queries,
        "docs-polyglot-page-index-agent-task",
        "docs/rfcs/2026-05-04-polyglot-compute-orchestrator-rfc.md",
    );
    assert_docs_gold_query(
        &entry.gold_queries,
        "docs-wendao-memory-layer-boundaries-rfc",
        "docs/rfcs/2026-04-05-wendao-memory-layer-boundaries-rfc.md",
    );
    assert_docs_gold_query(
        &entry.gold_queries,
        "docs-wendao-context-snapshot",
        "docs/03_features/wendao-context-snapshot.md",
    );
    assert_docs_gold_query(
        &entry.gold_queries,
        "docs-wendao-context-snapshot-alias",
        "docs/03_features/wendao-context-snapshot.md",
    );
    assert_docs_gold_query(
        &entry.gold_queries,
        "docs-traceability-policy",
        "docs/02_dev/standards/TRACEABILITY_POLICY.md",
    );
    assert_docs_gold_query(
        &entry.gold_queries,
        "docs-root-index-registry",
        "docs/00_vision/ROOT_INDEX.md",
    );
    assert!(
        entry
            .gold_queries
            .iter()
            .filter(|query| query.id.starts_with("docs-")
                && matches!(query.kind, RealRepoGoldQueryKind::LinkGraph))
            .count()
            >= 13
    );
    assert!(
        entry
            .knowledge_scenarios
            .iter()
            .any(|scenario| scenario.id == "known-item-documentation-hierarchy")
    );
    assert!(
        entry
            .knowledge_scenarios
            .iter()
            .any(|scenario| scenario.id == "multi-hop-projection-authority-boundary")
    );
    assert!(
        entry
            .knowledge_scenarios
            .iter()
            .any(|scenario| scenario.id == "negative-llm-output-authority-guard")
    );
    assert!(entry.knowledge_scenarios.len() >= 7);
    assert!(
        entry
            .knowledge_scenarios
            .iter()
            .all(|scenario| !scenario.query_variants.is_empty())
    );
    assert!(entry.gold_queries.iter().any(|query| matches!(
        query.kind,
        RealRepoGoldQueryKind::RepoAst
    ) && query.id
        == "repo-source-materialization-function"));
    assert!(entry.gold_queries.iter().any(|query| matches!(
        query.kind,
        RealRepoGoldQueryKind::RepoAst
    ) && query.id
        == "repo-code-search-query-async-function"
        && query.query == "search_repo_code_outcome_for_query"
        && query.language_filters == vec!["rust".to_string()]));
    assert!(entry.gold_queries.iter().any(|query| matches!(
        query.kind,
        RealRepoGoldQueryKind::RepoAst
    ) && query.id
        == "repo-link-graph-build-with-filters-source"
        && query.query == "build_with_filters"
        && query.must_hit_paths
            == vec![
                "packages/rust/crates/xiuxian-wendao/src/link_graph/index/build/assemble/api.rs"
                    .to_string()
            ]
        && query.language_filters == vec!["rust".to_string()]));
}

fn assert_docs_gold_query(gold_queries: &[RealRepoGoldQuery], query_id: &str, expected_path: &str) {
    assert!(
        gold_queries.iter().any(|query| query.id == query_id
            && matches!(query.kind, RealRepoGoldQueryKind::LinkGraph)
            && query.limit >= 20
            && query
                .must_hit_paths
                .iter()
                .any(|path| path == expected_path)),
        "missing docs gold query `{query_id}` for `{expected_path}`"
    );
}

fn assert_semantic_object_gold_query(
    gold_queries: &[RealRepoGoldQuery],
    query_id: &str,
    expected_path: &str,
) {
    assert!(
        gold_queries.iter().any(|query| query.id == query_id
            && matches!(query.kind, RealRepoGoldQueryKind::LinkGraph)
            && query.must_hit_paths == vec![expected_path.to_string()]
            && query.required_top_path.as_deref() == Some(expected_path)),
        "missing semantic object gold query `{query_id}` for `{expected_path}`"
    );
}

#[test]
fn sync_mode_parser_defaults_to_status() {
    assert_eq!(
        RealRepoPrecisionSyncMode::parse(None),
        RealRepoPrecisionSyncMode::Status
    );
    assert_eq!(
        RealRepoPrecisionSyncMode::parse(Some("unknown")),
        RealRepoPrecisionSyncMode::Status
    );
    assert_eq!(
        RealRepoPrecisionSyncMode::parse(Some("ensure")),
        RealRepoPrecisionSyncMode::Ensure
    );
    assert_eq!(
        RealRepoPrecisionSyncMode::parse(Some("REFRESH")),
        RealRepoPrecisionSyncMode::Refresh
    );
    assert_eq!(RealRepoGoldQueryKind::parse_filter(None), None);
    assert_eq!(
        RealRepoGoldQueryKind::parse_filter(Some("link_graph")),
        Some(RealRepoGoldQueryKind::LinkGraph)
    );
    assert_eq!(
        RealRepoGoldQueryKind::parse_filter(Some("REPO_AST")),
        Some(RealRepoGoldQueryKind::RepoAst)
    );
    assert_eq!(RealRepoGoldQueryKind::parse_filter(Some("all")), None);
}

#[test]
fn evaluator_passes_when_expected_paths_are_returned() {
    let query = gold_query(Some("docs/rfcs/rfc.md"));
    let hits = vec![
        "docs/rfcs/rfc.md".to_string(),
        "packages/rust/crates/xiuxian-wendao/README.md".to_string(),
    ];

    let receipt = evaluate_gold_query_paths(&query, hits);

    assert!(receipt.passed);
    assert_eq!(receipt.query_kind, "link_graph");
    assert!(receipt.missing_paths.is_empty());
    assert_eq!(
        receipt.observed_top_path.as_deref(),
        Some("docs/rfcs/rfc.md")
    );
    assert_eq!(receipt.best_required_path_rank, Some(0));
    assert_eq!(receipt.required_path_recall_at_1_bps, 10_000);
    assert_eq!(receipt.required_path_recall_at_3_bps, 10_000);
    assert_eq!(receipt.required_path_recall_at_5_bps, 10_000);
    assert_eq!(receipt.required_path_recall_at_10_bps, 10_000);
    assert_eq!(receipt.mean_required_path_reciprocal_rank_bps, 10_000);
}

#[test]
fn evaluator_reports_missing_paths_and_top_path_mismatch() {
    let query = gold_query(Some("docs/rfcs/rfc.md"));
    let hits = vec!["docs/other.md".to_string()];

    let receipt = evaluate_gold_query_paths(&query, hits);

    assert!(!receipt.passed);
    assert_eq!(receipt.missing_paths, vec!["docs/rfcs/rfc.md"]);
    assert_eq!(receipt.observed_top_path.as_deref(), Some("docs/other.md"));
    assert_eq!(receipt.best_required_path_rank, None);
    assert_eq!(receipt.required_path_recall_at_1_bps, 0);
    assert_eq!(receipt.required_path_recall_at_3_bps, 0);
    assert_eq!(receipt.required_path_recall_at_5_bps, 0);
    assert_eq!(receipt.required_path_recall_at_10_bps, 0);
    assert_eq!(receipt.mean_required_path_reciprocal_rank_bps, 0);
}

#[test]
fn evaluator_records_late_required_path_rank_quality() {
    let mut query = gold_query(None);
    query.must_hit_paths = vec!["docs/rfcs/rfc.md".to_string()];
    let hits = vec!["docs/other.md".to_string(), "docs/rfcs/rfc.md".to_string()];

    let receipt = evaluate_gold_query_paths(&query, hits);

    assert!(receipt.passed);
    assert_eq!(receipt.best_required_path_rank, Some(1));
    assert_eq!(receipt.required_path_recall_at_1_bps, 0);
    assert_eq!(receipt.required_path_recall_at_3_bps, 10_000);
    assert_eq!(receipt.required_path_recall_at_5_bps, 10_000);
    assert_eq!(receipt.required_path_recall_at_10_bps, 10_000);
    assert_eq!(receipt.mean_required_path_reciprocal_rank_bps, 5_000);
    assert_eq!(
        receipt.required_path_ranks[0].path,
        "docs/rfcs/rfc.md".to_string()
    );
    assert_eq!(receipt.required_path_ranks[0].zero_based_rank, Some(1));
}

#[test]
fn disabled_harness_skips_without_touching_receipt() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let options = RealRepoPrecisionRunOptions {
        enabled: false,
        sync_mode: RealRepoPrecisionSyncMode::Status,
        query_kind_filter: None,
        prewarmed_resident_only: false,
        project_root: temp.path().to_path_buf(),
        receipt_path: temp.path().join("receipt.json"),
        link_graph_cache_path: temp.path().join("link_graph_cache.duckdb"),
    };

    let status = run_real_repo_precision_harness_with_options(options, Vec::new())?;

    assert!(matches!(status, RealRepoPrecisionRunStatus::Skipped { .. }));
    assert!(!temp.path().join("receipt.json").exists());
    Ok(())
}

#[test]
fn status_mode_records_missing_remote_checkout_as_skipped_repository()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let receipt_path = temp.path().join("receipt.json");
    let options = RealRepoPrecisionRunOptions {
        enabled: true,
        sync_mode: RealRepoPrecisionSyncMode::Status,
        query_kind_filter: None,
        prewarmed_resident_only: false,
        project_root: temp.path().to_path_buf(),
        receipt_path: receipt_path.clone(),
        link_graph_cache_path: temp.path().join("link_graph_cache.duckdb"),
    };
    let catalog = vec![RealRepoPrecisionCatalogEntry {
        repository: RegisteredRepository {
            id: "missing-real-repo".to_string(),
            path: None,
            url: Some("https://example.invalid/missing-real-repo.git".to_string()),
            git_ref: None,
            refresh: RepositoryRefreshPolicy::Manual,
            plugins: Vec::new(),
        },
        include_dirs: Vec::new(),
        excluded_dirs: Vec::new(),
        gold_queries: vec![gold_query(None)],
        knowledge_scenarios: Vec::new(),
    }];

    let status = run_real_repo_precision_harness_with_options(options, catalog)?;

    let RealRepoPrecisionRunStatus::Completed(receipt) = status else {
        panic!("enabled status run should write a receipt");
    };
    assert_eq!(receipt.summary.repository_count, 1);
    assert_eq!(receipt.summary.skipped_repository_count, 1);
    assert_eq!(receipt.summary.query_count, 0);
    assert!(receipt_path.exists());
    let payload = fs::read_to_string(receipt_path)?;
    assert!(payload.contains("missing-real-repo"));
    Ok(())
}

#[test]
fn env_backed_harness_entrypoint_is_default_safe() -> Result<(), String> {
    let status = run_real_repo_precision_harness()?;
    match status {
        RealRepoPrecisionRunStatus::Skipped { reason } => {
            assert!(reason.contains("RUN_WENDAO_REAL_REPO_SEARCH_PRECISION_TEST"));
        }
        RealRepoPrecisionRunStatus::Completed(receipt) => {
            assert_eq!(
                receipt.schema,
                "xiuxian_wendao.real_repo_search_precision.v1"
            );
        }
    }
    Ok(())
}

#[cfg(feature = "duckdb")]
#[test]
fn link_graph_cache_metadata_records_miss_then_resident_hit()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_root = temp.path().join("repo");
    fs::create_dir_all(repo_root.join("docs/rfcs"))?;
    fs::write(
        repo_root.join("docs/rfcs/rfc.md"),
        "# Repo Native Semantic SSOT\n\nThis RFC defines semantic SSOT search precision.\n",
    )?;
    let git_status = std::process::Command::new("git")
        .arg("init")
        .arg("--quiet")
        .current_dir(&repo_root)
        .status()?;
    assert!(git_status.success(), "git init failed");

    let receipt_path = temp.path().join("receipt.json");
    let link_graph_cache_path = temp.path().join("link_graph_cache.duckdb");
    let options = RealRepoPrecisionRunOptions {
        enabled: true,
        sync_mode: RealRepoPrecisionSyncMode::Status,
        query_kind_filter: Some(RealRepoGoldQueryKind::LinkGraph),
        prewarmed_resident_only: false,
        project_root: temp.path().to_path_buf(),
        receipt_path,
        link_graph_cache_path: link_graph_cache_path.clone(),
    };
    let catalog = vec![RealRepoPrecisionCatalogEntry {
        repository: RegisteredRepository {
            id: "local-knowledge-repo".to_string(),
            path: Some(repo_root.clone()),
            url: None,
            git_ref: None,
            refresh: RepositoryRefreshPolicy::Manual,
            plugins: Vec::new(),
        },
        include_dirs: vec!["docs".to_string()],
        excluded_dirs: Vec::new(),
        gold_queries: vec![gold_query(Some("docs/rfcs/rfc.md"))],
        knowledge_scenarios: Vec::new(),
    }];

    let first = run_real_repo_precision_harness_with_options(options.clone(), catalog.clone())?;
    let RealRepoPrecisionRunStatus::Completed(first_receipt) = first else {
        panic!("enabled run should complete");
    };
    let first_repo = &first_receipt.repositories[0];
    assert_eq!(first_receipt.query_kind_filter, "link_graph");
    let first_corpus = first_repo
        .link_graph_corpus
        .as_ref()
        .unwrap_or_else(|| panic!("LinkGraph corpus receipt should be present"));
    assert_eq!(first_corpus.document_count, 1);
    assert_eq!(first_corpus.markdown_document_count, 1);
    assert!(first_corpus.total_word_count > 0);
    assert_eq!(first_receipt.summary.indexed_document_count, 1);
    assert_eq!(first_receipt.summary.indexed_markdown_document_count, 1);
    assert_eq!(
        first_receipt.summary.indexed_total_word_count,
        first_corpus.total_word_count
    );
    assert!(first_corpus.path_prefix_counts.iter().any(|prefix| {
        prefix.prefix == "docs/rfcs"
            && prefix.document_count == 1
            && prefix.word_count == first_corpus.total_word_count
    }));
    assert_eq!(
        first_repo.link_graph_cache_backend.as_deref(),
        Some("duckdb")
    );
    assert_eq!(first_repo.link_graph_cache_status.as_deref(), Some("miss"));
    assert_eq!(
        first_repo.link_graph_cache_miss_reason.as_deref(),
        Some("key_not_found")
    );
    assert_eq!(first_receipt.summary.passed_query_count, 1);

    let second = run_real_repo_precision_harness_with_options(options.clone(), catalog.clone())?;
    let RealRepoPrecisionRunStatus::Completed(second_receipt) = second else {
        panic!("enabled run should complete");
    };
    let second_repo = &second_receipt.repositories[0];
    assert_eq!(
        second_repo.link_graph_cache_backend.as_deref(),
        Some("resident")
    );
    assert_eq!(second_repo.link_graph_cache_status.as_deref(), Some("hit"));
    assert_eq!(second_repo.link_graph_cache_miss_reason, None);
    assert_eq!(second_receipt.summary.passed_query_count, 1);

    let prewarmed_options = RealRepoPrecisionRunOptions {
        prewarmed_resident_only: true,
        ..options
    };
    let third = run_real_repo_precision_harness_with_options(prewarmed_options, catalog)?;
    let RealRepoPrecisionRunStatus::Completed(third_receipt) = third else {
        panic!("enabled run should complete");
    };
    let third_repo = &third_receipt.repositories[0];
    assert_eq!(
        third_repo.link_graph_cache_backend.as_deref(),
        Some("resident-prewarmed")
    );
    assert_eq!(third_repo.link_graph_cache_status.as_deref(), Some("hit"));
    assert_eq!(third_receipt.summary.passed_query_count, 1);

    let removed = LinkGraphIndex::invalidate_resident_local_cache_path(
        repo_root.as_path(),
        &["docs".to_string()],
        &[],
        link_graph_cache_path.as_path(),
    )?;
    assert!(removed);
    let missed = LinkGraphIndex::lookup_prewarmed_resident_local_cache_path_with_meta(
        repo_root.as_path(),
        &["docs".to_string()],
        &[],
        link_graph_cache_path.as_path(),
    );
    assert!(missed.is_err());
    Ok(())
}

#[test]
fn docs_corpus_real_repo_harness_records_document_volume_and_precision() -> Result<(), String> {
    if !std::env::var(DOCS_CORPUS_PROOF_ENV).is_ok_and(|value| value.trim() == "1") {
        return Ok(());
    }

    let mut options = RealRepoPrecisionRunOptions::from_env();
    options.query_kind_filter = Some(RealRepoGoldQueryKind::LinkGraph);
    let status = run_real_repo_precision_harness_with_options(
        options,
        default_real_repo_precision_catalog(),
    )?;
    let RealRepoPrecisionRunStatus::Completed(receipt) = status else {
        panic!("docs corpus proof should complete");
    };
    assert_eq!(receipt.summary.failed_query_count, 0);
    assert_eq!(receipt.summary.failed_knowledge_scenario_count, 0);
    assert!(receipt.summary.knowledge_scenario_count >= 7);
    assert!(receipt.summary.query_count >= 23);
    assert!(receipt.summary.indexed_document_count >= 80);
    assert!(receipt.summary.indexed_markdown_document_count >= 75);
    assert!(receipt.summary.indexed_total_word_count > 20_000);

    let Some(repository) = receipt.repositories.first() else {
        panic!("docs corpus proof should emit one repository receipt");
    };
    assert!(repository.knowledge_scenarios.len() >= 7);
    assert!(
        repository
            .knowledge_scenarios
            .iter()
            .all(|scenario| scenario.passed)
    );
    assert!(
        repository
            .knowledge_scenarios
            .iter()
            .all(|scenario| scenario.reasoning_tree.passed)
    );
    assert!(
        repository
            .knowledge_scenarios
            .iter()
            .all(|scenario| scenario.reasoning_tree.anchor_count > 0)
    );
    assert!(
        repository
            .knowledge_scenarios
            .iter()
            .all(|scenario| scenario.reasoning_tree.source_step_count
                == scenario.covered_required_path_count)
    );
    assert!(
        repository
            .knowledge_scenarios
            .iter()
            .all(|scenario| scenario.required_path_recall_at_10_bps == 10_000)
    );
    assert!(
        repository
            .knowledge_scenarios
            .iter()
            .all(|scenario| scenario.mean_required_path_reciprocal_rank_bps > 0)
    );
    let corpus = repository
        .link_graph_corpus
        .as_ref()
        .unwrap_or_else(|| panic!("docs corpus proof should emit corpus stats"));
    for required_prefix in [
        "docs/00_vision",
        "docs/01_core",
        "docs/02_dev",
        "docs/03_features",
        "docs/04_chronicles",
        "docs/99_llm",
        "docs/rfcs",
        "semantic",
    ] {
        assert!(
            corpus
                .path_prefix_counts
                .iter()
                .any(|prefix| prefix.prefix == required_prefix && prefix.document_count > 0),
            "missing indexed corpus prefix `{required_prefix}`"
        );
    }
    let docs_query_count = repository
        .query_receipts
        .iter()
        .filter(|query| query.query_id.starts_with("docs-"))
        .count();
    assert!(docs_query_count >= 13);
    let query_variant_count = repository
        .knowledge_scenarios
        .iter()
        .map(|scenario| scenario.query_variant_count)
        .sum::<usize>();
    let failed_query_variant_count = repository
        .knowledge_scenarios
        .iter()
        .map(|scenario| scenario.failed_query_variant_count)
        .sum::<usize>();
    assert!(query_variant_count >= 15);
    assert_eq!(failed_query_variant_count, 0);
    assert!(
        repository
            .query_receipts
            .iter()
            .filter(|query| query.query_id.starts_with("docs-"))
            .all(|query| query.passed)
    );
    assert!(
        repository
            .query_receipts
            .iter()
            .filter(|query| query.query_id.starts_with("docs-"))
            .all(|query| query.mean_required_path_reciprocal_rank_bps > 0)
    );
    assert!(
        repository
            .knowledge_scenarios
            .iter()
            .flat_map(|scenario| scenario.query_variants.iter())
            .all(|variant| variant
                .query_evidence
                .mean_required_path_reciprocal_rank_bps
                > 0)
    );
    let late_docs_query_count = repository
        .query_receipts
        .iter()
        .filter(|query| {
            query.query_id.starts_with("docs-") && query.required_path_recall_at_10_bps < 10_000
        })
        .count();
    let reasoning_tree_step_count = repository
        .knowledge_scenarios
        .iter()
        .map(|scenario| scenario.reasoning_tree.disclosure_step_count)
        .sum::<usize>();
    eprintln!(
        "docs_corpus_real_repo_summary queries={} docs_queries={} knowledge_scenarios={} query_variants={} reasoning_tree_steps={} documents={} markdown_documents={} org_documents={} words={} min_scenario_recall_at_10_bps={} late_docs_query_count={} cache_backend={:?} cache_status={:?} total_ms={}",
        receipt.summary.query_count,
        docs_query_count,
        receipt.summary.knowledge_scenario_count,
        query_variant_count,
        reasoning_tree_step_count,
        corpus.document_count,
        corpus.markdown_document_count,
        corpus.org_document_count,
        corpus.total_word_count,
        repository
            .knowledge_scenarios
            .iter()
            .map(|scenario| scenario.required_path_recall_at_10_bps)
            .min()
            .unwrap_or(10_000),
        late_docs_query_count,
        repository.link_graph_cache_backend,
        repository.link_graph_cache_status,
        repository.total_ms
    );
    Ok(())
}

#[test]
fn resident_real_repo_harness_reuses_loaded_link_graph_index() -> Result<(), String> {
    if !std::env::var(RESIDENT_PROOF_ENV).is_ok_and(|value| value.trim() == "1") {
        return Ok(());
    }

    let options = RealRepoPrecisionRunOptions::from_env();
    let first = run_real_repo_precision_harness_with_options(
        options.clone(),
        default_real_repo_precision_catalog(),
    )?;
    let RealRepoPrecisionRunStatus::Completed(first_receipt) = first else {
        panic!("resident proof should complete the first run");
    };
    assert_eq!(first_receipt.summary.failed_query_count, 0);

    let second = run_real_repo_precision_harness_with_options(
        options,
        default_real_repo_precision_catalog(),
    )?;
    let RealRepoPrecisionRunStatus::Completed(second_receipt) = second else {
        panic!("resident proof should complete the second run");
    };
    assert_eq!(second_receipt.summary.failed_query_count, 0);
    let Some(repository) = second_receipt.repositories.first() else {
        panic!("resident proof should emit one repository receipt");
    };
    assert_eq!(
        repository.link_graph_cache_backend.as_deref(),
        Some("resident")
    );
    assert_eq!(repository.link_graph_cache_status.as_deref(), Some("hit"));
    Ok(())
}

#[test]
fn prewarmed_real_repo_harness_uses_loaded_link_graph_index_without_revalidation()
-> Result<(), String> {
    if !std::env::var(PREWARM_PROOF_ENV).is_ok_and(|value| value.trim() == "1") {
        return Ok(());
    }

    let mut prewarm_options = RealRepoPrecisionRunOptions::from_env();
    prewarm_options.query_kind_filter = Some(RealRepoGoldQueryKind::LinkGraph);
    prewarm_options.prewarmed_resident_only = false;
    let first = run_real_repo_precision_harness_with_options(
        prewarm_options.clone(),
        default_real_repo_precision_catalog(),
    )?;
    let RealRepoPrecisionRunStatus::Completed(first_receipt) = first else {
        panic!("prewarm proof should complete the validating run");
    };
    assert_eq!(first_receipt.summary.failed_query_count, 0);

    let mut request_options = prewarm_options;
    request_options.prewarmed_resident_only = true;
    let second = run_real_repo_precision_harness_with_options(
        request_options,
        default_real_repo_precision_catalog(),
    )?;
    let RealRepoPrecisionRunStatus::Completed(second_receipt) = second else {
        panic!("prewarm proof should complete the request-only run");
    };
    assert_eq!(second_receipt.summary.failed_query_count, 0);
    let Some(repository) = second_receipt.repositories.first() else {
        panic!("prewarm proof should emit one repository receipt");
    };
    assert_eq!(
        repository.link_graph_cache_backend.as_deref(),
        Some("resident-prewarmed")
    );
    assert_eq!(repository.link_graph_cache_status.as_deref(), Some("hit"));
    Ok(())
}

fn gold_query(required_top_path: Option<&str>) -> RealRepoGoldQuery {
    RealRepoGoldQuery {
        id: "gold".to_string(),
        kind: RealRepoGoldQueryKind::LinkGraph,
        query: "semantic SSOT".to_string(),
        limit: 5,
        must_hit_paths: vec!["docs/rfcs/rfc.md".to_string()],
        required_top_path: required_top_path.map(str::to_string),
        language_filters: Vec::new(),
    }
}
