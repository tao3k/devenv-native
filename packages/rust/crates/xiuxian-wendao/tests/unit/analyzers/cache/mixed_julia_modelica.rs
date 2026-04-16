use std::fs;

use serial_test::serial;
use xiuxian_git_repo::{
    LocalCheckoutMetadata, MaterializedRepo, RepoDriftState, RepoLifecycleState, RepoSourceKind,
};

use crate::analyzers::{RegisteredRepository, RepositoryPluginConfig, RepositoryRefreshPolicy};

use super::super::build_repository_analysis_cache_key;
use super::support::{
    ensure_linked_julia_parser_summary_service, ensure_linked_modelica_parser_summary_service,
};

#[test]
#[serial(mixed_julia_modelica_live)]
fn build_repository_analysis_cache_key_reuses_mixed_julia_modelica_identity_for_julia_ast_equivalent_source_churn()
-> Result<(), Box<dyn std::error::Error>> {
    ensure_linked_julia_parser_summary_service()?;
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    fs::write(
        tempdir.path().join("Project.toml"),
        "name = \"MixedCacheKeyDemo\"\n",
    )
    .unwrap_or_else(|error| panic!("write Project.toml: {error}"));
    fs::create_dir_all(tempdir.path().join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    let julia_source_path = tempdir.path().join("src/MixedCacheKeyDemo.jl");
    fs::write(
        &julia_source_path,
        "module MixedCacheKeyDemo\nalpha() = 1\nend\n",
    )
    .unwrap_or_else(|error| panic!("write Julia source: {error}"));
    fs::write(
        tempdir.path().join("Demo.mo"),
        "package Demo\n  model Sample\n    Real x;\n  end Sample;\nend Demo;\n",
    )
    .unwrap_or_else(|error| panic!("write Modelica source: {error}"));

    let repository = RegisteredRepository {
        id: "repo-cache-identity-mixed-julia-modelica-julia".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![
            RepositoryPluginConfig::Id("julia".to_string()),
            RepositoryPluginConfig::Id("modelica".to_string()),
        ],
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
        &julia_source_path,
        "module MixedCacheKeyDemo\nalpha() = 1\n# semantic no-op\nend\n",
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
    Ok(())
}

#[test]
#[serial(mixed_julia_modelica_live)]
fn build_repository_analysis_cache_key_reuses_mixed_julia_modelica_identity_for_modelica_ast_equivalent_source_churn()
-> Result<(), Box<dyn std::error::Error>> {
    ensure_linked_julia_parser_summary_service()?;
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    fs::write(
        tempdir.path().join("Project.toml"),
        "name = \"MixedCacheKeyDemo\"\n",
    )
    .unwrap_or_else(|error| panic!("write Project.toml: {error}"));
    fs::create_dir_all(tempdir.path().join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    fs::write(
        tempdir.path().join("src/MixedCacheKeyDemo.jl"),
        "module MixedCacheKeyDemo\nalpha() = 1\nend\n",
    )
    .unwrap_or_else(|error| panic!("write Julia source: {error}"));
    let modelica_source_path = tempdir.path().join("Demo.mo");
    fs::write(
        &modelica_source_path,
        "package Demo\n  model Sample\n    Real x;\n  end Sample;\nend Demo;\n",
    )
    .unwrap_or_else(|error| panic!("write Modelica source: {error}"));

    let repository = RegisteredRepository {
        id: "repo-cache-identity-mixed-julia-modelica-modelica".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![
            RepositoryPluginConfig::Id("julia".to_string()),
            RepositoryPluginConfig::Id("modelica".to_string()),
        ],
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
