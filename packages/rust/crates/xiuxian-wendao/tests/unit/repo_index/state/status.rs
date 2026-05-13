use std::path::PathBuf;

use crate::repo_index::state::fingerprint::timestamp_now;
use crate::repo_index::state::task::{
    AdaptiveConcurrencyController, repo_index_sync_concurrency_limit,
    repo_index_sync_requeue_attempt_limit,
};
use crate::repo_index::state::tests::new_coordinator;
use crate::repo_index::types::{RepoIndexEntryStatus, RepoIndexPhase};
use crate::search::SearchPlaneService;

#[test]
fn status_response_counts_each_phase() {
    let coordinator = new_coordinator(SearchPlaneService::new(PathBuf::from(".")));
    coordinator.set_concurrency_for_test(AdaptiveConcurrencyController::new_for_test(6));
    coordinator.set_status_for_test(RepoIndexEntryStatus {
        repo_id: "queued".to_string(),
        phase: RepoIndexPhase::Queued,
        queue_position: None,
        last_error: None,
        last_revision: None,
        updated_at: Some(timestamp_now()),
        attempt_count: 1,
    });
    coordinator.set_status_for_test(RepoIndexEntryStatus {
        repo_id: "ready".to_string(),
        phase: RepoIndexPhase::Ready,
        queue_position: None,
        last_error: None,
        last_revision: None,
        updated_at: Some(timestamp_now()),
        attempt_count: 1,
    });

    let status = coordinator.status_response(None);
    assert_eq!(status.total, 2);
    assert_eq!(status.target_concurrency, 1);
    assert_eq!(status.max_concurrency, 6);
    assert_eq!(
        status.sync_concurrency_limit,
        repo_index_sync_concurrency_limit()
    );
    assert_eq!(status.queued, 1);
    assert_eq!(status.ready, 1);
}

#[test]
fn status_response_filters_case_insensitively_from_cached_snapshot() {
    let coordinator = new_coordinator(SearchPlaneService::new(PathBuf::from(".")));
    coordinator.set_status_for_test(RepoIndexEntryStatus {
        repo_id: "DifferentialEquations.jl".to_string(),
        phase: RepoIndexPhase::Indexing,
        queue_position: None,
        last_error: None,
        last_revision: Some("abc123".to_string()),
        updated_at: Some(timestamp_now()),
        attempt_count: 2,
    });
    coordinator.mark_active_for_test("DifferentialEquations.jl");

    let status = coordinator.status_response(Some("differentialequations.jl"));
    assert_eq!(status.total, 1);
    assert_eq!(status.active, 1);
    assert_eq!(status.indexing, 1);
    assert_eq!(
        status.current_repo_id.as_deref(),
        Some("DifferentialEquations.jl")
    );
}

#[test]
fn status_response_exposes_active_repos_and_concurrency_metadata() {
    let coordinator = new_coordinator(SearchPlaneService::new(PathBuf::from(".")));
    coordinator.set_concurrency_for_test(
        crate::repo_index::state::task::AdaptiveConcurrencyController::new_for_test(8),
    );
    coordinator.set_status_for_test(RepoIndexEntryStatus {
        repo_id: "ADTypes.jl".to_string(),
        phase: RepoIndexPhase::Indexing,
        queue_position: None,
        last_error: None,
        last_revision: Some("abc123".to_string()),
        updated_at: Some(timestamp_now()),
        attempt_count: 1,
    });
    coordinator.mark_active_for_test("ADTypes.jl");

    let status = coordinator.status_response(None);
    assert_eq!(status.active, 1);
    assert_eq!(status.current_repo_id.as_deref(), Some("ADTypes.jl"));
    assert_eq!(status.active_repo_ids, vec!["ADTypes.jl".to_string()]);
    assert_eq!(status.target_concurrency, 1);
    assert_eq!(status.max_concurrency, 8);
    assert_eq!(
        status.sync_concurrency_limit,
        repo_index_sync_concurrency_limit()
    );
}

#[test]
fn retry_budget_surface_stays_positive() {
    assert!(repo_index_sync_requeue_attempt_limit() > 0);
}
