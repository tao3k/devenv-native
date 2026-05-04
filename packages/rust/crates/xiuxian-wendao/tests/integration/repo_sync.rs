//! Integration tests for explicit Repo Intelligence source synchronization.

use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};

use crate::support::repo_fixture::refresh_remote;
use crate::support::repo_intelligence::{
    assert_repo_json_snapshot, create_sample_julia_repo, write_repo_config,
};
use crate::support::wendao_command;
use serde_json::json;
use uuid::Uuid;
use xiuxian_io::PrjDirs;
use xiuxian_wendao::analyzers::{
    RepoSyncDriftState, RepoSyncHealthState, RepoSyncMode, RepoSyncQuery, RepoSyncStalenessState,
    repo_sync_from_config,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn repo_sync_reports_local_checkout_state() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_julia_repo(temp.path(), "LocalSyncPkg", true)?;
    let config_path = write_repo_config(temp.path(), &repo_dir, "local-sync")?;

    let result = repo_sync_from_config(
        &RepoSyncQuery {
            repo_id: "local-sync".to_string(),
            mode: RepoSyncMode::Ensure,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_eq!(result.health_state, RepoSyncHealthState::Healthy);
    assert!(result.checked_at.contains('T'));
    assert!(result.last_fetched_at.is_none());
    assert_eq!(
        result.staleness_state,
        RepoSyncStalenessState::NotApplicable
    );
    assert!(result.status_summary.lifecycle.checkout_ready);
    assert!(!result.status_summary.attention_required);

    let mut payload = json!(result);
    redact_checkout_path(&mut payload);
    redact_sync_timestamps(&mut payload);
    redact_sync_revisions(&mut payload);
    assert_repo_json_snapshot("repo_sync_local_result", payload);
    Ok(())
}

#[test]
fn repo_sync_reports_managed_remote_state() -> TestResult {
    let temp = tempfile::tempdir()?;
    let source_repo = create_sample_julia_repo(temp.path(), "ManagedSyncPkg", true)?;
    clear_managed_repo_cache(temp.path(), "managed-sync")?;
    let config_path = write_repo_url_config(temp.path(), &source_repo, "managed-sync")?;

    let result = repo_sync_from_config(
        &RepoSyncQuery {
            repo_id: "managed-sync".to_string(),
            mode: RepoSyncMode::Ensure,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_eq!(result.health_state, RepoSyncHealthState::Healthy);
    assert!(result.checked_at.contains('T'));
    assert!(result.last_fetched_at.is_some());
    assert_eq!(result.staleness_state, RepoSyncStalenessState::Fresh);
    assert!(result.status_summary.lifecycle.mirror_ready);
    assert!(result.status_summary.revisions.aligned_with_mirror);

    let mut payload = json!(result);
    redact_checkout_path(&mut payload);
    redact_mirror_path(&mut payload);
    redact_sync_timestamps(&mut payload);
    redact_upstream_url(&mut payload);
    redact_sync_revisions(&mut payload);
    assert_repo_json_snapshot("repo_sync_managed_result", payload);
    Ok(())
}

#[test]
fn repo_sync_status_reports_missing_managed_assets_without_materializing() -> TestResult {
    let temp = tempfile::tempdir()?;
    let source_repo = create_sample_julia_repo(temp.path(), "ManagedStatusPkg", true)?;
    clear_managed_repo_cache(temp.path(), "managed-status")?;
    let config_path = write_repo_url_config(temp.path(), &source_repo, "managed-status")?;

    let result = repo_sync_from_config(
        &RepoSyncQuery {
            repo_id: "managed-status".to_string(),
            mode: RepoSyncMode::Status,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert!(!managed_mirror_root(temp.path(), "managed-status").exists());
    assert!(
        !repo_cache_root(temp.path())
            .join("repos")
            .join(sanitize_repo_id("managed-status"))
            .exists()
    );
    assert_eq!(result.health_state, RepoSyncHealthState::MissingAssets);
    assert!(result.checked_at.contains('T'));
    assert!(result.last_fetched_at.is_none());
    assert_eq!(result.staleness_state, RepoSyncStalenessState::Unknown);
    assert!(result.status_summary.attention_required);
    assert!(!result.status_summary.lifecycle.checkout_ready);

    let mut payload = json!(result);
    redact_checkout_path(&mut payload);
    redact_mirror_path(&mut payload);
    redact_sync_timestamps(&mut payload);
    redact_upstream_url(&mut payload);
    redact_sync_revisions(&mut payload);
    assert_repo_json_snapshot("repo_sync_status_missing_managed_result", payload);
    Ok(())
}

#[test]
fn repo_sync_status_reports_ahead_when_managed_checkout_drifts_locally() -> TestResult {
    let temp = tempfile::tempdir()?;
    let source_repo = create_sample_julia_repo(temp.path(), "ManagedDriftPkg", true)?;
    clear_managed_repo_cache(temp.path(), "managed-drift")?;
    let config_path = write_repo_url_config(temp.path(), &source_repo, "managed-drift")?;

    repo_sync_from_config(
        &RepoSyncQuery {
            repo_id: "managed-drift".to_string(),
            mode: RepoSyncMode::Ensure,
        },
        Some(&config_path),
        temp.path(),
    )?;

    let managed_checkout = repo_cache_root(temp.path())
        .join("repos")
        .join(sanitize_repo_id("managed-drift"));
    append_repo_file_and_commit(
        &managed_checkout,
        "docs/local-drift.md",
        "# Local drift\n",
        "add local drift note",
    )?;

    let result = repo_sync_from_config(
        &RepoSyncQuery {
            repo_id: "managed-drift".to_string(),
            mode: RepoSyncMode::Status,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_eq!(result.drift_state, RepoSyncDriftState::Ahead);
    assert_eq!(result.health_state, RepoSyncHealthState::HasLocalCommits);
    assert_ne!(result.mirror_revision, result.revision);
    assert!(result.checked_at.contains('T'));
    assert!(result.last_fetched_at.is_some());
    assert_eq!(result.staleness_state, RepoSyncStalenessState::Fresh);
    assert!(result.status_summary.attention_required);
    assert!(!result.status_summary.revisions.aligned_with_mirror);

    let mut payload = json!(result);
    redact_checkout_path(&mut payload);
    redact_mirror_path(&mut payload);
    redact_sync_timestamps(&mut payload);
    redact_upstream_url(&mut payload);
    redact_sync_revisions(&mut payload);
    assert_repo_json_snapshot("repo_sync_status_ahead_managed_result", payload);
    Ok(())
}

#[test]
fn repo_sync_status_reports_behind_when_mirror_advances() -> TestResult {
    let temp = tempfile::tempdir()?;
    let source_repo = create_sample_julia_repo(temp.path(), "ManagedBehindPkg", true)?;
    clear_managed_repo_cache(temp.path(), "managed-behind")?;
    let config_path = write_repo_url_config(temp.path(), &source_repo, "managed-behind")?;

    repo_sync_from_config(
        &RepoSyncQuery {
            repo_id: "managed-behind".to_string(),
            mode: RepoSyncMode::Ensure,
        },
        Some(&config_path),
        temp.path(),
    )?;

    append_repo_file_and_commit(
        &source_repo,
        "docs/mirror-only.md",
        "# Mirror only\n",
        "advance source repo",
    )?;
    refresh_managed_mirror_only(temp.path(), "managed-behind")?;

    let result = repo_sync_from_config(
        &RepoSyncQuery {
            repo_id: "managed-behind".to_string(),
            mode: RepoSyncMode::Status,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_eq!(result.drift_state, RepoSyncDriftState::Behind);
    assert_eq!(result.health_state, RepoSyncHealthState::NeedsRefresh);
    assert_ne!(result.mirror_revision, result.revision);
    assert!(result.checked_at.contains('T'));
    assert!(result.last_fetched_at.is_some());
    assert_eq!(result.staleness_state, RepoSyncStalenessState::Fresh);
    Ok(())
}

#[test]
fn repo_sync_status_reports_diverged_when_mirror_and_checkout_both_move() -> TestResult {
    let temp = tempfile::tempdir()?;
    let source_repo = create_sample_julia_repo(temp.path(), "ManagedDivergedPkg", true)?;
    clear_managed_repo_cache(temp.path(), "managed-diverged")?;
    let config_path = write_repo_url_config(temp.path(), &source_repo, "managed-diverged")?;

    repo_sync_from_config(
        &RepoSyncQuery {
            repo_id: "managed-diverged".to_string(),
            mode: RepoSyncMode::Ensure,
        },
        Some(&config_path),
        temp.path(),
    )?;

    append_repo_file_and_commit(
        &source_repo,
        "docs/mirror-diverged.md",
        "# Mirror diverged\n",
        "advance source repo",
    )?;
    refresh_managed_mirror_only(temp.path(), "managed-diverged")?;

    let managed_checkout = repo_cache_root(temp.path())
        .join("repos")
        .join(sanitize_repo_id("managed-diverged"));
    append_repo_file_and_commit(
        &managed_checkout,
        "docs/local-diverged.md",
        "# Local diverged\n",
        "advance managed checkout",
    )?;

    let result = repo_sync_from_config(
        &RepoSyncQuery {
            repo_id: "managed-diverged".to_string(),
            mode: RepoSyncMode::Status,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_eq!(result.drift_state, RepoSyncDriftState::Diverged);
    assert_eq!(result.health_state, RepoSyncHealthState::Diverged);
    assert!(result.checked_at.contains('T'));
    assert!(result.last_fetched_at.is_some());
    assert_eq!(result.staleness_state, RepoSyncStalenessState::Fresh);
    Ok(())
}

#[test]
fn repo_sync_status_reports_stale_when_managed_mirror_fetch_is_old() -> TestResult {
    let temp = tempfile::tempdir()?;
    let source_repo = create_sample_julia_repo(temp.path(), "ManagedStalePkg", true)?;
    clear_managed_repo_cache(temp.path(), "managed-stale")?;
    let config_path = write_repo_url_config(temp.path(), &source_repo, "managed-stale")?;

    repo_sync_from_config(
        &RepoSyncQuery {
            repo_id: "managed-stale".to_string(),
            mode: RepoSyncMode::Ensure,
        },
        Some(&config_path),
        temp.path(),
    )?;

    set_managed_mirror_fetch_age(temp.path(), "managed-stale", Duration::from_hours(72))?;

    let result = repo_sync_from_config(
        &RepoSyncQuery {
            repo_id: "managed-stale".to_string(),
            mode: RepoSyncMode::Status,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_eq!(result.health_state, RepoSyncHealthState::Healthy);
    assert_eq!(result.staleness_state, RepoSyncStalenessState::Stale);
    assert!(result.last_fetched_at.is_some());
    assert!(result.status_summary.attention_required);

    let mut payload = json!(result);
    redact_checkout_path(&mut payload);
    redact_mirror_path(&mut payload);
    redact_sync_timestamps(&mut payload);
    redact_upstream_url(&mut payload);
    redact_sync_revisions(&mut payload);
    assert_repo_json_snapshot("repo_sync_status_stale_managed_result", payload);
    Ok(())
}

#[test]
fn cli_repo_sync_returns_serialized_result() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_julia_repo(temp.path(), "CliSyncPkg", true)?;
    let config_path = write_repo_config(temp.path(), &repo_dir, "cli-sync")?;

    let output = wendao_command()
        .arg("--conf")
        .arg(&config_path)
        .arg("--output")
        .arg("json")
        .arg("repo")
        .arg("sync")
        .arg("--repo")
        .arg("cli-sync")
        .output()?;

    assert!(output.status.success(), "{output:?}");

    let mut payload: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    redact_checkout_path(&mut payload);
    redact_sync_timestamps(&mut payload);
    redact_sync_revisions(&mut payload);
    assert_repo_json_snapshot("repo_sync_cli_json", payload);
    Ok(())
}

#[test]
fn repo_sync_refresh_mode_overrides_manual_policy() -> TestResult {
    let temp = tempfile::tempdir()?;
    let source_repo = create_sample_julia_repo(temp.path(), "ManagedForceSyncPkg", true)?;
    let repo_id = format!("managed-force-sync-{}", Uuid::new_v4());
    clear_managed_repo_cache(temp.path(), repo_id.as_str())?;
    let config_path =
        write_repo_url_config_with_refresh(temp.path(), &source_repo, repo_id.as_str(), "manual")?;

    let first = repo_sync_from_config(
        &RepoSyncQuery {
            repo_id: repo_id.clone(),
            mode: RepoSyncMode::Ensure,
        },
        Some(&config_path),
        temp.path(),
    )?;

    append_repo_file_and_commit(
        &source_repo,
        "docs/forced-refresh.md",
        "# Forced refresh\n",
        "add forced refresh guide",
    )?;

    let second = repo_sync_from_config(
        &RepoSyncQuery {
            repo_id: repo_id.clone(),
            mode: RepoSyncMode::Ensure,
        },
        Some(&config_path),
        temp.path(),
    )?;
    let refreshed = repo_sync_from_config(
        &RepoSyncQuery {
            repo_id,
            mode: RepoSyncMode::Refresh,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_eq!(first.revision, second.revision);
    assert_ne!(second.revision, refreshed.revision);
    assert_eq!(refreshed.mode, RepoSyncMode::Refresh);
    assert_eq!(refreshed.health_state, RepoSyncHealthState::Healthy);
    assert!(refreshed.checked_at.contains('T'));
    assert!(refreshed.last_fetched_at.is_some());
    assert_eq!(refreshed.staleness_state, RepoSyncStalenessState::Fresh);
    Ok(())
}

fn redact_checkout_path(value: &mut serde_json::Value) {
    if let Some(path) = value.pointer_mut("/checkout_path") {
        *path = serde_json::Value::String("[checkout-path]".to_string());
    }
}

fn redact_mirror_path(value: &mut serde_json::Value) {
    if let Some(path) = value.pointer_mut("/mirror_path") {
        *path = serde_json::Value::String("[mirror-path]".to_string());
    }
}

fn redact_sync_timestamps(value: &mut serde_json::Value) {
    if let Some(path) = value.pointer_mut("/checked_at") {
        *path = serde_json::Value::String("[checked-at]".to_string());
    }
    if let Some(path) = value.pointer_mut("/last_fetched_at") {
        *path = match path {
            serde_json::Value::Null => serde_json::Value::Null,
            _ => serde_json::Value::String("[last-fetched-at]".to_string()),
        };
    }
    if let Some(path) = value.pointer_mut("/status_summary/freshness/checked_at") {
        *path = serde_json::Value::String("[checked-at]".to_string());
    }
    if let Some(path) = value.pointer_mut("/status_summary/freshness/last_fetched_at") {
        *path = match path {
            serde_json::Value::Null => serde_json::Value::Null,
            _ => serde_json::Value::String("[last-fetched-at]".to_string()),
        };
    }
}

fn redact_upstream_url(value: &mut serde_json::Value) {
    if let Some(url) = value.pointer_mut("/upstream_url") {
        *url = serde_json::Value::String("[upstream-url]".to_string());
    }
}

fn redact_sync_revisions(value: &mut serde_json::Value) {
    for pointer in [
        "/revision",
        "/mirror_revision",
        "/tracking_revision",
        "/status_summary/revisions/checkout_revision",
        "/status_summary/revisions/mirror_revision",
        "/status_summary/revisions/tracking_revision",
    ] {
        if let Some(revision) = value.pointer_mut(pointer)
            && !revision.is_null()
        {
            *revision = serde_json::Value::String("[revision]".to_string());
        }
    }
}

fn write_repo_url_config(
    base: &Path,
    repo_url: &Path,
    repo_id: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    write_repo_url_config_with_refresh(base, repo_url, repo_id, "fetch")
}

fn write_repo_url_config_with_refresh(
    base: &Path,
    repo_url: &Path,
    repo_id: &str,
    refresh: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let config_path = base.join(format!("{repo_id}.wendao.toml"));
    fs::write(
        &config_path,
        format!(
            r#"[link_graph.projects.{repo_id}]
url = "{}"
refresh = "{refresh}"
plugins = ["julia"]
"#,
            repo_url.display(),
            refresh = refresh
        ),
    )?;
    Ok(config_path)
}

fn append_repo_file_and_commit(
    repo_dir: &Path,
    relative_path: &str,
    contents: &str,
    message: &str,
) -> TestResult {
    crate::support::repo_fixture::append_repo_file_and_commit(
        repo_dir,
        relative_path,
        contents,
        message,
    )
}

fn repo_cache_root(_cwd: &Path) -> std::path::PathBuf {
    PrjDirs::data_home()
        .join("xiuxian-wendao")
        .join("repo-intelligence")
}

fn managed_mirror_root(cwd: &Path, repo_id: &str) -> std::path::PathBuf {
    repo_cache_root(cwd)
        .join("mirrors")
        .join(format!("{}.git", sanitize_repo_id(repo_id)))
}

fn clear_managed_repo_cache(cwd: &Path, repo_id: &str) -> TestResult {
    let mirror_root = managed_mirror_root(cwd, repo_id);
    if mirror_root.exists() {
        fs::remove_dir_all(&mirror_root)?;
    }

    let checkout_root = repo_cache_root(cwd)
        .join("repos")
        .join(sanitize_repo_id(repo_id));
    if checkout_root.exists() {
        fs::remove_dir_all(checkout_root)?;
    }
    Ok(())
}

fn sanitize_repo_id(repo_id: &str) -> String {
    repo_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn refresh_managed_mirror_only(cwd: &Path, repo_id: &str) -> TestResult {
    let mirror_root = managed_mirror_root(cwd, repo_id);
    refresh_remote(&mirror_root, "origin")
}

fn set_managed_mirror_fetch_age(cwd: &Path, repo_id: &str, age: Duration) -> TestResult {
    let mirror_root = managed_mirror_root(cwd, repo_id);
    let target_time = SystemTime::now()
        .checked_sub(age)
        .ok_or_else(|| "failed to compute mirror age timestamp".to_string())?;

    for candidate in [mirror_root.join("FETCH_HEAD"), mirror_root.join("HEAD")] {
        if candidate.exists() {
            let file = fs::OpenOptions::new().write(true).open(&candidate)?;
            let times = fs::FileTimes::new().set_modified(target_time);
            file.set_times(times)?;
        }
    }

    Ok(())
}
