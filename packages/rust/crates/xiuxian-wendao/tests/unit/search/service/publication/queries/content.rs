use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use crate::search::RepoContentChunkSearchFilters;
use crate::search::service::tests::support::ok_or_panic;

use super::support::{publish_repo_content_chunks, repo_search_service};

#[tokio::test]
async fn search_repo_content_chunks_waits_for_repo_read_permit() {
    let service = repo_search_service();
    publish_repo_content_chunks(&service).await;

    let permit_count = service.repo_search_read_permits.available_permits();
    assert!(permit_count > 0);
    let held_permits = ok_or_panic(
        Arc::clone(&service.repo_search_read_permits)
            .acquire_many_owned(u32::try_from(permit_count).unwrap_or(u32::MAX))
            .await,
        "drain repo search read permits",
    );

    let query_service = service.clone();
    let query_task = tokio::spawn(async move {
        query_service
            .search_repo_content_chunks("alpha/repo", "reexport", &HashSet::new(), 5)
            .await
    });

    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(!query_task.is_finished());

    drop(held_permits);
    let hits = ok_or_panic(
        ok_or_panic(query_task.await, "join repo content query task"),
        "query repo content",
    );
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].match_reason.as_deref(), Some("repo_content_search"));
}

#[tokio::test]
async fn search_repo_content_chunks_with_filters_applies_sql_native_repo_filters() {
    let service = repo_search_service();
    publish_repo_content_chunks(&service).await;

    let hits = ok_or_panic(
        service
            .search_repo_content_chunks_with_filters(
                "alpha/repo",
                "reexport",
                &HashSet::new(),
                &RepoContentChunkSearchFilters {
                    path_prefixes: HashSet::from(["src/".to_string()]),
                    filename_filters: HashSet::from(["BaseModelica.jl".to_string()]),
                    ..RepoContentChunkSearchFilters::default()
                },
                5,
            )
            .await,
        "query repo content with sql-native filters",
    );

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].path, "src/BaseModelica.jl");
    assert_eq!(hits[0].title.as_deref(), Some("src/BaseModelica.jl"));
}

#[tokio::test]
async fn search_repo_content_chunks_with_filters_applies_title_filters() {
    let service = repo_search_service();
    publish_repo_content_chunks(&service).await;

    let hits = ok_or_panic(
        service
            .search_repo_content_chunks_with_filters(
                "alpha/repo",
                "reexport",
                &HashSet::new(),
                &RepoContentChunkSearchFilters {
                    title_filters: HashSet::from(["basemodelica".to_string()]),
                    ..RepoContentChunkSearchFilters::default()
                },
                5,
            )
            .await,
        "query repo content with title filters",
    );

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].path, "src/BaseModelica.jl");
}

#[tokio::test]
async fn search_repo_content_chunks_with_filters_applies_tag_filters() {
    let service = repo_search_service();
    publish_repo_content_chunks(&service).await;

    let hits = ok_or_panic(
        service
            .search_repo_content_chunks_with_filters(
                "alpha/repo",
                "reexport",
                &HashSet::new(),
                &RepoContentChunkSearchFilters {
                    tag_filters: HashSet::from(["lang:julia".to_string()]),
                    ..RepoContentChunkSearchFilters::default()
                },
                5,
            )
            .await,
        "query repo content with tag filters",
    );

    assert_eq!(hits.len(), 2);
    assert!(
        hits.iter()
            .all(|hit| hit.tags.iter().any(|tag| tag == "lang:julia"))
    );
}
