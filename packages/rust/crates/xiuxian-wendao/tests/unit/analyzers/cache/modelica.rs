use std::fs;

use serial_test::serial;
use xiuxian_git_repo::{
    LocalCheckoutMetadata, MaterializedRepo, RepoDriftState, RepoLifecycleState, RepoSourceKind,
};

use crate::analyzers::{RegisteredRepository, RepositoryPluginConfig, RepositoryRefreshPolicy};

use super::super::build_repository_analysis_cache_key;
use super::support::ensure_linked_modelica_parser_summary_service;

#[test]
#[serial(modelica_live)]
fn build_repository_analysis_cache_key_reuses_modelica_identity_for_ast_equivalent_source_churn()
-> Result<(), Box<dyn std::error::Error>> {
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let source_path = tempdir.path().join("Demo.mo");
    fs::write(
        &source_path,
        "package Demo\n  model Sample\n    Real x;\n  end Sample;\nend Demo;\n",
    )
    .unwrap_or_else(|error| panic!("write Modelica source: {error}"));

    let repository = RegisteredRepository {
        id: "repo-cache-identity-modelica-semantic".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("modelica".to_string())],
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
        "package Demo\n  model Sample\n    Real x;\n  end Sample;\nend Demo;\n// semantic no-op\n",
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

#[test]
#[serial(modelica_live)]
fn build_repository_analysis_cache_key_invalidates_on_modelica_source_change()
-> Result<(), Box<dyn std::error::Error>> {
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let source_path = tempdir.path().join("Demo.mo");
    fs::write(
        &source_path,
        "package Demo\n  model Sample\n    Real x;\n  end Sample;\nend Demo;\n",
    )
    .unwrap_or_else(|error| panic!("write Modelica source: {error}"));

    let repository = RegisteredRepository {
        id: "repo-cache-identity-modelica-change".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("modelica".to_string())],
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
        "package Demo\n  model Sample\n    Real x;\n    Real y;\n  end Sample;\nend Demo;\n",
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

    assert_ne!(first_key.analysis_identity, second_key.analysis_identity);
    Ok(())
}
