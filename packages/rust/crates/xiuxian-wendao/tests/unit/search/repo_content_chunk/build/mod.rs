use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use crate::repo_index::RepoCodeDocument;
use crate::search::cache::SearchPlaneFileFingerprintScope;
use crate::search::repo_content_chunk::build::orchestration::{
    publish_repo_content_chunks, publish_repo_content_chunks_incremental,
};
use crate::search::repo_content_chunk::build::plan::{
    merge_repo_content_chunk_file_fingerprints, plan_repo_content_chunk_build,
    plan_repo_content_chunk_incremental_build, repo_content_chunk_file_fingerprints,
    versioned_repo_content_table_name,
};
use crate::search::repo_content_chunk::build::types::RepoContentChunkBuildAction;
use crate::search::{
    SearchCorpusKind, SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlaneService,
    SearchPublicationStorageFormat, SearchRepoCorpusRecord, SearchRepoPublicationInput,
    SearchRepoPublicationRecord,
};

fn repo_document(
    path: &str,
    contents: &str,
    size_bytes: u64,
    modified_unix_ms: u64,
) -> RepoCodeDocument {
    RepoCodeDocument {
        path: path.to_string(),
        language: Some("rust".to_string()),
        contents: Arc::<str>::from(contents),
        size_bytes,
        modified_unix_ms,
    }
}

fn temp_dir_or_panic() -> tempfile::TempDir {
    tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"))
}

fn repo_content_service(temp_dir: &tempfile::TempDir) -> SearchPlaneService {
    SearchPlaneService::with_paths(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        SearchManifestKeyspace::new("xiuxian:test:repo-content-build"),
        SearchMaintenancePolicy::default(),
    )
}

fn repo_content_record_or_panic(
    record: Option<SearchRepoCorpusRecord>,
    context: &str,
) -> SearchRepoCorpusRecord {
    let Some(record) = record else {
        panic!("{context}");
    };
    record
}

fn repo_content_publication_or_panic<'a>(
    record: &'a SearchRepoCorpusRecord,
    context: &str,
) -> &'a SearchRepoPublicationRecord {
    let Some(publication) = record.publication.as_ref() else {
        panic!("{context}");
    };
    publication
}

fn assert_repo_content_prewarmed(record: &SearchRepoCorpusRecord) {
    assert!(
        record
            .maintenance
            .as_ref()
            .and_then(|maintenance| maintenance.last_prewarmed_at.as_ref())
            .is_some()
    );
}

fn assert_no_lance_table(service: &SearchPlaneService, table_name: &str, context: &str) {
    assert!(
        !service
            .corpus_root(SearchCorpusKind::RepoContentChunk)
            .join(format!("{table_name}.lance"))
            .exists(),
        "{context}"
    );
}

async fn assert_repo_content_hit_paths(
    service: &SearchPlaneService,
    search_term: &str,
    language_filters: &HashSet<String>,
    expected_paths: &[&str],
) {
    let hits = service
        .search_repo_content_chunks("alpha/repo", search_term, language_filters, 5)
        .await
        .unwrap_or_else(|error| panic!("query {search_term}: {error}"));
    let actual_paths = hits.iter().map(|hit| hit.path.as_str()).collect::<Vec<_>>();
    assert_eq!(actual_paths, expected_paths);
}

#[test]
fn plan_repo_content_chunk_build_only_rewrites_changed_files() {
    let first_documents = vec![
        repo_document("src/lib.rs", "fn alpha() {}\n", 14, 10),
        repo_document("src/util.rs", "fn beta() {}\n", 13, 10),
    ];
    let first_plan = plan_repo_content_chunk_build(
        "alpha/repo",
        &first_documents,
        Some("rev-1"),
        None,
        &BTreeMap::new(),
    );
    let previous_publication = match first_plan.action {
        RepoContentChunkBuildAction::ReplaceAll { ref table_name, .. } => {
            SearchRepoPublicationRecord::new(
                SearchCorpusKind::RepoContentChunk,
                "alpha/repo",
                SearchRepoPublicationInput {
                    table_name: table_name.clone(),
                    schema_version: SearchCorpusKind::RepoContentChunk.schema_version(),
                    source_revision: Some("rev-1".to_string()),
                    table_version_id: 1,
                    row_count: 2,
                    fragment_count: 1,
                    published_at: "2026-03-24T12:00:00Z".to_string(),
                },
            )
        }
        other => panic!("unexpected first build action: {other:?}"),
    };

    let second_documents = vec![
        repo_document("src/lib.rs", "fn gamma() {}\n", 14, 20),
        repo_document("src/util.rs", "fn beta() {}\n", 13, 10),
    ];
    let second_plan = plan_repo_content_chunk_build(
        "alpha/repo",
        &second_documents,
        Some("rev-2"),
        Some(&previous_publication),
        &first_plan.file_fingerprints,
    );

    match second_plan.action {
        RepoContentChunkBuildAction::CloneAndMutate {
            base_table_name,
            target_table_name,
            replaced_paths,
            changed_payload: changed_documents,
        } => {
            assert_eq!(base_table_name, previous_publication.table_name);
            assert_ne!(target_table_name, previous_publication.table_name);
            assert_eq!(
                replaced_paths.into_iter().collect::<Vec<_>>(),
                vec!["src/lib.rs".to_string()]
            );
            assert_eq!(changed_documents.len(), 1);
            assert_eq!(changed_documents[0].path, "src/lib.rs");
        }
        other => panic!("unexpected second build action: {other:?}"),
    }
}

#[test]
fn plan_repo_content_chunk_build_reuses_table_for_revision_only_refresh() {
    let documents = vec![repo_document("src/lib.rs", "fn alpha() {}\n", 14, 10)];
    let table_name = versioned_repo_content_table_name(
        "alpha/repo",
        &repo_content_chunk_file_fingerprints(&documents),
        Some("rev-1"),
    );
    let publication = SearchRepoPublicationRecord::new(
        SearchCorpusKind::RepoContentChunk,
        "alpha/repo",
        SearchRepoPublicationInput {
            table_name: table_name.clone(),
            schema_version: SearchCorpusKind::RepoContentChunk.schema_version(),
            source_revision: Some("rev-1".to_string()),
            table_version_id: 1,
            row_count: 1,
            fragment_count: 1,
            published_at: "2026-03-24T12:00:00Z".to_string(),
        },
    );
    let plan = plan_repo_content_chunk_build(
        "alpha/repo",
        &documents,
        Some("rev-2"),
        Some(&publication),
        &repo_content_chunk_file_fingerprints(&documents),
    );

    match plan.action {
        RepoContentChunkBuildAction::RefreshPublication { table_name } => {
            assert_eq!(table_name, publication.table_name);
        }
        other => panic!("unexpected build action: {other:?}"),
    }
}

#[test]
fn plan_repo_content_chunk_incremental_build_only_rewrites_changed_files() {
    let first_documents = vec![
        repo_document("src/lib.rs", "fn alpha() {}\n", 14, 10),
        repo_document("src/util.rs", "fn beta() {}\n", 13, 10),
    ];
    let first_plan = plan_repo_content_chunk_build(
        "alpha/repo",
        &first_documents,
        Some("rev-1"),
        None,
        &BTreeMap::new(),
    );
    let previous_publication = match first_plan.action {
        RepoContentChunkBuildAction::ReplaceAll { ref table_name, .. } => {
            SearchRepoPublicationRecord::new(
                SearchCorpusKind::RepoContentChunk,
                "alpha/repo",
                SearchRepoPublicationInput {
                    table_name: table_name.clone(),
                    schema_version: SearchCorpusKind::RepoContentChunk.schema_version(),
                    source_revision: Some("rev-1".to_string()),
                    table_version_id: 1,
                    row_count: 2,
                    fragment_count: 1,
                    published_at: "2026-03-24T12:00:00Z".to_string(),
                },
            )
        }
        other => panic!("unexpected first build action: {other:?}"),
    };
    let changed_documents = vec![repo_document("src/lib.rs", "fn gamma() {}\n", 14, 20)];
    let merged_fingerprints = merge_repo_content_chunk_file_fingerprints(
        &first_plan.file_fingerprints,
        &changed_documents,
        &std::collections::BTreeSet::new(),
    );
    let second_plan = plan_repo_content_chunk_incremental_build(
        "alpha/repo",
        &changed_documents,
        &merged_fingerprints,
        Some("rev-2"),
        Some(&previous_publication),
        &first_plan.file_fingerprints,
    );

    match second_plan.action {
        RepoContentChunkBuildAction::CloneAndMutate {
            base_table_name,
            target_table_name,
            replaced_paths,
            changed_payload,
        } => {
            assert_eq!(base_table_name, previous_publication.table_name);
            assert_ne!(target_table_name, previous_publication.table_name);
            assert_eq!(
                replaced_paths.into_iter().collect::<Vec<_>>(),
                vec!["src/lib.rs".to_string()]
            );
            assert_eq!(changed_payload.len(), 1);
            assert_eq!(changed_payload[0].path, "src/lib.rs");
        }
        other => panic!("unexpected second build action: {other:?}"),
    }
}

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

#[test]
fn plan_repo_content_chunk_build_ignores_metadata_only_edits_when_contents_are_unchanged() {
    let first_documents = vec![repo_document("src/lib.rs", "fn alpha() {}\n", 14, 10)];
    let first_plan = plan_repo_content_chunk_build(
        "alpha/repo",
        &first_documents,
        Some("rev-1"),
        None,
        &BTreeMap::new(),
    );
    let previous_publication = match first_plan.action {
        RepoContentChunkBuildAction::ReplaceAll { ref table_name, .. } => {
            SearchRepoPublicationRecord::new(
                SearchCorpusKind::RepoContentChunk,
                "alpha/repo",
                SearchRepoPublicationInput {
                    table_name: table_name.clone(),
                    schema_version: SearchCorpusKind::RepoContentChunk.schema_version(),
                    source_revision: Some("rev-1".to_string()),
                    table_version_id: 1,
                    row_count: 1,
                    fragment_count: 1,
                    published_at: "2026-03-24T12:00:00Z".to_string(),
                },
            )
        }
        other => panic!("unexpected first build action: {other:?}"),
    };

    let second_documents = vec![repo_document("src/lib.rs", "fn alpha() {}\n", 14, 20)];
    let second_plan = plan_repo_content_chunk_build(
        "alpha/repo",
        &second_documents,
        Some("rev-2"),
        Some(&previous_publication),
        &first_plan.file_fingerprints,
    );

    let first_table_name = versioned_repo_content_table_name(
        "alpha/repo",
        &first_plan.file_fingerprints,
        Some("rev-2"),
    );
    let second_table_name = versioned_repo_content_table_name(
        "alpha/repo",
        &second_plan.file_fingerprints,
        Some("rev-2"),
    );
    assert_eq!(first_table_name, second_table_name);
    assert_eq!(
        first_plan
            .file_fingerprints
            .get("src/lib.rs")
            .and_then(|fingerprint| fingerprint.blake3.as_deref()),
        second_plan
            .file_fingerprints
            .get("src/lib.rs")
            .and_then(|fingerprint| fingerprint.blake3.as_deref())
    );
    assert_eq!(
        second_plan
            .file_fingerprints
            .get("src/lib.rs")
            .map(|fingerprint| fingerprint.modified_unix_ms),
        Some(20)
    );

    match second_plan.action {
        RepoContentChunkBuildAction::RefreshPublication { table_name } => {
            assert_eq!(table_name, previous_publication.table_name);
        }
        other => panic!("unexpected second build action: {other:?}"),
    }
}
