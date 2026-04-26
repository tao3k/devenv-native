use std::fs;

use xiuxian_git_repo::{
    LocalCheckoutMetadata, MaterializedRepo, RepoDriftState, RepoLifecycleState, RepoSourceKind,
};

use crate::analyzers::{RegisteredRepository, RepositoryPluginConfig, RepositoryRefreshPolicy};

use super::super::build_repository_analysis_cache_key;

#[test]
fn build_repository_analysis_cache_key_reuses_julia_identity_for_non_affecting_churn() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    fs::write(
        tempdir.path().join("Project.toml"),
        "name = \"CacheKeyDemo\"\n",
    )
    .unwrap_or_else(|error| panic!("write Project.toml: {error}"));
    fs::create_dir_all(tempdir.path().join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    fs::write(
        tempdir.path().join("src/CacheKeyDemo.jl"),
        "module CacheKeyDemo\nend\n",
    )
    .unwrap_or_else(|error| panic!("write Julia source: {error}"));
    fs::write(tempdir.path().join("notes.txt"), "first note\n")
        .unwrap_or_else(|error| panic!("write notes: {error}"));

    let repository = RegisteredRepository {
        id: "repo-cache-identity".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("julia".to_string())],
    };
    let source = MaterializedRepo {
        checkout_root: tempdir.path().to_path_buf(),
        mirror_root: None,
        mirror_revision: Some("mirror-1".to_string()),
        tracking_revision: Some("tracking-1".to_string()),
        last_fetched_at: None,
        drift_state: RepoDriftState::NotApplicable,
        mirror_state: RepoLifecycleState::NotApplicable,
        checkout_state: RepoLifecycleState::Validated,
        source_kind: RepoSourceKind::LocalCheckout,
    };
    let first_metadata = Some(LocalCheckoutMetadata {
        revision: Some("rev-1".to_string()),
        remote_url: None,
    });
    let first_key =
        build_repository_analysis_cache_key(&repository, &source, first_metadata.as_ref());

    fs::write(
        tempdir.path().join("notes.txt"),
        "second note that should stay non-affecting\n",
    )
    .unwrap_or_else(|error| panic!("rewrite notes: {error}"));
    let second_metadata = Some(LocalCheckoutMetadata {
        revision: Some("rev-2".to_string()),
        remote_url: None,
    });
    let second_key =
        build_repository_analysis_cache_key(&repository, &source, second_metadata.as_ref());

    assert_eq!(first_key.analysis_identity, second_key.analysis_identity);
    assert_eq!(first_key, second_key);
    assert_ne!(first_key.checkout_revision, second_key.checkout_revision);
}

#[test]
fn build_repository_analysis_cache_key_reuses_julia_identity_for_ast_equivalent_source_churn() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    fs::write(
        tempdir.path().join("Project.toml"),
        "name = \"CacheKeyDemo\"\n",
    )
    .unwrap_or_else(|error| panic!("write Project.toml: {error}"));
    fs::create_dir_all(tempdir.path().join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    let source_path = tempdir.path().join("src/CacheKeyDemo.jl");
    fs::write(&source_path, "module CacheKeyDemo\nalpha() = 1\nend\n")
        .unwrap_or_else(|error| panic!("write Julia source: {error}"));

    let repository = RegisteredRepository {
        id: "repo-cache-identity-semantic".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("julia".to_string())],
    };
    let source = MaterializedRepo {
        checkout_root: tempdir.path().to_path_buf(),
        mirror_root: None,
        mirror_revision: Some("mirror-1".to_string()),
        tracking_revision: Some("tracking-1".to_string()),
        last_fetched_at: None,
        drift_state: RepoDriftState::NotApplicable,
        mirror_state: RepoLifecycleState::NotApplicable,
        checkout_state: RepoLifecycleState::Validated,
        source_kind: RepoSourceKind::LocalCheckout,
    };
    let first_key = build_repository_analysis_cache_key(
        &repository,
        &source,
        Some(&LocalCheckoutMetadata {
            revision: Some("rev-1".to_string()),
            remote_url: None,
        }),
    );

    fs::write(
        &source_path,
        "module CacheKeyDemo\nalpha() = 1\n# semantic no-op\nend\n",
    )
    .unwrap_or_else(|error| panic!("rewrite Julia source: {error}"));
    let second_key = build_repository_analysis_cache_key(
        &repository,
        &source,
        Some(&LocalCheckoutMetadata {
            revision: Some("rev-2".to_string()),
            remote_url: None,
        }),
    );

    assert_eq!(first_key.analysis_identity, second_key.analysis_identity);
}

#[test]
fn build_repository_analysis_cache_key_invalidates_on_julia_source_change() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    fs::write(
        tempdir.path().join("Project.toml"),
        "name = \"CacheKeyDemo\"\n",
    )
    .unwrap_or_else(|error| panic!("write Project.toml: {error}"));
    fs::create_dir_all(tempdir.path().join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    let source_path = tempdir.path().join("src/CacheKeyDemo.jl");
    fs::write(&source_path, "module CacheKeyDemo\nend\n")
        .unwrap_or_else(|error| panic!("write Julia source: {error}"));

    let repository = RegisteredRepository {
        id: "repo-cache-identity-change".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("julia".to_string())],
    };
    let source = MaterializedRepo {
        checkout_root: tempdir.path().to_path_buf(),
        mirror_root: None,
        mirror_revision: Some("mirror-1".to_string()),
        tracking_revision: Some("tracking-1".to_string()),
        last_fetched_at: None,
        drift_state: RepoDriftState::NotApplicable,
        mirror_state: RepoLifecycleState::NotApplicable,
        checkout_state: RepoLifecycleState::Validated,
        source_kind: RepoSourceKind::LocalCheckout,
    };
    let first_key = build_repository_analysis_cache_key(
        &repository,
        &source,
        Some(&LocalCheckoutMetadata {
            revision: Some("rev-1".to_string()),
            remote_url: None,
        }),
    );

    fs::write(
        &source_path,
        "module CacheKeyDemo\nconst CACHE_KEY_VERSION = 2\nend\n",
    )
    .unwrap_or_else(|error| panic!("rewrite Julia source: {error}"));
    let second_key = build_repository_analysis_cache_key(
        &repository,
        &source,
        Some(&LocalCheckoutMetadata {
            revision: Some("rev-2".to_string()),
            remote_url: None,
        }),
    );

    assert_ne!(first_key.analysis_identity, second_key.analysis_identity);
}
