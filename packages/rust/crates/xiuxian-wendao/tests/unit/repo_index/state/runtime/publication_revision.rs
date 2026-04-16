use super::support::*;

#[tokio::test]
async fn managed_remote_reuses_revision_scoped_publications_after_latest_record_advances() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let search_plane = SearchPlaneService::with_paths(
        PathBuf::from("."),
        temp_dir.path().join("search-plane"),
        SearchManifestKeyspace::new("xiuxian:test:repo-revision-scoped-reuse"),
        SearchMaintenancePolicy::default(),
    );
    let documents = vec![crate::repo_index::types::RepoCodeDocument {
        path: "src/lib.rs".to_string(),
        language: Some("rust".to_string()),
        contents: Arc::<str>::from("fn alpha() {}\n"),
        size_bytes: 14,
        modified_unix_ms: 0,
    }];
    let analysis = RepositoryAnalysisOutput {
        modules: vec![crate::analyzers::ModuleRecord {
            repo_id: "alpha/repo".to_string(),
            module_id: "module:alpha".to_string(),
            qualified_name: "Alpha".to_string(),
            path: "src/lib.rs".to_string(),
        }],
        ..RepositoryAnalysisOutput::default()
    };
    search_plane
        .publish_repo_entities_with_revision("alpha/repo", &analysis, &documents, Some("rev-1"))
        .await
        .unwrap_or_else(|error| panic!("publish repo entities rev-1: {error}"));
    search_plane
        .publish_repo_content_chunks_with_revision("alpha/repo", &documents, Some("rev-1"))
        .await
        .unwrap_or_else(|error| panic!("publish repo content chunks rev-1: {error}"));
    search_plane
        .publish_repo_entities_with_revision("alpha/repo", &analysis, &documents, Some("rev-2"))
        .await
        .unwrap_or_else(|error| panic!("publish repo entities rev-2: {error}"));
    search_plane
        .publish_repo_content_chunks_with_revision("alpha/repo", &documents, Some("rev-2"))
        .await
        .unwrap_or_else(|error| panic!("publish repo content chunks rev-2: {error}"));

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
async fn managed_remote_does_not_reuse_evicted_revision_scoped_publications() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let search_plane = SearchPlaneService::with_test_cache_and_revision_retention(
        PathBuf::from("."),
        temp_dir.path().join("search-plane"),
        SearchManifestKeyspace::new("xiuxian:test:repo-revision-retention"),
        SearchMaintenancePolicy::default(),
        1,
    );
    for (table_version_id, revision) in [(1, "rev-1"), (2, "rev-2")] {
        for corpus in [
            SearchCorpusKind::RepoEntity,
            SearchCorpusKind::RepoContentChunk,
        ] {
            search_plane
                .record_repo_publication_input_with_storage_format(
                    corpus,
                    "alpha/repo",
                    SearchRepoPublicationInput {
                        table_name: format!("{corpus}_alpha_repo_{revision}"),
                        schema_version: corpus.schema_version(),
                        source_revision: Some(revision.to_string()),
                        table_version_id,
                        row_count: 1,
                        fragment_count: 1,
                        published_at: format!("2026-04-06T00:00:0{table_version_id}Z"),
                    },
                    SearchPublicationStorageFormat::Parquet,
                )
                .await;
        }
    }

    let coordinator = new_coordinator(search_plane);

    assert!(
        !coordinator
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
    assert!(
        coordinator
            .repo_publications_are_current(
                "alpha/repo",
                &RepoSyncResult {
                    repo_id: "alpha/repo".to_string(),
                    source_kind: RepoSourceKind::ManagedRemote,
                    revision: Some("rev-2".to_string()),
                    ..RepoSyncResult::default()
                },
            )
            .await
    );
}

#[tokio::test]
async fn managed_remote_requires_both_repo_corpora_to_be_current_parquet_publications() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let search_plane = SearchPlaneService::with_paths(
        PathBuf::from("."),
        temp_dir.path().join("search-plane"),
        SearchManifestKeyspace::new("xiuxian:test:repo-missing-parquet-short-circuit"),
        SearchMaintenancePolicy::default(),
    );
    let published_at = Utc::now().to_rfc3339();
    search_plane
        .record_repo_publication_input_with_storage_format(
            SearchCorpusKind::RepoEntity,
            "alpha/repo",
            SearchRepoPublicationInput {
                table_name: "repo_entity_alpha_repo".to_string(),
                schema_version: SearchCorpusKind::RepoEntity.schema_version(),
                source_revision: Some("rev-1".to_string()),
                table_version_id: 1,
                row_count: 1,
                fragment_count: 1,
                published_at: published_at.clone(),
            },
            SearchPublicationStorageFormat::Lance,
        )
        .await;
    search_plane
        .record_repo_publication_input_with_storage_format(
            SearchCorpusKind::RepoContentChunk,
            "alpha/repo",
            SearchRepoPublicationInput {
                table_name: "repo_content_chunk_alpha_repo".to_string(),
                schema_version: SearchCorpusKind::RepoContentChunk.schema_version(),
                source_revision: Some("rev-1".to_string()),
                table_version_id: 1,
                row_count: 1,
                fragment_count: 1,
                published_at,
            },
            SearchPublicationStorageFormat::Parquet,
        )
        .await;
    let coordinator = new_coordinator(search_plane);

    assert!(
        !coordinator
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
