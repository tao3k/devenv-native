use std::fs;

use uuid::Uuid;
use xiuxian_config_core::ProjectDirs;
use xiuxian_git_repo::{
    RepoLifecycleState, RepoRefreshPolicy, RepoSourceKind, RepoSpec, RevisionSelector, SyncMode,
    discover_checkout_metadata, discover_managed_remote_probe_state, managed_checkout_root_for,
    resolve_repository_source,
};

use crate::support::{
    append_repo_file_and_commit, create_annotated_tag, create_branch_and_commit,
    init_test_repository, must, must_some, set_repo_remote_url, temp_dir,
};

#[test]
fn resolve_repository_source_materializes_remote_checkout_under_prj_data_home() {
    let source = temp_dir();
    let cwd = temp_cwd();
    init_test_repository(source.path());
    let repo_id = format!("checkout-test-{}", Uuid::new_v4());

    let spec = RepoSpec {
        id: repo_id.clone(),
        local_path: None,
        remote_url: Some(source.path().display().to_string()),
        revision: None,
        refresh: RepoRefreshPolicy::Manual,
    };
    let target_root = managed_checkout_root_for(&spec);
    if target_root.exists() {
        must(fs::remove_dir_all(&target_root), "cleanup stale checkout");
    }

    let resolved = must(
        resolve_repository_source(&spec, cwd.path(), SyncMode::Ensure),
        "resolve managed checkout",
    );

    assert!(resolved.checkout_root.starts_with(ProjectDirs::data_home()));
    assert!(resolved.checkout_root.is_dir());
    assert_eq!(resolved.source_kind, RepoSourceKind::ManagedRemote);
    assert!(resolved.tracking_revision.is_some());
    let metadata = must_some(
        discover_checkout_metadata(&resolved.checkout_root),
        "discover checkout metadata",
    );
    let mirror_root = must_some(
        resolved.mirror_root.clone(),
        "managed checkout should expose mirror root",
    );
    assert_eq!(
        metadata.remote_url.as_deref(),
        Some(
            std::fs::canonicalize(&mirror_root)
                .unwrap_or_else(|_| mirror_root.clone())
                .display()
                .to_string()
                .as_str(),
        )
    );
    let mirror_metadata = must_some(
        discover_checkout_metadata(mirror_root.as_path()),
        "discover managed mirror metadata",
    );
    assert_eq!(
        mirror_metadata.remote_url.as_deref(),
        Some(canonical_display_path(source.path()).as_str())
    );

    must(fs::remove_dir_all(&mirror_root), "cleanup managed mirror");
    must(
        fs::remove_dir_all(&resolved.checkout_root),
        "cleanup managed checkout",
    );
}

#[test]
fn resolve_repository_source_reuses_fetch_policy_checkout_when_revision_is_unchanged() {
    let source = temp_dir();
    let cwd = temp_cwd();
    init_test_repository(source.path());
    let repo_id = format!("checkout-reuse-test-{}", Uuid::new_v4());
    let spec = RepoSpec {
        id: repo_id,
        local_path: None,
        remote_url: Some(source.path().display().to_string()),
        revision: None,
        refresh: RepoRefreshPolicy::Fetch,
    };

    let first = must(
        resolve_repository_source(&spec, cwd.path(), SyncMode::Ensure),
        "resolve first ensure",
    );
    let first_revision = must_some(
        discover_checkout_metadata(&first.checkout_root),
        "discover first checkout metadata",
    )
    .revision;
    let second = must(
        resolve_repository_source(&spec, cwd.path(), SyncMode::Ensure),
        "resolve second ensure",
    );
    let second_metadata = must_some(
        discover_checkout_metadata(&second.checkout_root),
        "discover second checkout metadata",
    );
    let mirror_root = must_some(second.mirror_root.clone(), "mirror root");

    assert_eq!(first_revision, second_metadata.revision);
    assert_eq!(second.mirror_state, RepoLifecycleState::Reused);
    assert_eq!(second.checkout_state, RepoLifecycleState::Reused);
    assert_eq!(
        discover_managed_remote_probe_state(&mirror_root).and_then(|state| state.target_revision),
        second_metadata.revision
    );

    must(fs::remove_dir_all(&mirror_root), "cleanup managed mirror");
    must(
        fs::remove_dir_all(&second.checkout_root),
        "cleanup managed checkout",
    );
}

#[test]
fn resolve_repository_source_reuses_branch_pinned_fetch_policy_checkout_when_revision_is_unchanged()
{
    let source = temp_dir();
    let cwd = temp_cwd();
    init_test_repository(source.path());
    create_branch_and_commit(
        source.path(),
        "release",
        "src/release.jl",
        "const RELEASE = true\n",
        "release branch commit",
    );
    let repo_id = format!("checkout-branch-reuse-test-{}", Uuid::new_v4());
    let spec = RepoSpec {
        id: repo_id,
        local_path: None,
        remote_url: Some(source.path().display().to_string()),
        revision: Some(RevisionSelector::Branch("release".to_string())),
        refresh: RepoRefreshPolicy::Fetch,
    };

    let first = must(
        resolve_repository_source(&spec, cwd.path(), SyncMode::Ensure),
        "resolve first ensure",
    );
    let first_revision = must_some(
        discover_checkout_metadata(&first.checkout_root),
        "discover first checkout metadata",
    )
    .revision;
    let second = must(
        resolve_repository_source(&spec, cwd.path(), SyncMode::Ensure),
        "resolve second ensure",
    );
    let second_metadata = must_some(
        discover_checkout_metadata(&second.checkout_root),
        "discover second checkout metadata",
    );
    let mirror_root = must_some(second.mirror_root.clone(), "mirror root");

    assert_eq!(first_revision, second_metadata.revision);
    assert_eq!(second.mirror_state, RepoLifecycleState::Reused);
    assert_eq!(second.checkout_state, RepoLifecycleState::Reused);
    assert_eq!(
        discover_managed_remote_probe_state(&mirror_root).and_then(|state| state.target_revision),
        second_metadata.revision
    );

    must(fs::remove_dir_all(&mirror_root), "cleanup managed mirror");
    must(
        fs::remove_dir_all(&second.checkout_root),
        "cleanup managed checkout",
    );
}

#[test]
fn resolve_repository_source_reuses_tag_pinned_fetch_policy_checkout_when_revision_is_unchanged() {
    let source = temp_dir();
    let cwd = temp_cwd();
    init_test_repository(source.path());
    create_annotated_tag(source.path(), "v1.0.0", "release tag");
    let repo_id = format!("checkout-tag-reuse-test-{}", Uuid::new_v4());
    let spec = RepoSpec {
        id: repo_id,
        local_path: None,
        remote_url: Some(source.path().display().to_string()),
        revision: Some(RevisionSelector::Tag("v1.0.0".to_string())),
        refresh: RepoRefreshPolicy::Fetch,
    };

    let first = must(
        resolve_repository_source(&spec, cwd.path(), SyncMode::Ensure),
        "resolve first ensure",
    );
    let first_revision = must_some(
        discover_checkout_metadata(&first.checkout_root),
        "discover first checkout metadata",
    )
    .revision;
    let second = must(
        resolve_repository_source(&spec, cwd.path(), SyncMode::Ensure),
        "resolve second ensure",
    );
    let second_metadata = must_some(
        discover_checkout_metadata(&second.checkout_root),
        "discover second checkout metadata",
    );
    let mirror_root = must_some(second.mirror_root.clone(), "mirror root");

    assert_eq!(first_revision, second_metadata.revision);
    assert_eq!(second.mirror_state, RepoLifecycleState::Reused);
    assert_eq!(second.checkout_state, RepoLifecycleState::Reused);
    assert_eq!(
        discover_managed_remote_probe_state(&mirror_root).and_then(|state| state.target_revision),
        second_metadata.revision
    );

    must(fs::remove_dir_all(&mirror_root), "cleanup managed mirror");
    must(
        fs::remove_dir_all(&second.checkout_root),
        "cleanup managed checkout",
    );
}

#[test]
fn resolve_repository_source_refreshes_fetch_policy_checkout_when_remote_revision_advances() {
    let source = temp_dir();
    let cwd = temp_cwd();
    init_test_repository(source.path());
    let repo_id = format!("checkout-refresh-test-{}", Uuid::new_v4());
    let spec = RepoSpec {
        id: repo_id,
        local_path: None,
        remote_url: Some(source.path().display().to_string()),
        revision: None,
        refresh: RepoRefreshPolicy::Fetch,
    };

    let first = must(
        resolve_repository_source(&spec, cwd.path(), SyncMode::Ensure),
        "resolve first ensure",
    );
    let first_revision = must_some(
        discover_checkout_metadata(&first.checkout_root),
        "discover first checkout metadata",
    )
    .revision;
    append_repo_file_and_commit(
        source.path(),
        "src/advanced.jl",
        "const ADVANCED = true\n",
        "advance source",
    );

    let second = must(
        resolve_repository_source(&spec, cwd.path(), SyncMode::Ensure),
        "resolve second ensure after advance",
    );
    let second_metadata = must_some(
        discover_checkout_metadata(&second.checkout_root),
        "discover second checkout metadata",
    );
    let mirror_root = must_some(second.mirror_root.clone(), "mirror root");

    assert_ne!(first_revision, second_metadata.revision);
    assert_eq!(second.mirror_state, RepoLifecycleState::Refreshed);
    assert_eq!(second.checkout_state, RepoLifecycleState::Refreshed);

    must(fs::remove_dir_all(&mirror_root), "cleanup managed mirror");
    must(
        fs::remove_dir_all(&second.checkout_root),
        "cleanup managed checkout",
    );
}

#[test]
fn resolve_repository_source_reuses_commit_pinned_fetch_policy_remote() {
    let source = temp_dir();
    let cwd = temp_cwd();
    init_test_repository(source.path());
    let initial_revision = must_some(
        must_some(
            discover_checkout_metadata(source.path()),
            "discover source metadata",
        )
        .revision,
        "source revision",
    );
    let repo_id = format!("checkout-commit-pin-test-{}", Uuid::new_v4());
    let spec = RepoSpec {
        id: repo_id,
        local_path: None,
        remote_url: Some(source.path().display().to_string()),
        revision: Some(RevisionSelector::Commit(initial_revision.clone())),
        refresh: RepoRefreshPolicy::Fetch,
    };

    let first = must(
        resolve_repository_source(&spec, cwd.path(), SyncMode::Ensure),
        "resolve first ensure",
    );
    append_repo_file_and_commit(
        source.path(),
        "src/future.jl",
        "const FUTURE = true\n",
        "advance source branch without changing pinned commit",
    );

    let second = must(
        resolve_repository_source(&spec, cwd.path(), SyncMode::Ensure),
        "resolve second ensure for pinned commit",
    );
    let second_metadata = must_some(
        discover_checkout_metadata(&second.checkout_root),
        "discover second checkout metadata",
    );
    let mirror_root = must_some(second.mirror_root.clone(), "mirror root");

    assert_eq!(
        second_metadata.revision.as_deref(),
        Some(initial_revision.as_str())
    );
    assert_eq!(second.mirror_state, RepoLifecycleState::Reused);
    assert_eq!(second.checkout_state, RepoLifecycleState::Reused);

    must(fs::remove_dir_all(&mirror_root), "cleanup managed mirror");
    must(
        fs::remove_dir_all(&second.checkout_root),
        "cleanup managed checkout",
    );
    fs::remove_dir_all(&first.checkout_root).ok();
}

#[test]
fn resolve_repository_source_realigns_existing_remote_urls_after_drift() {
    let source = temp_dir();
    init_test_repository(source.path());
    let wrong_remote = temp_dir();
    let cwd = temp_cwd();
    init_test_repository(wrong_remote.path());
    let repo_id = format!("checkout-remote-realign-test-{}", Uuid::new_v4());
    let spec = RepoSpec {
        id: repo_id,
        local_path: None,
        remote_url: Some(source.path().display().to_string()),
        revision: None,
        refresh: RepoRefreshPolicy::Fetch,
    };

    let first = must(
        resolve_repository_source(&spec, cwd.path(), SyncMode::Ensure),
        "resolve first ensure",
    );
    let mirror_root = must_some(first.mirror_root.clone(), "mirror root");
    let wrong_remote_url = wrong_remote.path().display().to_string();
    set_repo_remote_url(&mirror_root, "origin", wrong_remote_url.as_str());
    set_repo_remote_url(&first.checkout_root, "origin", wrong_remote_url.as_str());

    let second = must(
        resolve_repository_source(&spec, cwd.path(), SyncMode::Ensure),
        "resolve second ensure after remote drift",
    );

    let mirror_metadata = must_some(
        discover_checkout_metadata(&mirror_root),
        "discover managed mirror metadata",
    );
    let checkout_metadata = must_some(
        discover_checkout_metadata(&second.checkout_root),
        "discover checkout metadata",
    );
    let expected_checkout_origin = std::fs::canonicalize(&mirror_root)
        .unwrap_or_else(|_| mirror_root.clone())
        .display()
        .to_string();

    assert_eq!(
        mirror_metadata.remote_url.as_deref(),
        Some(canonical_display_path(source.path()).as_str())
    );
    assert_eq!(
        checkout_metadata.remote_url.as_deref(),
        Some(expected_checkout_origin.as_str())
    );

    must(fs::remove_dir_all(mirror_root), "cleanup managed mirror");
    must(
        fs::remove_dir_all(&second.checkout_root),
        "cleanup managed checkout",
    );
}

fn temp_cwd() -> tempfile::TempDir {
    temp_dir()
}

fn canonical_display_path(path: &std::path::Path) -> String {
    std::fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}
