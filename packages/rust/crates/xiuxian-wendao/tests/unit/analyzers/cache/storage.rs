use std::path::PathBuf;

use xiuxian_git_repo::{
    LocalCheckoutMetadata, MaterializedRepo, RepoDriftState, RepoLifecycleState, RepoSourceKind,
};

use crate::analyzers::{RegisteredRepository, RepositoryPluginConfig, RepositoryRefreshPolicy};
use crate::search::FuzzySearchOptions;

use super::super::{
    RepositorySearchQueryCacheKey, build_repository_analysis_cache_key,
    load_cached_repository_analysis_for_revision, load_cached_repository_search_artifacts,
    load_cached_repository_search_result, store_cached_repository_analysis,
    store_cached_repository_search_artifacts, store_cached_repository_search_result,
};
use super::support::{empty_artifacts, ok_or_panic, sample_analysis_key, some_or_panic};

#[test]
fn repository_search_artifacts_cache_roundtrip_uses_analysis_identity() {
    let key = sample_analysis_key("artifact-cache-roundtrip");
    let stored = ok_or_panic(
        store_cached_repository_search_artifacts(key.clone(), empty_artifacts()),
        "artifact cache store should succeed",
    );
    let loaded = some_or_panic(
        ok_or_panic(
            load_cached_repository_search_artifacts(&key),
            "artifact cache load should succeed",
        ),
        "stored artifacts should be present",
    );

    assert!(std::sync::Arc::ptr_eq(&stored, &loaded));
}

#[test]
fn repository_analysis_cache_can_recover_previous_revision_base() {
    let key = sample_analysis_key("revision-base-roundtrip");
    let analysis = crate::analyzers::RepositoryAnalysisOutput {
        modules: vec![crate::analyzers::ModuleRecord {
            repo_id: key.repo_id.clone(),
            module_id: "module:alpha".to_string(),
            qualified_name: "Alpha".to_string(),
            path: "src/lib.rs".to_string(),
        }],
        ..crate::analyzers::RepositoryAnalysisOutput::default()
    };

    ok_or_panic(
        store_cached_repository_analysis(key.clone(), &analysis),
        "store analysis cache",
    );
    let loaded = ok_or_panic(
        load_cached_repository_analysis_for_revision(
            key.repo_id.as_str(),
            key.checkout_root.as_str(),
            key.plugin_ids.as_slice(),
            "rev-1",
        ),
        "load analysis cache by revision",
    );

    assert_eq!(loaded, Some(analysis));
}

#[test]
fn repository_search_query_cache_isolated_by_endpoint_and_filter() {
    let analysis_key = sample_analysis_key("query-cache-isolation");
    let options = FuzzySearchOptions::document_search();
    let module_key = RepositorySearchQueryCacheKey::new(
        &analysis_key,
        "repo.module-search",
        "solve",
        None,
        options,
        10,
    );
    let projected_key = RepositorySearchQueryCacheKey::new(
        &analysis_key,
        "repo.projected-page-search",
        "solve",
        Some("reference".to_string()),
        options,
        10,
    );

    ok_or_panic(
        store_cached_repository_search_result(&module_key, &vec!["module"]),
        "query cache store should succeed",
    );
    ok_or_panic(
        store_cached_repository_search_result(&projected_key, &vec!["projected"]),
        "query cache store should succeed",
    );

    let module_value: Vec<String> = some_or_panic(
        ok_or_panic(
            load_cached_repository_search_result(&module_key),
            "query cache load should succeed",
        ),
        "module cached value should exist",
    );
    let projected_value: Vec<String> = some_or_panic(
        ok_or_panic(
            load_cached_repository_search_result(&projected_key),
            "query cache load should succeed",
        ),
        "projected cached value should exist",
    );

    assert_eq!(module_value, vec!["module".to_string()]);
    assert_eq!(projected_value, vec!["projected".to_string()]);
}

#[test]
fn repository_search_query_cache_key_is_stable_for_normalized_plugin_identity() {
    let source = MaterializedRepo {
        checkout_root: PathBuf::from("/tmp/repo-query-cache-normalized"),
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
    let first_analysis_key = build_repository_analysis_cache_key(
        &RegisteredRepository {
            id: "repo-query-cache-normalized".to_string(),
            path: Some(PathBuf::from("/tmp/repo-query-cache-normalized")),
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
        },
        &source,
        metadata.as_ref(),
    );
    let second_analysis_key = build_repository_analysis_cache_key(
        &RegisteredRepository {
            id: "repo-query-cache-normalized".to_string(),
            path: Some(PathBuf::from("/tmp/repo-query-cache-normalized")),
            url: None,
            git_ref: None,
            refresh: RepositoryRefreshPolicy::Fetch,
            plugins: vec![
                RepositoryPluginConfig::Config {
                    id: "modelica".to_string(),
                    options: serde_json::json!({
                        "mode": "doc-surface"
                    }),
                },
                RepositoryPluginConfig::Id("ast-grep".to_string()),
                RepositoryPluginConfig::Id("julia".to_string()),
                RepositoryPluginConfig::Id("ast-grep".to_string()),
            ],
        },
        &source,
        metadata.as_ref(),
    );
    let options = FuzzySearchOptions::document_search();
    let first_query_key = RepositorySearchQueryCacheKey::new(
        &first_analysis_key,
        "repo.projected-page-search",
        "solve",
        Some("reference".to_string()),
        options,
        10,
    );
    let second_query_key = RepositorySearchQueryCacheKey::new(
        &second_analysis_key,
        "repo.projected-page-search",
        "solve",
        Some("reference".to_string()),
        options,
        10,
    );

    assert_eq!(first_analysis_key, second_analysis_key);
    assert_eq!(first_query_key, second_query_key);
}
