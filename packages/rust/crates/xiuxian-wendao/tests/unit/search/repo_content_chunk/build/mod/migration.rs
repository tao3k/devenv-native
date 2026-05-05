use std::collections::HashSet;

use crate::search::cache::SearchPlaneFileFingerprintScope;
use crate::search::repo_content_chunk::build::orchestration::{
    publish_repo_content_chunks, publish_repo_content_chunks_incremental,
};
use crate::search::repo_content_chunk::build::plan::repo_content_chunk_file_fingerprints;
use crate::search::repo_content_chunk::schema::{
    path_column, repo_content_chunk_batches, repo_content_chunk_schema, rows_from_documents,
};
use crate::search::repo_publication_parquet::{
    RepoPublicationRewriteRequest, rewrite_repo_publication_parquet,
};
use crate::search::{SearchCorpusKind, SearchPublicationStorageFormat, SearchRepoPublicationInput};

use super::{
    assert_repo_content_hit_paths, repo_content_publication_or_panic, repo_content_record_or_panic,
    repo_content_service, repo_document, temp_dir_or_panic,
};

#[tokio::test]
async fn repo_content_chunk_incremental_publish_migrates_legacy_single_parquet_to_partitioned_root()
{
    let temp_dir = temp_dir_or_panic();
    let service = repo_content_service(&temp_dir);
    let first_documents = vec![
        repo_document("src/lib.rs", "fn alpha() {}\n", 14, 10),
        repo_document("src/util.rs", "fn beta() {}\n", 13, 10),
    ];
    let first_fingerprints = repo_content_chunk_file_fingerprints(&first_documents);
    let rows = rows_from_documents(&first_documents);
    let changed_batches =
        repo_content_chunk_batches(&rows).unwrap_or_else(|error| panic!("legacy batches: {error}"));
    let parquet_stats = rewrite_repo_publication_parquet(
        &service,
        RepoPublicationRewriteRequest {
            corpus: SearchCorpusKind::RepoContentChunk,
            base_table_name: None,
            target_table_name: "legacy_repo_content_chunk_alpha_repo",
            path_column: path_column(),
            replaced_paths: &std::collections::BTreeSet::new(),
            changed_batches: changed_batches.as_slice(),
            empty_schema: Some(repo_content_chunk_schema()),
        },
    )
    .await
    .unwrap_or_else(|error| panic!("legacy single parquet publish: {error}"));
    service
        .record_repo_publication_input_with_storage_format(
            SearchCorpusKind::RepoContentChunk,
            "alpha/repo",
            SearchRepoPublicationInput {
                table_name: "legacy_repo_content_chunk_alpha_repo".to_string(),
                schema_version: SearchCorpusKind::RepoContentChunk.schema_version(),
                source_revision: Some("rev-1".to_string()),
                table_version_id: parquet_stats.table_version_id,
                row_count: parquet_stats.row_count,
                fragment_count: parquet_stats.fragment_count,
                published_at: parquet_stats.published_at,
            },
            SearchPublicationStorageFormat::Parquet,
        )
        .await;
    service
        .set_file_fingerprints(
            SearchPlaneFileFingerprintScope::repo_corpus(
                SearchCorpusKind::RepoContentChunk,
                "alpha/repo",
            ),
            &first_fingerprints,
        )
        .await;

    let changed_documents = vec![repo_document("src/lib.rs", "fn gamma() {}\n", 14, 20)];
    publish_repo_content_chunks_incremental(
        &service,
        "alpha/repo",
        &changed_documents,
        &std::collections::BTreeSet::from(["src/util.rs".to_string()]),
        Some("rev-2"),
    )
    .await
    .unwrap_or_else(|error| panic!("incremental publish from legacy parquet: {error}"));

    let record = repo_content_record_or_panic(
        service
            .repo_corpus_record_for_reads(SearchCorpusKind::RepoContentChunk, "alpha/repo")
            .await,
        "repo content record after legacy migration",
    );
    let publication = repo_content_publication_or_panic(&record, "publication after migration");
    let publication_root = service.repo_publication_parquet_path(
        SearchCorpusKind::RepoContentChunk,
        publication.table_name.as_str(),
    );
    assert!(
        publication_root.is_dir(),
        "legacy repo-content parquet should migrate to a partitioned publication root"
    );
    assert!(
        publication_root.join("_stats.json").exists(),
        "legacy migration should materialize a stats sidecar for the new partitioned root"
    );
    let language_filters = HashSet::default();
    assert_repo_content_hit_paths(&service, "gamma", &language_filters, &["src/lib.rs"]).await;
    assert_repo_content_hit_paths(&service, "beta", &language_filters, &[]).await;
}

#[tokio::test]
async fn repo_content_chunk_incremental_publish_refreshes_revision_only_metadata_edits() {
    let temp_dir = temp_dir_or_panic();
    let service = repo_content_service(&temp_dir);
    let first_documents = vec![repo_document("src/lib.rs", "fn alpha() {}\n", 14, 10)];
    publish_repo_content_chunks(&service, "alpha/repo", &first_documents, Some("rev-1"))
        .await
        .unwrap_or_else(|error| panic!("first publish: {error}"));
    let first_record = repo_content_record_or_panic(
        service
            .repo_corpus_record_for_reads(SearchCorpusKind::RepoContentChunk, "alpha/repo")
            .await,
        "first record",
    );
    let first_publication = repo_content_publication_or_panic(&first_record, "first publication");
    let first_table_name = first_publication.table_name.clone();

    let changed_documents = vec![repo_document("src/lib.rs", "fn alpha() {}\n", 14, 20)];
    publish_repo_content_chunks_incremental(
        &service,
        "alpha/repo",
        &changed_documents,
        &std::collections::BTreeSet::new(),
        Some("rev-2"),
    )
    .await
    .unwrap_or_else(|error| panic!("incremental publish: {error}"));

    let second_record = repo_content_record_or_panic(
        service
            .repo_corpus_record_for_reads(SearchCorpusKind::RepoContentChunk, "alpha/repo")
            .await,
        "second record",
    );
    let second_publication =
        repo_content_publication_or_panic(&second_record, "second publication");
    assert_eq!(second_publication.table_name, first_table_name);
    assert_eq!(second_publication.source_revision.as_deref(), Some("rev-2"));
    let fingerprints = service
        .file_fingerprints(SearchPlaneFileFingerprintScope::repo_corpus(
            SearchCorpusKind::RepoContentChunk,
            "alpha/repo",
        ))
        .await;
    assert_eq!(
        fingerprints
            .get("src/lib.rs")
            .map(|fingerprint| fingerprint.modified_unix_ms),
        Some(20)
    );
}
