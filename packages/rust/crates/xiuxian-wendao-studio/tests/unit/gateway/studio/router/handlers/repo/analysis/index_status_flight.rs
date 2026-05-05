use crate::studio::arrow_types::LanceArray;
#[cfg(feature = "duckdb")]
use xiuxian_wendao::duckdb::LocalRelationEngineKind;

#[cfg(feature = "duckdb")]
use crate::studio::router::handlers::repo::analysis::index_status_flight::configured_repo_index_status_diagnostics_engine_kind;
use crate::studio::router::handlers::repo::analysis::index_status_flight::{
    build_repo_index_status_flight_batch, build_repo_index_status_flight_metadata,
    repo_index_status_response_with_diagnostics, summarize_repo_index_status_diagnostics,
};
use xiuxian_wendao::repo_index::{RepoIndexEntryStatus, RepoIndexPhase, RepoIndexStatusResponse};
#[cfg(feature = "duckdb")]
use xiuxian_wendao::set_link_graph_wendao_config_override;

fn repo_index_status_row(
    repo_id: &str,
    phase: RepoIndexPhase,
    revision: &str,
    updated_at: &str,
    attempt_count: u32,
) -> RepoIndexEntryStatus {
    RepoIndexEntryStatus {
        repo_id: repo_id.to_string(),
        phase,
        queue_position: None,
        last_error: None,
        last_revision: Some(revision.to_string()),
        updated_at: Some(updated_at.to_string()),
        attempt_count: attempt_count as usize,
    }
}

#[test]
fn repo_index_status_flight_batch_preserves_summary_fields() {
    let batch = build_repo_index_status_flight_batch(&RepoIndexStatusResponse {
        total: 3,
        active: 2,
        queued: 1,
        checking: 0,
        syncing: 1,
        indexing: 1,
        ready: 1,
        unsupported: 0,
        failed: 0,
        target_concurrency: 2,
        max_concurrency: 4,
        sync_concurrency_limit: 1,
        current_repo_id: Some("gateway-sync".to_string()),
        active_repo_ids: vec!["gateway-sync".to_string()],
        repos: vec![RepoIndexEntryStatus {
            repo_id: "gateway-sync".to_string(),
            phase: RepoIndexPhase::Ready,
            queue_position: None,
            last_error: None,
            last_revision: Some("rev:123".to_string()),
            updated_at: Some("2026-04-03T19:15:00Z".to_string()),
            attempt_count: 2,
        }],
    })
    .unwrap_or_else(|error| panic!("repo index status batch should build: {error}"));

    assert_eq!(batch.num_rows(), 1);
    let Some(ready_column) = batch.column_by_name("ready") else {
        panic!("ready column");
    };
    let Some(ready) = ready_column
        .as_any()
        .downcast_ref::<crate::studio::arrow_types::LanceInt32Array>()
    else {
        panic!("ready should be int32");
    };
    assert_eq!(ready.value(0), 1);

    let Some(repos_json_column) = batch.column_by_name("reposJson") else {
        panic!("reposJson column");
    };
    let Some(repos_json) = repos_json_column
        .as_any()
        .downcast_ref::<crate::studio::arrow_types::LanceStringArray>()
    else {
        panic!("reposJson should be utf8");
    };
    assert!(repos_json.value(0).contains("gateway-sync"));
}

#[test]
fn repo_index_status_flight_metadata_preserves_summary_fields() {
    let metadata = build_repo_index_status_flight_metadata(&RepoIndexStatusResponse {
        total: 3,
        active: 2,
        queued: 1,
        checking: 0,
        syncing: 1,
        indexing: 1,
        ready: 1,
        unsupported: 0,
        failed: 0,
        target_concurrency: 2,
        max_concurrency: 4,
        sync_concurrency_limit: 1,
        current_repo_id: Some("gateway-sync".to_string()),
        active_repo_ids: vec!["gateway-sync".to_string()],
        repos: vec![RepoIndexEntryStatus {
            repo_id: "gateway-sync".to_string(),
            phase: RepoIndexPhase::Ready,
            queue_position: None,
            last_error: None,
            last_revision: Some("rev:123".to_string()),
            updated_at: Some("2026-04-03T19:15:00Z".to_string()),
            attempt_count: 2,
        }],
    })
    .unwrap_or_else(|error| panic!("repo index status metadata should encode: {error}"));

    let payload: serde_json::Value = serde_json::from_slice(&metadata)
        .unwrap_or_else(|error| panic!("metadata should decode: {error}"));
    assert_eq!(payload["total"], 3);
    assert_eq!(payload["syncConcurrencyLimit"], 1);
    assert_eq!(payload["currentRepoId"], "gateway-sync");
    assert_eq!(payload["repos"][0]["repoId"], "gateway-sync");
}

#[tokio::test]
async fn repo_index_status_diagnostics_recompute_summary_counts_from_rows() {
    let input = RepoIndexStatusResponse {
        total: 99,
        active: 99,
        queued: 99,
        checking: 99,
        syncing: 99,
        indexing: 99,
        ready: 99,
        unsupported: 99,
        failed: 99,
        target_concurrency: 2,
        max_concurrency: 4,
        sync_concurrency_limit: 1,
        current_repo_id: Some("stale-current".to_string()),
        active_repo_ids: vec!["gateway-failed".to_string(), "gateway-sync".to_string()],
        repos: vec![
            RepoIndexEntryStatus {
                repo_id: "gateway-sync".to_string(),
                phase: RepoIndexPhase::Queued,
                queue_position: Some(1),
                last_error: None,
                last_revision: Some("rev:123".to_string()),
                updated_at: Some("2026-04-03T19:15:00Z".to_string()),
                attempt_count: 2,
            },
            RepoIndexEntryStatus {
                repo_id: "gateway-ready".to_string(),
                phase: RepoIndexPhase::Ready,
                queue_position: None,
                last_error: None,
                last_revision: Some("rev:456".to_string()),
                updated_at: Some("2026-04-03T19:16:00Z".to_string()),
                attempt_count: 1,
            },
            RepoIndexEntryStatus {
                repo_id: "gateway-failed".to_string(),
                phase: RepoIndexPhase::Failed,
                queue_position: None,
                last_error: Some("boom".to_string()),
                last_revision: None,
                updated_at: Some("2026-04-03T19:17:00Z".to_string()),
                attempt_count: 3,
            },
        ],
    };

    let summary = summarize_repo_index_status_diagnostics(&input)
        .await
        .unwrap_or_else(|error| panic!("repo index diagnostics summary should build: {error}"));
    assert_eq!(summary.total, 3);
    assert_eq!(summary.active, 2);
    assert_eq!(summary.queued, 1);
    assert_eq!(summary.ready, 1);
    assert_eq!(summary.failed, 1);
    assert_eq!(
        summary.active_repo_ids,
        vec!["gateway-failed".to_string(), "gateway-sync".to_string()]
    );
    assert_eq!(summary.current_repo_id.as_deref(), Some("gateway-failed"));

    let response = repo_index_status_response_with_diagnostics(&input).await;

    assert_eq!(response.total, 3);
    assert_eq!(response.active, 2);
    assert_eq!(response.queued, 1);
    assert_eq!(response.checking, 0);
    assert_eq!(response.syncing, 0);
    assert_eq!(response.indexing, 0);
    assert_eq!(response.ready, 1);
    assert_eq!(response.unsupported, 0);
    assert_eq!(response.failed, 1);
    assert_eq!(response.target_concurrency, 2);
    assert_eq!(response.current_repo_id.as_deref(), Some("gateway-failed"));
    assert_eq!(
        response.active_repo_ids,
        vec!["gateway-failed".to_string(), "gateway-sync".to_string()]
    );
}

#[tokio::test]
async fn repo_index_status_diagnostics_ignore_repo_row_order_when_active_order_is_fixed() {
    let left = RepoIndexStatusResponse {
        total: 0,
        active: 0,
        queued: 0,
        checking: 0,
        syncing: 0,
        indexing: 0,
        ready: 0,
        unsupported: 0,
        failed: 0,
        target_concurrency: 2,
        max_concurrency: 4,
        sync_concurrency_limit: 1,
        current_repo_id: Some("stale-current".to_string()),
        active_repo_ids: vec!["gateway-sync".to_string(), "gateway-ready".to_string()],
        repos: vec![
            repo_index_status_row(
                "gateway-ready",
                RepoIndexPhase::Ready,
                "rev:456",
                "2026-04-03T19:16:00Z",
                1,
            ),
            repo_index_status_row(
                "gateway-sync",
                RepoIndexPhase::Syncing,
                "rev:789",
                "2026-04-03T19:18:00Z",
                2,
            ),
        ],
    };
    let right = RepoIndexStatusResponse {
        repos: vec![
            repo_index_status_row(
                "gateway-sync",
                RepoIndexPhase::Syncing,
                "rev:789",
                "2026-04-03T19:18:00Z",
                2,
            ),
            repo_index_status_row(
                "gateway-ready",
                RepoIndexPhase::Ready,
                "rev:456",
                "2026-04-03T19:16:00Z",
                1,
            ),
        ],
        ..left.clone()
    };

    let left_summary = summarize_repo_index_status_diagnostics(&left)
        .await
        .unwrap_or_else(|error| panic!("left diagnostics summary should build: {error}"));
    let right_summary = summarize_repo_index_status_diagnostics(&right)
        .await
        .unwrap_or_else(|error| panic!("right diagnostics summary should build: {error}"));
    assert_eq!(left_summary, right_summary);
    assert_eq!(
        left_summary.active_repo_ids,
        vec!["gateway-sync".to_string(), "gateway-ready".to_string()]
    );
    assert_eq!(
        left_summary.current_repo_id.as_deref(),
        Some("gateway-sync")
    );

    let left_response = repo_index_status_response_with_diagnostics(&left).await;
    let right_response = repo_index_status_response_with_diagnostics(&right).await;
    assert_eq!(
        left_response.active_repo_ids,
        vec!["gateway-sync".to_string(), "gateway-ready".to_string()]
    );
    assert_eq!(
        left_response.current_repo_id.as_deref(),
        Some("gateway-sync")
    );
    assert_eq!(left_response.total, right_response.total);
    assert_eq!(left_response.active, right_response.active);
    assert_eq!(left_response.queued, right_response.queued);
    assert_eq!(left_response.checking, right_response.checking);
    assert_eq!(left_response.syncing, right_response.syncing);
    assert_eq!(left_response.indexing, right_response.indexing);
    assert_eq!(left_response.ready, right_response.ready);
    assert_eq!(left_response.unsupported, right_response.unsupported);
    assert_eq!(left_response.failed, right_response.failed);
    assert_eq!(
        left_response.current_repo_id,
        right_response.current_repo_id
    );
    assert_eq!(
        left_response.active_repo_ids,
        right_response.active_repo_ids
    );
    assert_eq!(left_response.repos, left.repos);
    assert_eq!(right_response.repos, right.repos);
}

#[cfg(feature = "duckdb")]
#[tokio::test]
async fn repo_index_status_diagnostics_select_duckdb_when_enabled() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let config_path = temp.path().join("wendao.toml");
    std::fs::write(
        &config_path,
        r#"[search.duckdb]
enabled = true
database_path = ":memory:"
temp_directory = ".data/duckdb/tmp"
threads = 2
materialize_threshold_rows = 16
prefer_virtual_arrow = true
"#,
    )
    .unwrap_or_else(|error| panic!("write config: {error}"));
    set_link_graph_wendao_config_override(&config_path.to_string_lossy());

    assert_eq!(
        configured_repo_index_status_diagnostics_engine_kind()
            .unwrap_or(LocalRelationEngineKind::DataFusion),
        LocalRelationEngineKind::DuckDb
    );

    let input = RepoIndexStatusResponse {
        total: 0,
        active: 0,
        queued: 0,
        checking: 0,
        syncing: 0,
        indexing: 0,
        ready: 0,
        unsupported: 0,
        failed: 0,
        target_concurrency: 2,
        max_concurrency: 4,
        sync_concurrency_limit: 1,
        current_repo_id: Some("stale-current".to_string()),
        active_repo_ids: vec!["gateway-sync".to_string(), "gateway-ready".to_string()],
        repos: vec![
            RepoIndexEntryStatus {
                repo_id: "gateway-ready".to_string(),
                phase: RepoIndexPhase::Ready,
                queue_position: None,
                last_error: None,
                last_revision: Some("rev:456".to_string()),
                updated_at: Some("2026-04-03T19:16:00Z".to_string()),
                attempt_count: 1,
            },
            RepoIndexEntryStatus {
                repo_id: "gateway-sync".to_string(),
                phase: RepoIndexPhase::Syncing,
                queue_position: None,
                last_error: None,
                last_revision: Some("rev:789".to_string()),
                updated_at: Some("2026-04-03T19:18:00Z".to_string()),
                attempt_count: 2,
            },
        ],
    };

    let summary = summarize_repo_index_status_diagnostics(&input)
        .await
        .unwrap_or_else(|error| panic!("repo index diagnostics summary should build: {error}"));
    assert_eq!(summary.total, 2);
    assert_eq!(summary.active, 2);
    assert_eq!(summary.ready, 1);
    assert_eq!(summary.syncing, 1);
    assert_eq!(
        summary.active_repo_ids,
        vec!["gateway-sync".to_string(), "gateway-ready".to_string()]
    );
    assert_eq!(summary.current_repo_id.as_deref(), Some("gateway-sync"));

    let response = repo_index_status_response_with_diagnostics(&input).await;

    assert_eq!(response.total, 2);
    assert_eq!(response.active, 2);
    assert_eq!(response.ready, 1);
    assert_eq!(response.syncing, 1);
    assert_eq!(response.current_repo_id.as_deref(), Some("gateway-sync"));
    assert_eq!(
        response.active_repo_ids,
        vec!["gateway-sync".to_string(), "gateway-ready".to_string()]
    );
}
