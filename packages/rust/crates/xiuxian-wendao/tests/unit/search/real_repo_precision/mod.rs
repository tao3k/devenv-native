use std::fs;
use std::path::PathBuf;

use xiuxian_wendao_core::repo_intelligence::{RegisteredRepository, RepositoryRefreshPolicy};

#[cfg(feature = "duckdb")]
use crate::link_graph::LinkGraphIndex;
use crate::search::real_repo_precision::{
    DOCS_CORPUS_PROOF_ENV, PREWARM_PROOF_ENV, RESIDENT_PROOF_ENV, RealRepoGoldQuery,
    RealRepoGoldQueryKind, RealRepoPrecisionCatalogEntry, RealRepoPrecisionRunOptions,
    RealRepoPrecisionRunStatus, RealRepoPrecisionSyncMode, default_real_repo_precision_catalog,
    evaluate_gold_query_paths, run_real_repo_precision_harness,
    run_real_repo_precision_harness_with_options,
};

#[cfg(feature = "julia")]
#[path = "docs_page_index.rs"]
mod docs_page_index;
#[path = "scenario_matrix.rs"]
mod scenario_matrix;
#[cfg(feature = "julia")]
#[path = "semantic_gate.rs"]
mod semantic_gate;

#[test]
fn default_catalog_uses_managed_repo_contracts() {
    let catalog = default_real_repo_precision_catalog();
    assert_eq!(catalog.len(), 2);
    let entry = catalog
        .iter()
        .find(|entry| entry.repository.id == "xiuxian-artisan-workshop")
        .unwrap_or_else(|| panic!("missing xiuxian-artisan-workshop catalog entry"));

    assert_artisan_catalog_contract(entry);

    let pi_wendao = catalog
        .iter()
        .find(|entry| entry.repository.id == "pi-wendao")
        .unwrap_or_else(|| panic!("missing pi-wendao catalog entry"));
    assert_pi_wendao_catalog_contract(pi_wendao);
}

fn assert_artisan_catalog_contract(entry: &RealRepoPrecisionCatalogEntry) {
    assert_eq!(entry.repository.id, "xiuxian-artisan-workshop");
    assert!(entry.repository.path.is_none());
    assert_eq!(
        entry.repository.url.as_deref(),
        Some("https://github.com/tao3k/xiuxian-artisan-workshop.git")
    );
    assert_eq!(entry.repository.refresh, RepositoryRefreshPolicy::Manual);
    assert!(entry.include_dirs.iter().any(|path| path == "semantic"));
    assert_semantic_object_gold_query(
        &entry.gold_queries,
        "semantic-decision-repo-native-authority",
        "semantic/objects/decision/semantic-ssot-repo-native-first.md",
    );
    assert_docs_gold_query(
        &entry.gold_queries,
        "docs-polyglot-compute-orchestrator-rfc",
        "docs/rfcs/2026-05-04-polyglot-compute-orchestrator-rfc.md",
    );
    assert!(entry.gold_queries.iter().any(|query| matches!(
        query.kind,
        RealRepoGoldQueryKind::RepoAst
    ) && query.id
        == "repo-link-graph-build-with-filters-source"));
    assert!(entry.knowledge_scenarios.len() >= 7);
    assert!(
        entry
            .knowledge_scenarios
            .iter()
            .all(|scenario| !scenario.query_variants.is_empty())
    );
}

fn assert_pi_wendao_catalog_contract(pi_wendao: &RealRepoPrecisionCatalogEntry) {
    assert_eq!(
        pi_wendao.repository.path.as_deref(),
        Some(std::path::Path::new(".data/pi-wendao"))
    );
    assert_eq!(
        pi_wendao.repository.url.as_deref(),
        Some("https://github.com/tao3k/pi-wendao.git")
    );
    assert_eq!(pi_wendao.include_dirs, vec![".".to_string()]);
    assert_docs_gold_query(
        &pi_wendao.gold_queries,
        "pi-wendao-readme-subagents-host",
        "README.md",
    );
    assert_docs_gold_query(
        &pi_wendao.gold_queries,
        "pi-wendao-named-workflows-brainstorm-cache",
        "docs/named-workflows.md",
    );
    assert!(pi_wendao.gold_queries.iter().any(|query| matches!(
        query.kind,
        RealRepoGoldQueryKind::RepoAst
    ) && query.id
        == "pi-wendao-agent-host-interface-source"));
    assert!(
        pi_wendao
            .knowledge_scenarios
            .iter()
            .any(|scenario| scenario.id == "pi-wendao-agent-workflow-boundary")
    );
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

    let status = run_real_repo_precision_harness_with_options(&options, Vec::new())?;

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

    let status = run_real_repo_precision_harness_with_options(&options, catalog)?;

    let RealRepoPrecisionRunStatus::Completed(receipt) = status else {
        panic!("enabled status run should write a receipt");
    };
    assert_eq!(receipt.summary.repositories_total, 1);
    assert_eq!(receipt.summary.repositories_skipped, 1);
    assert_eq!(receipt.summary.queries_total, 0);
    assert!(receipt_path.exists());
    let payload = fs::read_to_string(receipt_path)?;
    assert!(payload.contains("missing-real-repo"));
    Ok(())
}

#[test]
fn pi_wendao_local_checkout_real_repo_harness_records_external_orchestration_evidence()
-> Result<(), Box<dyn std::error::Error>> {
    let project_root = test_project_root();
    if !project_root.join(".data/pi-wendao").is_dir() {
        return Ok(());
    }

    let temp = tempfile::tempdir()?;
    let options = RealRepoPrecisionRunOptions {
        enabled: true,
        sync_mode: RealRepoPrecisionSyncMode::Status,
        query_kind_filter: None,
        prewarmed_resident_only: false,
        project_root,
        receipt_path: temp.path().join("pi_wendao_receipt.json"),
        link_graph_cache_path: temp.path().join("pi_wendao_link_graph_cache.duckdb"),
    };
    let catalog = default_real_repo_precision_catalog()
        .into_iter()
        .filter(|entry| entry.repository.id == "pi-wendao")
        .collect::<Vec<_>>();

    let status = run_real_repo_precision_harness_with_options(&options, catalog)?;

    let RealRepoPrecisionRunStatus::Completed(receipt) = status else {
        panic!("pi-wendao local checkout proof should complete");
    };
    if receipt.summary.queries_failed > 0 {
        for repository in &receipt.repositories {
            for query in &repository.query_receipts {
                if !query.passed {
                    eprintln!(
                        "pi_wendao_failed_query id={} kind={} missing={:?} top={:?} observed={:?}",
                        query.query_id,
                        query.query_kind,
                        query.missing_paths,
                        query.observed_top_path,
                        query.observed_paths
                    );
                }
            }
        }
    }
    assert_eq!(receipt.summary.repositories_total, 1);
    assert_eq!(receipt.summary.queries_failed, 0);
    assert_eq!(receipt.summary.knowledge_scenarios_failed, 0);
    assert_eq!(receipt.summary.queries_total, 6);
    assert_eq!(receipt.summary.knowledge_scenarios_total, 2);

    let Some(repository) = receipt.repositories.first() else {
        panic!("pi-wendao proof should emit a repository receipt");
    };
    assert_eq!(repository.repo_id, "pi-wendao");
    assert!(repository.indexed);
    assert!(repository.repo_ast_index_file_count > 0);
    assert!(repository.repo_ast_index_symbol_count > 0);
    assert_eq!(repository.knowledge_scenarios.len(), 2);
    assert!(
        repository
            .knowledge_scenarios
            .iter()
            .all(|scenario| scenario.passed)
    );
    let corpus = repository
        .link_graph_corpus
        .as_ref()
        .unwrap_or_else(|| panic!("pi-wendao proof should emit LinkGraph corpus stats"));
    assert!(corpus.markdown_document_count >= 3);
    for query_id in [
        "pi-wendao-readme-subagents-host",
        "pi-wendao-named-workflows-brainstorm-cache",
        "pi-wendao-bpmn-format-runtime-ownership",
        "pi-wendao-subagents-extension-source",
        "pi-wendao-agent-host-interface-source",
        "pi-wendao-model-resolver-source",
    ] {
        assert!(
            repository
                .query_receipts
                .iter()
                .any(|query| query.query_id == query_id && query.passed),
            "missing passed pi-wendao query `{query_id}`"
        );
    }
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
            path: Some(repo_root.clone().into()),
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

    let first = run_real_repo_precision_harness_with_options(&options, catalog.clone())?;
    let RealRepoPrecisionRunStatus::Completed(first_receipt) = first else {
        panic!("enabled run should complete");
    };
    let first_repo = &first_receipt.repositories[0];
    let first_corpus = first_repo
        .link_graph_corpus
        .as_ref()
        .unwrap_or_else(|| panic!("LinkGraph corpus receipt should be present"));
    assert_eq!(first_corpus.document_count, 1);
    assert_eq!(
        first_repo.link_graph_cache_backend.as_deref(),
        Some("duckdb")
    );
    assert_eq!(first_repo.link_graph_cache_status.as_deref(), Some("miss"));
    assert_eq!(
        first_repo.link_graph_cache_miss_reason.as_deref(),
        Some("key_not_found")
    );
    assert_eq!(first_receipt.summary.queries_passed, 1);

    let second = run_real_repo_precision_harness_with_options(&options, catalog.clone())?;
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
    assert_eq!(second_receipt.summary.queries_passed, 1);

    let prewarmed_options = RealRepoPrecisionRunOptions {
        prewarmed_resident_only: true,
        ..options
    };
    let third = run_real_repo_precision_harness_with_options(&prewarmed_options, catalog)?;
    let RealRepoPrecisionRunStatus::Completed(third_receipt) = third else {
        panic!("enabled run should complete");
    };
    let third_repo = &third_receipt.repositories[0];
    assert_eq!(
        third_repo.link_graph_cache_backend.as_deref(),
        Some("resident-prewarmed")
    );
    assert_eq!(third_repo.link_graph_cache_status.as_deref(), Some("hit"));
    assert_eq!(third_receipt.summary.queries_passed, 1);

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
        &options,
        default_real_repo_precision_catalog(),
    )?;
    let RealRepoPrecisionRunStatus::Completed(receipt) = status else {
        panic!("docs corpus proof should complete");
    };
    assert_eq!(receipt.summary.queries_failed, 0);
    assert!(receipt.summary.indexed_documents >= 80);
    assert!(receipt.summary.indexed_markdown_documents >= 75);
    assert!(receipt.summary.indexed_total_words > 20_000);

    let Some(repository) = receipt.repositories.first() else {
        panic!("docs corpus proof should emit one repository receipt");
    };
    assert!(repository.query_wall_ms <= repository.total_ms);
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
    assert!(corpus.document_count >= 80);
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
    assert!(query_variant_count >= 15);
    assert!(
        repository
            .query_receipts
            .iter()
            .filter(|query| query.query_id.starts_with("docs-"))
            .all(|query| query.passed)
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
        &options,
        default_real_repo_precision_catalog(),
    )?;
    let RealRepoPrecisionRunStatus::Completed(first_receipt) = first else {
        panic!("resident proof should complete the first run");
    };
    assert_eq!(first_receipt.summary.queries_failed, 0);

    let second = run_real_repo_precision_harness_with_options(
        &options,
        default_real_repo_precision_catalog(),
    )?;
    let RealRepoPrecisionRunStatus::Completed(second_receipt) = second else {
        panic!("resident proof should complete the second run");
    };
    assert_eq!(second_receipt.summary.queries_failed, 0);
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
        &prewarm_options,
        default_real_repo_precision_catalog(),
    )?;
    let RealRepoPrecisionRunStatus::Completed(first_receipt) = first else {
        panic!("prewarm proof should complete the validating run");
    };
    assert_eq!(first_receipt.summary.queries_failed, 0);

    let mut request_options = prewarm_options;
    request_options.prewarmed_resident_only = true;
    let second = run_real_repo_precision_harness_with_options(
        &request_options,
        default_real_repo_precision_catalog(),
    )?;
    let RealRepoPrecisionRunStatus::Completed(second_receipt) = second else {
        panic!("prewarm proof should complete the request-only run");
    };
    assert_eq!(second_receipt.summary.queries_failed, 0);
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

fn test_project_root() -> PathBuf {
    std::env::var_os("PRJ_ROOT").map_or_else(
        || {
            let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            for _ in 0..4 {
                root = root
                    .parent()
                    .unwrap_or_else(|| panic!("crate path should be under the repository root"))
                    .to_path_buf();
            }
            root
        },
        PathBuf::from,
    )
}
