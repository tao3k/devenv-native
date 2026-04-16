use std::path::PathBuf;

use crate::analyzers::{RegisteredRepository, RepositoryPluginConfig, RepositoryRefreshPolicy};
use xiuxian_git_repo::{
    LocalCheckoutMetadata, MaterializedRepo, RepoDriftState, RepoLifecycleState, RepoSourceKind,
};

use super::super::build_repository_analysis_cache_key;

#[test]
fn build_repository_analysis_cache_key_sorts_and_deduplicates_plugin_ids() {
    let repository = RegisteredRepository {
        id: "repo-cache-key".to_string(),
        path: Some(PathBuf::from("/tmp/repo-cache-key")),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![
            RepositoryPluginConfig::Id("plugin-z".to_string()),
            RepositoryPluginConfig::Id("plugin-a".to_string()),
            RepositoryPluginConfig::Id("plugin-z".to_string()),
        ],
    };
    let source = MaterializedRepo {
        checkout_root: PathBuf::from("/tmp/repo-cache-key"),
        mirror_root: None,
        mirror_revision: Some("mirror-1".to_string()),
        tracking_revision: Some("tracking-1".to_string()),
        last_fetched_at: None,
        drift_state: RepoDriftState::NotApplicable,
        mirror_state: RepoLifecycleState::NotApplicable,
        checkout_state: RepoLifecycleState::Validated,
        source_kind: RepoSourceKind::LocalCheckout,
    };
    let metadata = Some(LocalCheckoutMetadata {
        revision: Some("rev-1".to_string()),
        remote_url: None,
    });

    let key = build_repository_analysis_cache_key(&repository, &source, metadata.as_ref());

    assert_eq!(
        key.plugin_ids,
        vec!["plugin-a".to_string(), "plugin-z".to_string()]
    );
    assert!(!key.analysis_identity.is_empty());
    assert_eq!(key.checkout_revision, Some("rev-1".to_string()));
}

#[test]
fn build_repository_analysis_cache_key_normalizes_mixed_plugin_declarations() {
    let first_repository = RegisteredRepository {
        id: "repo-cache-key-normalized".to_string(),
        path: Some(PathBuf::from("/tmp/repo-cache-key-normalized")),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![
            RepositoryPluginConfig::Id("ast-grep".to_string()),
            RepositoryPluginConfig::Id("julia".to_string()),
            RepositoryPluginConfig::Config {
                id: "modelica".to_string(),
                options: serde_json::json!({
                    "mode": "parser-summary"
                }),
            },
        ],
    };
    let reordered_repository = RegisteredRepository {
        plugins: vec![
            RepositoryPluginConfig::Config {
                id: "modelica".to_string(),
                options: serde_json::json!({
                    "mode": "doc-surface"
                }),
            },
            RepositoryPluginConfig::Id("julia".to_string()),
            RepositoryPluginConfig::Id("ast-grep".to_string()),
            RepositoryPluginConfig::Id("ast-grep".to_string()),
        ],
        ..first_repository.clone()
    };
    let source = MaterializedRepo {
        checkout_root: PathBuf::from("/tmp/repo-cache-key-normalized"),
        mirror_root: None,
        mirror_revision: Some("mirror-1".to_string()),
        tracking_revision: Some("tracking-1".to_string()),
        last_fetched_at: None,
        drift_state: RepoDriftState::NotApplicable,
        mirror_state: RepoLifecycleState::NotApplicable,
        checkout_state: RepoLifecycleState::Validated,
        source_kind: RepoSourceKind::LocalCheckout,
    };
    let metadata = Some(LocalCheckoutMetadata {
        revision: Some("rev-1".to_string()),
        remote_url: None,
    });

    let first_key =
        build_repository_analysis_cache_key(&first_repository, &source, metadata.as_ref());
    let second_key =
        build_repository_analysis_cache_key(&reordered_repository, &source, metadata.as_ref());

    assert_eq!(
        first_key.plugin_ids,
        vec![
            "ast-grep".to_string(),
            "julia".to_string(),
            "modelica".to_string()
        ]
    );
    assert_eq!(first_key, second_key);
    assert_eq!(first_key.analysis_identity, second_key.analysis_identity);
}
