use xiuxian_db_store::LanceArray;

use crate::analyzers::RepositoryRefreshPolicy;
use crate::analyzers::{
    RepoSourceKind, RepoSyncDriftState, RepoSyncFreshnessSummary, RepoSyncHealthState,
    RepoSyncLifecycleSummary, RepoSyncMode, RepoSyncResult, RepoSyncRevisionSummary,
    RepoSyncStalenessState, RepoSyncState, RepoSyncStatusSummary,
};
use crate::gateway::studio::router::handlers::repo::analysis::sync_flight::build_repo_sync_flight_batch;

fn sample_sync_result() -> RepoSyncResult {
    RepoSyncResult {
        repo_id: "gateway-sync".to_string(),
        mode: RepoSyncMode::Status,
        source_kind: RepoSourceKind::ManagedRemote,
        refresh: RepositoryRefreshPolicy::Fetch,
        mirror_state: RepoSyncState::Validated,
        checkout_state: RepoSyncState::Reused,
        checkout_path: "/tmp/gateway-sync".to_string(),
        mirror_path: Some("/tmp/gateway-sync.mirror".to_string()),
        checked_at: "2026-04-03T19:15:00Z".to_string(),
        last_fetched_at: Some("2026-04-03T19:10:00Z".to_string()),
        mirror_revision: Some("mirror:123".to_string()),
        tracking_revision: Some("tracking:123".to_string()),
        upstream_url: Some("https://example.com/repo.git".to_string()),
        drift_state: RepoSyncDriftState::InSync,
        health_state: RepoSyncHealthState::Healthy,
        staleness_state: RepoSyncStalenessState::Fresh,
        status_summary: RepoSyncStatusSummary {
            lifecycle: RepoSyncLifecycleSummary {
                source_kind: RepoSourceKind::ManagedRemote,
                mirror_state: RepoSyncState::Validated,
                checkout_state: RepoSyncState::Reused,
                mirror_ready: true,
                checkout_ready: true,
            },
            freshness: RepoSyncFreshnessSummary {
                checked_at: "2026-04-03T19:15:00Z".to_string(),
                last_fetched_at: Some("2026-04-03T19:10:00Z".to_string()),
                staleness_state: RepoSyncStalenessState::Fresh,
            },
            revisions: RepoSyncRevisionSummary {
                checkout_revision: Some("rev:123".to_string()),
                mirror_revision: Some("mirror:123".to_string()),
                tracking_revision: Some("tracking:123".to_string()),
                aligned_with_mirror: true,
            },
            health_state: RepoSyncHealthState::Healthy,
            drift_state: RepoSyncDriftState::InSync,
            attention_required: false,
        },
        revision: Some("rev:123".to_string()),
    }
}

#[test]
fn repo_sync_flight_batch_preserves_summary_fields() {
    let batch = build_repo_sync_flight_batch(&sample_sync_result())
        .unwrap_or_else(|error| panic!("repo sync batch should build: {error}"));

    assert_eq!(batch.num_rows(), 1);
    let Some(mode_column) = batch.column_by_name("mode") else {
        panic!("mode column");
    };
    let Some(mode) = mode_column
        .as_any()
        .downcast_ref::<xiuxian_db_store::LanceStringArray>()
    else {
        panic!("mode should be utf8");
    };
    assert_eq!(mode.value(0), "status");

    let Some(health_state_column) = batch.column_by_name("healthState") else {
        panic!("healthState column");
    };
    let Some(health_state) = health_state_column
        .as_any()
        .downcast_ref::<xiuxian_db_store::LanceStringArray>()
    else {
        panic!("healthState should be utf8");
    };
    assert_eq!(health_state.value(0), "healthy");
}

#[test]
fn repo_sync_flight_metadata_preserves_full_payload() {
    let metadata = serde_json::to_vec(&sample_sync_result())
        .unwrap_or_else(|error| panic!("metadata should encode: {error}"));
    let payload: serde_json::Value = serde_json::from_slice(&metadata)
        .unwrap_or_else(|error| panic!("metadata should decode: {error}"));
    assert_eq!(payload["repo_id"], "gateway-sync");
    assert_eq!(payload["mode"], "status");
    assert_eq!(payload["health_state"], "healthy");
}
