use super::support::{
    Arc, PathBuf, RepoSourceKind, RepoSyncResult, RepositoryAnalysisOutput, SearchCorpusKind,
    SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlaneCache, SearchPlaneService,
    SearchPublicationStorageFormat, SearchRepoPublicationInput, new_coordinator,
};

#[tokio::test]
async fn managed_remote_skips_reindex_when_repo_publications_already_match_revision() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let search_plane = SearchPlaneService::with_paths(
        PathBuf::from("."),
        temp_dir.path().join("search-plane"),
        SearchManifestKeyspace::new("xiuxian:test:repo-current-publications"),
        SearchMaintenancePolicy::default(),
    );
    let documents = vec![crate::repo_index::types::RepoCodeDocument {
        path: "src/lib.rs".to_string(),
        language: Some("rust".to_string()),
        contents: Arc::<str>::from("fn alpha() {}\n"),
        size_bytes: 14,
        modified_unix_ms: 0,
    }];
    search_plane
        .publish_repo_entities_with_revision(
            "alpha/repo",
            &RepositoryAnalysisOutput {
                modules: vec![crate::analyzers::ModuleRecord {
                    repo_id: "alpha/repo".to_string().into(),
                    module_id: "module:alpha".to_string().into(),
                    qualified_name: "Alpha".to_string(),
                    path: "src/lib.rs".to_string().into(),
                }],
                ..RepositoryAnalysisOutput::default()
            },
            &documents,
            Some("rev-1"),
        )
        .await
        .unwrap_or_else(|error| panic!("publish repo entities: {error}"));
    search_plane
        .publish_repo_content_chunks_with_revision("alpha/repo", &documents, Some("rev-1"))
        .await
        .unwrap_or_else(|error| panic!("publish repo content chunks: {error}"));
    let coordinator = new_coordinator(search_plane);

    assert!(
        coordinator
            .repo_publications_are_current(
                "alpha/repo",
                &RepoSyncResult {
                    repo_id: "alpha/repo".to_string(),
                    source_kind: RepoSourceKind::ManagedRemote,
                    revision: Some("rev-1".to_string()),
                    ..RepoSyncResult::default()
                },
            )
            .await
    );
}

#[tokio::test]
async fn managed_remote_reuses_latest_persisted_publications_without_revision_cache() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let keyspace = SearchManifestKeyspace::new("xiuxian:test:repo-latest-publication-fast-path");
    let cache = SearchPlaneCache::for_tests(keyspace.clone());
    let search_plane = SearchPlaneService::with_runtime(
        PathBuf::from("."),
        temp_dir.path().join("search-plane"),
        keyspace,
        SearchMaintenancePolicy::default(),
        cache.clone(),
    );

    for corpus in [
        SearchCorpusKind::RepoEntity,
        SearchCorpusKind::RepoContentChunk,
    ] {
        search_plane
            .record_repo_publication_input_with_storage_format(
                corpus,
                "alpha/repo",
                SearchRepoPublicationInput {
                    table_name: format!("{corpus}_alpha_repo_rev_1"),
                    schema_version: corpus.schema_version(),
                    source_revision: Some("rev-1".to_string()),
                    table_version_id: 1,
                    row_count: 1,
                    fragment_count: 1,
                    published_at: "2026-04-06T00:00:01Z".to_string(),
                },
                SearchPublicationStorageFormat::Parquet,
            )
            .await;
        cache
            .delete_repo_publication_revision_cache(corpus, "alpha/repo")
            .await;
    }

    search_plane.clear_all_in_memory_repo_corpus_records_for_test();
    let coordinator = new_coordinator(search_plane);

    assert!(
        coordinator
            .repo_publications_are_current(
                "alpha/repo",
                &RepoSyncResult {
                    repo_id: "alpha/repo".to_string(),
                    source_kind: RepoSourceKind::ManagedRemote,
                    revision: Some("rev-1".to_string()),
                    ..RepoSyncResult::default()
                },
            )
            .await
    );
}

#[tokio::test]
async fn local_checkout_does_not_short_circuit_on_revision_match() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let search_plane = SearchPlaneService::with_paths(
        PathBuf::from("."),
        temp_dir.path().join("search-plane"),
        SearchManifestKeyspace::new("xiuxian:test:repo-local-checkout-short-circuit"),
        SearchMaintenancePolicy::default(),
    );
    let documents = vec![crate::repo_index::types::RepoCodeDocument {
        path: "src/lib.rs".to_string(),
        language: Some("rust".to_string()),
        contents: Arc::<str>::from("fn alpha() {}\n"),
        size_bytes: 14,
        modified_unix_ms: 0,
    }];
    search_plane
        .publish_repo_entities_with_revision(
            "alpha/repo",
            &RepositoryAnalysisOutput {
                modules: vec![crate::analyzers::ModuleRecord {
                    repo_id: "alpha/repo".to_string().into(),
                    module_id: "module:alpha".to_string().into(),
                    qualified_name: "Alpha".to_string(),
                    path: "src/lib.rs".to_string().into(),
                }],
                ..RepositoryAnalysisOutput::default()
            },
            &documents,
            Some("rev-1"),
        )
        .await
        .unwrap_or_else(|error| panic!("publish repo entities: {error}"));
    search_plane
        .publish_repo_content_chunks_with_revision("alpha/repo", &documents, Some("rev-1"))
        .await
        .unwrap_or_else(|error| panic!("publish repo content chunks: {error}"));
    let coordinator = new_coordinator(search_plane);

    assert!(
        !coordinator
            .repo_publications_are_current(
                "alpha/repo",
                &RepoSyncResult {
                    repo_id: "alpha/repo".to_string(),
                    source_kind: RepoSourceKind::LocalCheckout,
                    revision: Some("rev-1".to_string()),
                    ..RepoSyncResult::default()
                },
            )
            .await
    );
}
