use super::support::{
    SearchCorpusKind, SearchPublicationStorageFormat, SearchRepoCorpusRecord,
    SearchRepoPublicationInput, SearchRepoPublicationRecord, cache_for_tests,
};

#[tokio::test]
async fn delete_repo_publication_revision_cache_clears_retained_revision_entries() {
    let cache = cache_for_tests();
    let publication = SearchRepoPublicationRecord::new_with_storage_format(
        SearchCorpusKind::RepoEntity,
        "alpha/repo",
        SearchRepoPublicationInput {
            table_name: "repo_entity_alpha_repo".to_string(),
            schema_version: 1,
            source_revision: Some("rev-clean-build".to_string()),
            table_version_id: 7,
            row_count: 5,
            fragment_count: 1,
            published_at: "2026-04-06T00:00:00Z".to_string(),
        },
        SearchPublicationStorageFormat::Lance,
    );

    cache
        .set_repo_publication_for_revision(SearchCorpusKind::RepoEntity, "alpha/repo", &publication)
        .await;
    assert!(
        cache
            .get_repo_publication_for_revision(
                SearchCorpusKind::RepoEntity,
                "alpha/repo",
                "rev-clean-build",
            )
            .await
            .is_some()
    );

    cache
        .delete_repo_publication_revision_cache(SearchCorpusKind::RepoEntity, "alpha/repo")
        .await;

    assert!(
        cache
            .get_repo_publication_for_revision(
                SearchCorpusKind::RepoEntity,
                "alpha/repo",
                "rev-clean-build",
            )
            .await
            .is_none()
    );
    assert!(
        cache
            .get_repo_publication_revisions(SearchCorpusKind::RepoEntity, "alpha/repo")
            .await
            .is_empty()
    );
}

#[tokio::test]
async fn delete_repo_publication_revision_cache_preserves_latest_repo_corpus_record() {
    let cache = cache_for_tests();
    let publication = SearchRepoPublicationRecord::new_with_storage_format(
        SearchCorpusKind::RepoEntity,
        "alpha/repo",
        SearchRepoPublicationInput {
            table_name: "repo_entity_alpha_repo".to_string(),
            schema_version: 1,
            source_revision: Some("rev-clean-build".to_string()),
            table_version_id: 7,
            row_count: 5,
            fragment_count: 1,
            published_at: "2026-04-06T00:00:00Z".to_string(),
        },
        SearchPublicationStorageFormat::Parquet,
    );

    cache
        .set_repo_corpus_record(&SearchRepoCorpusRecord::new(
            SearchCorpusKind::RepoEntity,
            "alpha/repo",
            None,
            Some(publication.clone()),
        ))
        .await;
    cache
        .set_repo_publication_for_revision(SearchCorpusKind::RepoEntity, "alpha/repo", &publication)
        .await;

    cache
        .delete_repo_publication_revision_cache(SearchCorpusKind::RepoEntity, "alpha/repo")
        .await;

    let record = cache
        .get_repo_corpus_record(SearchCorpusKind::RepoEntity, "alpha/repo")
        .await
        .unwrap_or_else(|| panic!("latest repo corpus record should remain available"));
    assert_eq!(
        record
            .publication
            .as_ref()
            .and_then(|publication| publication.source_revision.as_deref()),
        Some("rev-clean-build")
    );
    assert!(
        cache
            .get_repo_publication_for_revision(
                SearchCorpusKind::RepoEntity,
                "alpha/repo",
                "rev-clean-build",
            )
            .await
            .is_none()
    );
}
