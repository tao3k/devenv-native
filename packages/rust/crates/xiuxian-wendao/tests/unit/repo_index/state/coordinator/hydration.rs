use std::fs;

use super::{
    managed_remote_bootstrap_is_safe, managed_remote_probe_freshness,
    managed_remote_retryable_probe_failure_is_recent,
};
use crate::analyzers::{RegisteredRepository, RepositoryPluginConfig, RepositoryRefreshPolicy};
use crate::analyzers::{
    RepoSourceKind, RepoSyncHealthState, RepoSyncResult, RepoSyncStalenessState,
};
use crate::repo_index::types::{RepoIndexEntryStatus, RepoIndexPhase};
use xiuxian_git_repo::{record_managed_remote_probe_failure, record_managed_remote_probe_state};

fn tempdir_or_panic() -> tempfile::TempDir {
    tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"))
}

fn read_json_value_or_panic(path: &std::path::Path) -> serde_json::Value {
    let payload = fs::read(path).unwrap_or_else(|error| panic!("read probe state: {error}"));
    serde_json::from_slice(&payload).unwrap_or_else(|error| panic!("parse probe state: {error}"))
}

fn write_json_value_or_panic(path: &std::path::Path, payload: &serde_json::Value) {
    let encoded =
        serde_json::to_vec(payload).unwrap_or_else(|error| panic!("encode probe state: {error}"));
    fs::write(path, encoded).unwrap_or_else(|error| panic!("rewrite probe state: {error}"));
}

#[test]
fn managed_remote_bootstrap_requires_ready_status_and_matching_revision() {
    let repository = managed_remote_repository(RepositoryRefreshPolicy::Fetch);
    let mut sync_result = managed_remote_sync_result();

    let ready = ready_status(Some("rev-1"));
    assert!(managed_remote_bootstrap_is_safe(
        &repository,
        &ready,
        &sync_result,
        None,
        false,
    ));

    sync_result.revision = Some("rev-2".to_string());
    assert!(!managed_remote_bootstrap_is_safe(
        &repository,
        &ready,
        &sync_result,
        None,
        false,
    ));

    assert!(!managed_remote_bootstrap_is_safe(
        &repository,
        &RepoIndexEntryStatus {
            phase: RepoIndexPhase::Failed,
            ..ready
        },
        &managed_remote_sync_result(),
        None,
        false,
    ));
}

#[test]
fn persisted_publication_bootstrap_only_accepts_ready_status() {
    assert!(super::persisted_publication_bootstrap_is_searchable(
        &ready_status(None)
    ));
    assert!(!super::persisted_publication_bootstrap_is_searchable(
        &RepoIndexEntryStatus {
            phase: RepoIndexPhase::Queued,
            ..ready_status(None)
        }
    ));
}

#[test]
fn managed_remote_bootstrap_allows_aging_fetch_for_fetch_policy() {
    let repository = managed_remote_repository(RepositoryRefreshPolicy::Fetch);
    let mut sync_result = managed_remote_sync_result();
    sync_result.staleness_state = RepoSyncStalenessState::Aging;

    assert!(managed_remote_bootstrap_is_safe(
        &repository,
        &ready_status(Some("rev-1")),
        &sync_result,
        None,
        false,
    ));
}

#[test]
fn managed_remote_bootstrap_rejects_stale_fetch_for_fetch_policy() {
    let repository = managed_remote_repository(RepositoryRefreshPolicy::Fetch);
    let mut sync_result = managed_remote_sync_result();
    sync_result.staleness_state = RepoSyncStalenessState::Stale;

    assert!(!managed_remote_bootstrap_is_safe(
        &repository,
        &ready_status(Some("rev-1")),
        &sync_result,
        None,
        false,
    ));
}

#[test]
fn managed_remote_bootstrap_allows_manual_policy_without_fresh_fetch() {
    let repository = managed_remote_repository(RepositoryRefreshPolicy::Manual);
    let mut sync_result = managed_remote_sync_result();
    sync_result.staleness_state = RepoSyncStalenessState::Stale;

    assert!(managed_remote_bootstrap_is_safe(
        &repository,
        &ready_status(Some("rev-1")),
        &sync_result,
        None,
        false,
    ));
}

#[test]
fn managed_remote_bootstrap_allows_stale_commit_pinned_fetch_policy() {
    let mut repository = managed_remote_repository(RepositoryRefreshPolicy::Fetch);
    repository.git_ref = Some(crate::analyzers::RepositoryRef::Commit("rev-1".to_string()));
    let mut sync_result = managed_remote_sync_result();
    sync_result.staleness_state = RepoSyncStalenessState::Stale;

    assert!(managed_remote_bootstrap_is_safe(
        &repository,
        &ready_status(Some("rev-1")),
        &sync_result,
        None,
        false,
    ));
}

#[test]
fn managed_remote_bootstrap_allows_stale_fetch_policy_when_probe_state_is_recent() {
    let repository = managed_remote_repository(RepositoryRefreshPolicy::Fetch);
    let mut sync_result = managed_remote_sync_result();
    sync_result.staleness_state = RepoSyncStalenessState::Stale;

    let probe_dir = tempdir_or_panic();
    record_managed_remote_probe_state(probe_dir.path(), Some("rev-1"))
        .unwrap_or_else(|error| panic!("record probe state: {error}"));
    sync_result.mirror_path = Some(probe_dir.path().display().to_string());

    assert!(managed_remote_bootstrap_is_safe(
        &repository,
        &ready_status(Some("rev-1")),
        &sync_result,
        managed_remote_probe_freshness(&sync_result),
        false,
    ));
}

#[test]
fn managed_remote_probe_freshness_ignores_mismatched_revision() {
    let mut sync_result = managed_remote_sync_result();
    let probe_dir = tempdir_or_panic();
    record_managed_remote_probe_state(probe_dir.path(), Some("rev-2"))
        .unwrap_or_else(|error| panic!("record probe state: {error}"));
    sync_result.mirror_path = Some(probe_dir.path().display().to_string());

    assert_eq!(managed_remote_probe_freshness(&sync_result), None);
}

#[test]
fn managed_remote_probe_freshness_reports_stale_for_old_probe_state() {
    let mut sync_result = managed_remote_sync_result();
    let probe_dir = tempdir_or_panic();
    record_managed_remote_probe_state(probe_dir.path(), Some("rev-1"))
        .unwrap_or_else(|error| panic!("record probe state: {error}"));
    let state_path = probe_dir.path().join("xiuxian-upstream-probe-state.json");
    let mut payload = read_json_value_or_panic(&state_path);
    payload["checked_at"] = serde_json::Value::String("2000-01-01T00:00:00+00:00".to_string());
    write_json_value_or_panic(&state_path, &payload);
    sync_result.mirror_path = Some(probe_dir.path().display().to_string());

    assert_eq!(
        managed_remote_probe_freshness(&sync_result),
        Some(RepoSyncStalenessState::Stale)
    );
}

#[test]
fn managed_remote_bootstrap_allows_recent_retryable_probe_failure() {
    let repository = managed_remote_repository(RepositoryRefreshPolicy::Fetch);
    let mut sync_result = managed_remote_sync_result();
    sync_result.staleness_state = RepoSyncStalenessState::Stale;

    let probe_dir = tempdir_or_panic();
    record_managed_remote_probe_failure(probe_dir.path(), "operation timed out", true)
        .unwrap_or_else(|error| panic!("record probe failure: {error}"));
    sync_result.mirror_path = Some(probe_dir.path().display().to_string());

    assert!(managed_remote_retryable_probe_failure_is_recent(
        &sync_result
    ));
    assert!(managed_remote_bootstrap_is_safe(
        &repository,
        &ready_status(Some("rev-1")),
        &sync_result,
        None,
        true,
    ));
}

#[test]
fn managed_remote_bootstrap_rejects_non_retryable_probe_failure() {
    let repository = managed_remote_repository(RepositoryRefreshPolicy::Fetch);
    let mut sync_result = managed_remote_sync_result();
    sync_result.staleness_state = RepoSyncStalenessState::Stale;

    let probe_dir = tempdir_or_panic();
    record_managed_remote_probe_failure(probe_dir.path(), "authentication required", false)
        .unwrap_or_else(|error| panic!("record probe failure: {error}"));
    sync_result.mirror_path = Some(probe_dir.path().display().to_string());

    assert!(!managed_remote_retryable_probe_failure_is_recent(
        &sync_result
    ));
    assert!(!managed_remote_bootstrap_is_safe(
        &repository,
        &ready_status(Some("rev-1")),
        &sync_result,
        None,
        false,
    ));
}

#[test]
fn managed_remote_probe_freshness_uses_last_success_marker_after_retryable_failure() {
    let mut sync_result = managed_remote_sync_result();
    let probe_dir = tempdir_or_panic();
    record_managed_remote_probe_state(probe_dir.path(), Some("rev-1"))
        .unwrap_or_else(|error| panic!("record probe state: {error}"));
    record_managed_remote_probe_failure(probe_dir.path(), "operation timed out", true)
        .unwrap_or_else(|error| panic!("record probe failure: {error}"));
    sync_result.mirror_path = Some(probe_dir.path().display().to_string());

    assert!(matches!(
        managed_remote_probe_freshness(&sync_result),
        Some(RepoSyncStalenessState::Fresh)
    ));
}

fn managed_remote_repository(refresh: RepositoryRefreshPolicy) -> RegisteredRepository {
    RegisteredRepository {
        id: "managed-remote".to_string(),
        path: None,
        url: Some("https://example.com/managed-remote.git".to_string()),
        git_ref: None,
        refresh,
        plugins: vec![RepositoryPluginConfig::Id("julia".to_string())],
    }
}

fn ready_status(last_revision: Option<&str>) -> RepoIndexEntryStatus {
    RepoIndexEntryStatus {
        repo_id: "managed-remote".to_string(),
        phase: RepoIndexPhase::Ready,
        queue_position: None,
        last_error: None,
        last_revision: last_revision.map(str::to_string),
        updated_at: Some("2026-04-02T00:00:00Z".to_string()),
        attempt_count: 0,
    }
}

fn managed_remote_sync_result() -> RepoSyncResult {
    RepoSyncResult {
        repo_id: "managed-remote".to_string(),
        source_kind: RepoSourceKind::ManagedRemote,
        health_state: RepoSyncHealthState::Healthy,
        staleness_state: RepoSyncStalenessState::Fresh,
        revision: Some("rev-1".to_string()),
        ..RepoSyncResult::default()
    }
}
