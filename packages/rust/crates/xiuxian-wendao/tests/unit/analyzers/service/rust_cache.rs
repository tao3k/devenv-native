use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::analyzers::PluginRegistry;
use crate::analyzers::RepositoryPluginConfig;
use crate::analyzers::service::analysis::analyze_registered_repository_bundle_with_registry;
use crate::analyzers::{RegisteredRepository, RepositoryRefreshPolicy};
use crate::gateway::studio::test_support::{commit_all, init_git_repository};

use super::support::CountingRustPlugin;

#[test]
fn analyze_repository_reuses_cached_analysis_for_generic_rust_ast_equivalent_source_churn() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    init_git_repository(tempdir.path());
    fs::create_dir_all(tempdir.path().join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    let source_path = tempdir.path().join("src/lib.rs");
    fs::write(&source_path, "fn solve(x: i32) -> i32 {\n    x + 1\n}\n")
        .unwrap_or_else(|error| panic!("write Rust source: {error}"));
    commit_all(tempdir.path(), "initial");

    let repository = RegisteredRepository {
        id: "counting-rust-cache".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        git_ref: None,
        plugins: vec![RepositoryPluginConfig::Id("rust".to_string())],
    };
    let calls = Arc::new(AtomicUsize::new(0));
    let mut registry = PluginRegistry::new();
    registry
        .register(CountingRustPlugin {
            calls: Arc::clone(&calls),
        })
        .unwrap_or_else(|error| panic!("register test plugin: {error}"));

    let first =
        analyze_registered_repository_bundle_with_registry(&repository, tempdir.path(), &registry)
            .unwrap_or_else(|error| panic!("first analysis should succeed: {error}"));
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    fs::write(
        &source_path,
        "fn solve(x: i32) -> i32 {\n    // semantic no-op\n    x + 1\n}\n",
    )
    .unwrap_or_else(|error| panic!("rewrite Rust source: {error}"));
    commit_all(tempdir.path(), "ast-equivalent");

    let second =
        analyze_registered_repository_bundle_with_registry(&repository, tempdir.path(), &registry)
            .unwrap_or_else(|error| panic!("second analysis should succeed: {error}"));

    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        first.cache_key.analysis_identity,
        second.cache_key.analysis_identity
    );
}

#[test]
fn analyze_repository_invalidates_cached_analysis_for_generic_rust_signature_change() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    init_git_repository(tempdir.path());
    fs::create_dir_all(tempdir.path().join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    let source_path = tempdir.path().join("src/lib.rs");
    fs::write(&source_path, "fn solve(x: i32) -> i32 {\n    x + 1\n}\n")
        .unwrap_or_else(|error| panic!("write Rust source: {error}"));
    commit_all(tempdir.path(), "initial");

    let repository = RegisteredRepository {
        id: "counting-rust-cache-change".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        git_ref: None,
        plugins: vec![RepositoryPluginConfig::Id("rust".to_string())],
    };
    let calls = Arc::new(AtomicUsize::new(0));
    let mut registry = PluginRegistry::new();
    registry
        .register(CountingRustPlugin {
            calls: Arc::clone(&calls),
        })
        .unwrap_or_else(|error| panic!("register test plugin: {error}"));

    let first =
        analyze_registered_repository_bundle_with_registry(&repository, tempdir.path(), &registry)
            .unwrap_or_else(|error| panic!("first analysis should succeed: {error}"));
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    fs::write(
        &source_path,
        "fn solve(x: i32, y: i32) -> i32 {\n    x + y\n}\n",
    )
    .unwrap_or_else(|error| panic!("rewrite Rust source: {error}"));
    commit_all(tempdir.path(), "affecting");

    let second =
        analyze_registered_repository_bundle_with_registry(&repository, tempdir.path(), &registry)
            .unwrap_or_else(|error| panic!("second analysis should succeed: {error}"));

    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_ne!(
        first.cache_key.analysis_identity,
        second.cache_key.analysis_identity
    );
}
