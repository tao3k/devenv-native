use std::fs;

use xiuxian_wendao_core::repo_intelligence::{RegisteredRepository, RepositoryRefreshPolicy};

use crate::search::real_repo_precision::{
    DOCS_CORPUS_PROOF_ENV, PREWARM_PROOF_ENV, RESIDENT_PROOF_ENV, RealRepoGoldQueryKind,
    RealRepoPrecisionCatalogEntry, RealRepoPrecisionRunOptions, RealRepoPrecisionRunStatus,
    RealRepoPrecisionSyncMode, default_real_repo_precision_catalog,
    run_real_repo_precision_harness, run_real_repo_precision_harness_with_options,
};

use super::support::{gold_query, test_project_root};

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
    assert_eq!(receipt.summary.queries_total, 3);
    assert_eq!(receipt.summary.knowledge_scenarios_total, 2);

    let Some(repository) = receipt.repositories.first() else {
        panic!("pi-wendao proof should emit a repository receipt");
    };
    assert_eq!(repository.repo_id, "pi-wendao");
    assert!(repository.indexed);
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
