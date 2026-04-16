use std::fs;

use serial_test::serial;
use xiuxian_git_repo::{
    LocalCheckoutMetadata, MaterializedRepo, RepoDriftState, RepoLifecycleState, RepoSourceKind,
};

use crate::analyzers::{RegisteredRepository, RepositoryRefreshPolicy};

use super::super::build_repository_analysis_cache_key;
use super::support::{
    ensure_linked_modelica_parser_summary_service, mixed_modelica_rust_plugin_configs,
};

#[test]
#[serial(mixed_modelica_rust_live)]
fn build_repository_analysis_cache_key_reuses_mixed_modelica_rust_identity_for_rust_ast_equivalent_source_churn()
-> Result<(), Box<dyn std::error::Error>> {
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
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

    let repository = RegisteredRepository {
        id: "repo-cache-identity-mixed-modelica-rust-rust".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: mixed_modelica_rust_plugin_configs(),
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
        &rust_source_path,
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
    Ok(())
}

#[test]
#[serial(mixed_modelica_rust_live)]
fn build_repository_analysis_cache_key_reuses_mixed_modelica_rust_identity_for_modelica_ast_equivalent_source_churn()
-> Result<(), Box<dyn std::error::Error>> {
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
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

    let repository = RegisteredRepository {
        id: "repo-cache-identity-mixed-modelica-rust-modelica".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: mixed_modelica_rust_plugin_configs(),
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
        &modelica_source_path,
        "within DemoLib;\nmodel PI\n  parameter Real k = 1;\nend PI;\n// semantic no-op\n",
    )
    .unwrap_or_else(|error| panic!("rewrite Modelica source: {error}"));
    let second_key = build_repository_analysis_cache_key(
        &repository,
        &source,
        Some(&LocalCheckoutMetadata {
            revision: Some("rev-2".to_string()),
            remote_url: None,
        }),
    );

    assert_eq!(first_key.analysis_identity, second_key.analysis_identity);
    Ok(())
}
