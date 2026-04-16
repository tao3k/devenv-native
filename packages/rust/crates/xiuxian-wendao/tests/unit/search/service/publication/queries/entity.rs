use std::iter::FromIterator;

use super::support::{publish_repo_entities, repo_search_service, *};

#[tokio::test]
async fn search_repo_entities_reads_hits_from_published_table() {
    let service = repo_search_service();
    publish_repo_entities(&service).await;

    let kind_filters = HashSet::from_iter([String::from("function")]);
    let hits = ok_or_panic(
        service
            .search_repo_entities("alpha/repo", "reexport", &HashSet::new(), &kind_filters, 5)
            .await,
        "query repo entities",
    );

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].doc_type.as_deref(), Some("symbol"));
    assert_eq!(hits[0].stem, "reexport");
    assert_eq!(hits[0].path, "src/BaseModelica.jl");
    assert_eq!(hits[0].match_reason.as_deref(), Some("repo_symbol_search"));
}

#[tokio::test]
async fn search_repo_entities_waits_for_repo_read_permit() {
    let service = repo_search_service();
    publish_repo_entities(&service).await;

    let permit_count = service.repo_search_read_permits.available_permits();
    assert!(permit_count > 0);
    let held_permits = ok_or_panic(
        Arc::clone(&service.repo_search_read_permits)
            .acquire_many_owned(u32::try_from(permit_count).unwrap_or(u32::MAX))
            .await,
        "drain repo search read permits",
    );

    let kind_filters = HashSet::from_iter([String::from("function")]);
    let query_service = service.clone();
    let query_task = tokio::spawn(async move {
        query_service
            .search_repo_entities("alpha/repo", "reexport", &HashSet::new(), &kind_filters, 5)
            .await
    });

    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(!query_task.is_finished());

    drop(held_permits);
    let hits = ok_or_panic(
        ok_or_panic(query_task.await, "join repo entity query task"),
        "query repo entities",
    );
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].path, "src/BaseModelica.jl");
}
