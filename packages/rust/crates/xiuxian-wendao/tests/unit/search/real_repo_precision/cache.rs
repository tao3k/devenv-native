use std::fs;

use xiuxian_wendao_core::repo_intelligence::{RegisteredRepository, RepositoryRefreshPolicy};

use crate::link_graph::LinkGraphIndex;
use crate::search::real_repo_precision::{
    RealRepoGoldQueryKind, RealRepoPrecisionCatalogEntry, RealRepoPrecisionRunOptions,
    RealRepoPrecisionRunStatus, RealRepoPrecisionSyncMode,
    run_real_repo_precision_harness_with_options,
};

use super::support::gold_query;

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
