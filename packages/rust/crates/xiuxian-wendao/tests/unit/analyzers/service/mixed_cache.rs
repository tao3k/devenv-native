use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::analyzers::PluginRegistry;
use crate::analyzers::service::analysis::analyze_registered_repository_bundle_with_registry;
use crate::analyzers::{RegisteredRepository, RepositoryRefreshPolicy};
use crate::gateway::studio::search::handlers::tests::linked_parser_summary::ensure_linked_modelica_parser_summary_service;
use crate::gateway::studio::test_support::{commit_all, init_git_repository};

use super::support::{
    CountingModelicaPlugin, CountingRustPlugin,
    bootstrap_builtin_registry_with_counting_rust_plugin, mixed_modelica_rust_plugin_configs,
    mixed_modelica_unknown_plugin_configs, mixed_rust_unknown_plugin_configs,
};

#[test]
fn analyze_repository_reuses_cached_analysis_for_ast_equivalent_mixed_modelica_rust_rust_source_churn()
 {
    ensure_linked_modelica_parser_summary_service()
        .unwrap_or_else(|error| panic!("linked Modelica parser-summary service: {error}"));
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    init_git_repository(tempdir.path());
    fs::create_dir_all(tempdir.path().join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    let rust_source_path = tempdir.path().join("src/lib.rs");
    fs::write(
        &rust_source_path,
        "fn solve(x: i32) -> i32 {\n    x + 1\n}\n",
    )
    .unwrap_or_else(|error| panic!("write Rust source: {error}"));
    fs::write(
        tempdir.path().join("package.mo"),
        "within ;\npackage DemoLib\nend DemoLib;\n",
    )
    .unwrap_or_else(|error| panic!("write root package: {error}"));
    fs::write(
        tempdir.path().join("PI.mo"),
        "within DemoLib;\nmodel PI\n  parameter Real k = 1;\nend PI;\n",
    )
    .unwrap_or_else(|error| panic!("write Modelica source: {error}"));
    commit_all(tempdir.path(), "initial");

    let repository = RegisteredRepository {
        id: "counting-mixed-modelica-rust-rust".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        git_ref: None,
        plugins: mixed_modelica_rust_plugin_configs(),
    };
    let calls = Arc::new(AtomicUsize::new(0));
    let registry = bootstrap_builtin_registry_with_counting_rust_plugin(Arc::clone(&calls));

    let first =
        analyze_registered_repository_bundle_with_registry(&repository, tempdir.path(), &registry)
            .unwrap_or_else(|error| panic!("first mixed analysis should succeed: {error}"));
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    fs::write(
        &rust_source_path,
        "fn solve(x: i32) -> i32 {\n    // semantic no-op\n    x + 1\n}\n",
    )
    .unwrap_or_else(|error| panic!("rewrite Rust source: {error}"));
    commit_all(tempdir.path(), "ast-equivalent mixed rust");

    let second =
        analyze_registered_repository_bundle_with_registry(&repository, tempdir.path(), &registry)
            .unwrap_or_else(|error| panic!("second mixed analysis should succeed: {error}"));

    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        first.cache_key.analysis_identity,
        second.cache_key.analysis_identity
    );
}

#[test]
fn analyze_repository_reuses_cached_analysis_for_ast_equivalent_mixed_modelica_rust_modelica_source_churn()
 {
    ensure_linked_modelica_parser_summary_service()
        .unwrap_or_else(|error| panic!("linked Modelica parser-summary service: {error}"));
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    init_git_repository(tempdir.path());
    fs::create_dir_all(tempdir.path().join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    fs::write(
        tempdir.path().join("src/lib.rs"),
        "fn solve(x: i32) -> i32 {\n    x + 1\n}\n",
    )
    .unwrap_or_else(|error| panic!("write Rust source: {error}"));
    fs::write(
        tempdir.path().join("package.mo"),
        "within ;\npackage DemoLib\nend DemoLib;\n",
    )
    .unwrap_or_else(|error| panic!("write root package: {error}"));
    let modelica_source_path = tempdir.path().join("PI.mo");
    fs::write(
        &modelica_source_path,
        "within DemoLib;\nmodel PI\n  parameter Real k = 1;\nend PI;\n",
    )
    .unwrap_or_else(|error| panic!("write Modelica source: {error}"));
    commit_all(tempdir.path(), "initial");

    let repository = RegisteredRepository {
        id: "counting-mixed-modelica-rust-modelica".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        git_ref: None,
        plugins: mixed_modelica_rust_plugin_configs(),
    };
    let calls = Arc::new(AtomicUsize::new(0));
    let registry = bootstrap_builtin_registry_with_counting_rust_plugin(Arc::clone(&calls));

    let first =
        analyze_registered_repository_bundle_with_registry(&repository, tempdir.path(), &registry)
            .unwrap_or_else(|error| panic!("first mixed analysis should succeed: {error}"));
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    fs::write(
        &modelica_source_path,
        "within DemoLib;\nmodel PI\n  parameter Real k = 1;\nend PI;\n// semantic no-op\n",
    )
    .unwrap_or_else(|error| panic!("rewrite Modelica source: {error}"));
    commit_all(tempdir.path(), "ast-equivalent mixed modelica");

    let second =
        analyze_registered_repository_bundle_with_registry(&repository, tempdir.path(), &registry)
            .unwrap_or_else(|error| panic!("second mixed analysis should succeed: {error}"));

    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        first.cache_key.analysis_identity,
        second.cache_key.analysis_identity
    );
}

#[test]
fn analyze_repository_invalidates_cached_analysis_for_ast_equivalent_mixed_rust_unknown_plugin_source_churn()
 {
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    init_git_repository(tempdir.path());
    fs::create_dir_all(tempdir.path().join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    let rust_source_path = tempdir.path().join("src/lib.rs");
    fs::write(
        &rust_source_path,
        "fn solve(x: i32) -> i32 {\n    x + 1\n}\n",
    )
    .unwrap_or_else(|error| panic!("write Rust source: {error}"));
    commit_all(tempdir.path(), "initial");

    let repository = RegisteredRepository {
        id: "counting-mixed-rust-unknown".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        git_ref: None,
        plugins: mixed_rust_unknown_plugin_configs(),
    };
    let calls = Arc::new(AtomicUsize::new(0));
    let mut registry = PluginRegistry::new();
    registry
        .register(CountingRustPlugin {
            calls: Arc::clone(&calls),
        })
        .unwrap_or_else(|error| panic!("register Rust plugin: {error}"));

    let first =
        analyze_registered_repository_bundle_with_registry(&repository, tempdir.path(), &registry)
            .unwrap_or_else(|error| panic!("first mixed analysis should succeed: {error}"));
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    fs::write(
        &rust_source_path,
        "fn solve(x: i32) -> i32 {\n    // semantic no-op\n    x + 1\n}\n",
    )
    .unwrap_or_else(|error| panic!("rewrite Rust source: {error}"));
    commit_all(tempdir.path(), "ast-equivalent mixed rust unknown");

    let second =
        analyze_registered_repository_bundle_with_registry(&repository, tempdir.path(), &registry)
            .unwrap_or_else(|error| panic!("second mixed analysis should succeed: {error}"));

    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_ne!(
        first.cache_key.analysis_identity,
        second.cache_key.analysis_identity
    );
}

#[test]
fn analyze_repository_invalidates_cached_analysis_for_ast_equivalent_mixed_modelica_unknown_plugin_source_churn()
 {
    ensure_linked_modelica_parser_summary_service()
        .unwrap_or_else(|error| panic!("linked Modelica parser-summary service: {error}"));
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    init_git_repository(tempdir.path());
    fs::write(
        tempdir.path().join("package.mo"),
        "within ;\npackage DemoLib\nend DemoLib;\n",
    )
    .unwrap_or_else(|error| panic!("write root package: {error}"));
    let modelica_source_path = tempdir.path().join("PI.mo");
    fs::write(
        &modelica_source_path,
        "within DemoLib;\nmodel PI\n  parameter Real k = 1;\nend PI;\n",
    )
    .unwrap_or_else(|error| panic!("write Modelica source: {error}"));
    commit_all(tempdir.path(), "initial");

    let repository = RegisteredRepository {
        id: "counting-mixed-modelica-unknown".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        git_ref: None,
        plugins: mixed_modelica_unknown_plugin_configs(),
    };
    let calls = Arc::new(AtomicUsize::new(0));
    let mut registry = PluginRegistry::new();
    registry
        .register(CountingModelicaPlugin {
            calls: Arc::clone(&calls),
        })
        .unwrap_or_else(|error| panic!("register Modelica plugin: {error}"));

    let first =
        analyze_registered_repository_bundle_with_registry(&repository, tempdir.path(), &registry)
            .unwrap_or_else(|error| {
                panic!("first mixed Modelica analysis should succeed: {error}")
            });
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    fs::write(
        &modelica_source_path,
        "within DemoLib;\nmodel PI\n  parameter Real k = 1;\nend PI;\n// semantic no-op\n",
    )
    .unwrap_or_else(|error| panic!("rewrite Modelica source: {error}"));
    commit_all(tempdir.path(), "ast-equivalent mixed modelica unknown");

    let second =
        analyze_registered_repository_bundle_with_registry(&repository, tempdir.path(), &registry)
            .unwrap_or_else(|error| {
                panic!("second mixed Modelica analysis should succeed: {error}")
            });

    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_ne!(
        first.cache_key.analysis_identity,
        second.cache_key.analysis_identity
    );
}
