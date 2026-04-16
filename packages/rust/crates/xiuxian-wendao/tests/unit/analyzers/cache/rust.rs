use std::fs;

use xiuxian_git_repo::{
    LocalCheckoutMetadata, MaterializedRepo, RepoDriftState, RepoLifecycleState, RepoSourceKind,
};

use crate::analyzers::{RegisteredRepository, RepositoryPluginConfig, RepositoryRefreshPolicy};

use super::super::build_repository_analysis_cache_key;

#[test]
fn build_repository_analysis_cache_key_reuses_generic_rust_identity_for_ast_equivalent_source_churn()
 {
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    fs::create_dir_all(tempdir.path().join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    let source_path = tempdir.path().join("src/lib.rs");
    fs::write(&source_path, "fn solve(x: i32) -> i32 {\n    x + 1\n}\n")
        .unwrap_or_else(|error| panic!("write Rust source: {error}"));

    let repository = RegisteredRepository {
        id: "repo-cache-identity-rust-semantic".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("rust".to_string())],
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
        "fn solve(x: i32) -> i32 {\n    // semantic no-op\n    x + 1\n}\n",
    )
    .unwrap_or_else(|error| panic!("rewrite Rust source: {error}"));
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
fn build_repository_analysis_cache_key_invalidates_on_generic_rust_signature_change() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    fs::create_dir_all(tempdir.path().join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    let source_path = tempdir.path().join("src/lib.rs");
    fs::write(&source_path, "fn solve(x: i32) -> i32 {\n    x + 1\n}\n")
        .unwrap_or_else(|error| panic!("write Rust source: {error}"));

    let repository = RegisteredRepository {
        id: "repo-cache-identity-rust-change".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("rust".to_string())],
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
        "fn solve(x: i32, y: i32) -> i32 {\n    x + y\n}\n",
    )
    .unwrap_or_else(|error| panic!("rewrite Rust source: {error}"));
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

#[test]
fn build_repository_analysis_cache_key_reuses_generic_rust_identity_without_repo_plugin_config() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    fs::create_dir_all(tempdir.path().join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    let source_path = tempdir.path().join("src/lib.rs");
    fs::write(&source_path, "fn solve(x: i32) -> i32 {\n    x + 1\n}\n")
        .unwrap_or_else(|error| panic!("write Rust source: {error}"));

    let repository = RegisteredRepository {
        id: "repo-cache-identity-rust-generic-no-plugin".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: Vec::new(),
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
        "fn solve(x: i32) -> i32 {\n    // semantic no-op\n    x + 1\n}\n",
    )
    .unwrap_or_else(|error| panic!("rewrite Rust source: {error}"));
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
fn build_repository_analysis_cache_key_invalidates_generic_rust_identity_without_repo_plugin_config()
 {
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    fs::create_dir_all(tempdir.path().join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    let source_path = tempdir.path().join("src/lib.rs");
    fs::write(&source_path, "fn solve(x: i32) -> i32 {\n    x + 1\n}\n")
        .unwrap_or_else(|error| panic!("write Rust source: {error}"));

    let repository = RegisteredRepository {
        id: "repo-cache-identity-rust-generic-no-plugin-change".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: Vec::new(),
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
        "fn solve(x: i32, y: i32) -> i32 {\n    x + y\n}\n",
    )
    .unwrap_or_else(|error| panic!("rewrite Rust source: {error}"));
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
