use super::support::{
    CachedRepoSearchProbe, normalized_gateway_analysis_keys, unique_repo_gateway_keyspace,
};
use super::*;

#[tokio::test]
async fn cached_repo_search_result_reuses_hot_query_payload() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let keyspace = unique_repo_gateway_keyspace("cache", temp_dir.path());
    let search_plane = SearchPlaneService::with_test_cache(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        keyspace,
        SearchMaintenancePolicy::default(),
    );
    let load_count = Arc::new(AtomicUsize::new(0));

    let first = with_cached_repo_search_result(
        &search_plane,
        "repo.symbol-search",
        "alpha/repo",
        "solve",
        5,
        {
            let load_count = Arc::clone(&load_count);
            || async move {
                load_count.fetch_add(1, Ordering::SeqCst);
                Ok(CachedRepoSearchProbe {
                    value: "first".to_string(),
                })
            }
        },
    )
    .await
    .unwrap_or_else(|error| panic!("first cached search result: {error:?}"));

    let second = with_cached_repo_search_result(
        &search_plane,
        "repo.symbol-search",
        "alpha/repo",
        "solve",
        5,
        {
            let load_count = Arc::clone(&load_count);
            || async move {
                load_count.fetch_add(1, Ordering::SeqCst);
                Err(StudioApiError::internal(
                    "UNEXPECTED_RELOAD",
                    "cached repo search should not execute loader twice",
                    None,
                ))
            }
        },
    )
    .await
    .unwrap_or_else(|error| panic!("cached repo search hit should succeed: {error:?}"));

    assert_eq!(first, second);
    assert_eq!(first.value, "first");
    assert_eq!(load_count.load(Ordering::SeqCst), 1);
}

#[test]
fn repository_search_key_is_stable_for_normalized_plugin_identity() {
    let (first_analysis_key, second_analysis_key) = normalized_gateway_analysis_keys();

    let first_key = repository_search_key(
        &first_analysis_key,
        "repo.module-search",
        "solve",
        10,
        FuzzySearchOptions::document_search(),
    );
    let second_key = repository_search_key(
        &second_analysis_key,
        "repo.module-search",
        "solve",
        10,
        FuzzySearchOptions::document_search(),
    );

    assert_eq!(first_analysis_key, second_analysis_key);
    assert_eq!(first_key, second_key);
}

#[test]
fn projected_page_search_cache_key_is_stable_for_normalized_plugin_identity() {
    let (first_analysis_key, second_analysis_key) = normalized_gateway_analysis_keys();

    let first_key = RepositorySearchQueryCacheKey::new(
        &first_analysis_key,
        "repo.projected-page-search",
        "solve",
        Some("reference".to_string()),
        FuzzySearchOptions::document_search(),
        10,
    );
    let second_key = RepositorySearchQueryCacheKey::new(
        &second_analysis_key,
        "repo.projected-page-search",
        "solve",
        Some("reference".to_string()),
        FuzzySearchOptions::document_search(),
        10,
    );

    assert_eq!(first_analysis_key, second_analysis_key);
    assert_eq!(first_key, second_key);
}
