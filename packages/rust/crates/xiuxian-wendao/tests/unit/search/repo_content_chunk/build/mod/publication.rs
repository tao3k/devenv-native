use std::collections::HashSet;
#[cfg(unix)]
use std::{fs, os::unix::fs::MetadataExt};

use crate::search::cache::SearchPlaneFileFingerprintScope;
use crate::search::repo_content_chunk::build::orchestration::{
    publish_repo_content_chunks, publish_repo_content_chunks_incremental,
};
#[cfg(unix)]
use crate::search::repo_content_chunk::build::partitions::{
    repo_content_chunk_partition_count_for_document_count,
    repo_content_chunk_partition_id_for_count,
};
use crate::search::{SearchCorpusKind, SearchPublicationStorageFormat};

use super::{
    assert_no_lance_table, assert_repo_content_hit_paths, assert_repo_content_prewarmed,
    repo_content_publication_or_panic, repo_content_record_or_panic, repo_content_service,
    repo_content_untouched_partition_paths, repo_document, temp_dir_or_panic,
};

#[tokio::test]
async fn repo_content_chunk_incremental_refresh_reuses_unchanged_rows() {
    let temp_dir = temp_dir_or_panic();
    let service = repo_content_service(&temp_dir);
    let first_documents = vec![
        repo_document("src/lib.rs", "fn alpha() {}\n", 14, 10),
        repo_document("src/util.rs", "fn beta() {}\n", 13, 10),
    ];
    publish_repo_content_chunks(&service, "alpha/repo", &first_documents, Some("rev-1"))
        .await
        .unwrap_or_else(|error| panic!("first publish: {error}"));

    let first_record = repo_content_record_or_panic(
        service
            .repo_corpus_record_for_reads(SearchCorpusKind::RepoContentChunk, "alpha/repo")
            .await,
        "first repo content record",
    );
    let first_table_name = repo_content_publication_or_panic(&first_record, "first publication")
        .table_name
        .clone();
    assert_no_lance_table(
        &service,
        first_table_name.as_str(),
        "repo content publication should no longer create a Lance table",
    );
    assert_repo_content_prewarmed(&first_record);

    let second_documents = vec![
        repo_document("src/lib.rs", "fn gamma() {}\n", 14, 20),
        repo_document("src/util.rs", "fn beta() {}\n", 13, 10),
    ];
    publish_repo_content_chunks(&service, "alpha/repo", &second_documents, Some("rev-2"))
        .await
        .unwrap_or_else(|error| panic!("second publish: {error}"));

    let second_record = repo_content_record_or_panic(
        service
            .repo_corpus_record_for_reads(SearchCorpusKind::RepoContentChunk, "alpha/repo")
            .await,
        "second repo content record",
    );
    let second_publication =
        repo_content_publication_or_panic(&second_record, "second publication");
    assert_ne!(second_publication.table_name, first_table_name);
    assert_no_lance_table(
        &service,
        second_publication.table_name.as_str(),
        "repo content incremental publication should stay parquet-only",
    );
    assert_eq!(second_publication.source_revision.as_deref(), Some("rev-2"));
    assert_eq!(
        second_publication.storage_format,
        SearchPublicationStorageFormat::Parquet
    );
    assert_repo_content_prewarmed(&second_record);
    let parquet_path = service.repo_publication_parquet_path(
        SearchCorpusKind::RepoContentChunk,
        second_publication.table_name.as_str(),
    );
    assert!(parquet_path.exists(), "missing repo content parquet export");
    assert!(
        parquet_path.is_dir(),
        "repo content publication should now export one partitioned parquet directory"
    );
    assert!(
        parquet_path.join("_stats.json").exists(),
        "repo content partitioned publication should persist a stats sidecar"
    );

    let language_filters = HashSet::default();
    assert_repo_content_hit_paths(&service, "beta", &language_filters, &["src/util.rs"]).await;
    assert_repo_content_hit_paths(&service, "gamma", &language_filters, &["src/lib.rs"]).await;
    assert_repo_content_hit_paths(&service, "alpha", &language_filters, &[]).await;

    let fingerprints = service
        .file_fingerprints(SearchPlaneFileFingerprintScope::repo_corpus(
            SearchCorpusKind::RepoContentChunk,
            "alpha/repo",
        ))
        .await;
    assert_eq!(fingerprints.len(), 2);
    assert_eq!(
        fingerprints
            .get("src/lib.rs")
            .map(|fingerprint| fingerprint.modified_unix_ms),
        Some(20)
    );
}

#[tokio::test]
async fn repo_content_chunk_incremental_publish_updates_changed_and_deleted_rows() {
    let temp_dir = temp_dir_or_panic();
    let service = repo_content_service(&temp_dir);
    let first_documents = vec![
        repo_document("src/lib.rs", "fn alpha() {}\n", 14, 10),
        repo_document("src/util.rs", "fn beta() {}\n", 13, 10),
    ];
    publish_repo_content_chunks(&service, "alpha/repo", &first_documents, Some("rev-1"))
        .await
        .unwrap_or_else(|error| panic!("first publish: {error}"));

    let changed_documents = vec![repo_document("src/lib.rs", "fn gamma() {}\n", 14, 20)];
    publish_repo_content_chunks_incremental(
        &service,
        "alpha/repo",
        &changed_documents,
        &std::collections::BTreeSet::from(["src/util.rs".to_string()]),
        Some("rev-2"),
    )
    .await
    .unwrap_or_else(|error| panic!("incremental publish: {error}"));

    let record = repo_content_record_or_panic(
        service
            .repo_corpus_record_for_reads(SearchCorpusKind::RepoContentChunk, "alpha/repo")
            .await,
        "repo content record after incremental publish",
    );
    let publication = repo_content_publication_or_panic(&record, "publication after incremental");
    assert_eq!(publication.source_revision.as_deref(), Some("rev-2"));
    assert_eq!(
        publication.storage_format,
        SearchPublicationStorageFormat::Parquet
    );
    assert!(
        service
            .repo_publication_parquet_path(
                SearchCorpusKind::RepoContentChunk,
                publication.table_name.as_str()
            )
            .is_dir(),
        "repo content incremental publication should export one partitioned parquet directory"
    );
    assert!(
        service
            .repo_publication_parquet_path(
                SearchCorpusKind::RepoContentChunk,
                publication.table_name.as_str()
            )
            .join("_stats.json")
            .exists(),
        "repo content incremental publication should persist a stats sidecar"
    );

    let language_filters = HashSet::default();
    assert_repo_content_hit_paths(&service, "gamma", &language_filters, &["src/lib.rs"]).await;
    assert_repo_content_hit_paths(&service, "beta", &language_filters, &[]).await;
}

#[cfg(unix)]
#[tokio::test]
async fn repo_content_chunk_incremental_publish_hard_links_untouched_partition_files() {
    let temp_dir = temp_dir_or_panic();
    let service = repo_content_service(&temp_dir);
    let (changed_path, deleted_path, untouched_path) = repo_content_untouched_partition_paths();
    let first_documents = vec![
        repo_document(changed_path.as_str(), "fn alpha() {}\n", 14, 10),
        repo_document(deleted_path.as_str(), "fn beta() {}\n", 13, 10),
        repo_document(untouched_path.as_str(), "fn delta() {}\n", 14, 10),
    ];
    publish_repo_content_chunks(&service, "alpha/repo", &first_documents, Some("rev-1"))
        .await
        .unwrap_or_else(|error| panic!("first publish: {error}"));
    let first_record = repo_content_record_or_panic(
        service
            .repo_corpus_record_for_reads(SearchCorpusKind::RepoContentChunk, "alpha/repo")
            .await,
        "first repo content record",
    );
    let first_publication =
        repo_content_publication_or_panic(&first_record, "first repo content publication");
    let first_root = service.repo_publication_parquet_path(
        SearchCorpusKind::RepoContentChunk,
        first_publication.table_name.as_str(),
    );

    let changed_documents = vec![repo_document(
        changed_path.as_str(),
        "fn gamma() {}\n",
        14,
        20,
    )];
    publish_repo_content_chunks_incremental(
        &service,
        "alpha/repo",
        &changed_documents,
        &std::collections::BTreeSet::from([deleted_path.clone()]),
        Some("rev-2"),
    )
    .await
    .unwrap_or_else(|error| panic!("incremental publish: {error}"));

    let second_record = repo_content_record_or_panic(
        service
            .repo_corpus_record_for_reads(SearchCorpusKind::RepoContentChunk, "alpha/repo")
            .await,
        "second repo content record",
    );
    let second_publication =
        repo_content_publication_or_panic(&second_record, "second repo content publication");
    let second_root = service.repo_publication_parquet_path(
        SearchCorpusKind::RepoContentChunk,
        second_publication.table_name.as_str(),
    );
    let untouched_partition_id = repo_content_chunk_partition_id_for_count(
        untouched_path.as_str(),
        repo_content_chunk_partition_count_for_document_count(3),
    );
    let first_partition_path = first_root.join(format!("part_{untouched_partition_id}.parquet"));
    let second_partition_path = second_root.join(format!("part_{untouched_partition_id}.parquet"));
    let first_metadata = fs::metadata(first_partition_path)
        .unwrap_or_else(|error| panic!("first untouched partition metadata: {error}"));
    let second_metadata = fs::metadata(second_partition_path)
        .unwrap_or_else(|error| panic!("second untouched partition metadata: {error}"));

    assert_eq!(first_metadata.dev(), second_metadata.dev());
    assert_eq!(first_metadata.ino(), second_metadata.ino());
}
