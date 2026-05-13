use crate::search::SearchCorpusKind;
use crate::search::manifest::{
    SearchPublicationStorageFormat, SearchRepoPublicationInput, SearchRepoPublicationRecord,
    build_repo_publication_epoch,
};

#[test]
fn repo_publication_id_changes_when_table_version_changes() {
    let first = SearchRepoPublicationRecord::new(
        SearchCorpusKind::RepoEntity,
        "alpha/repo",
        SearchRepoPublicationInput {
            table_name: "repo_entity_repo_alpha".to_string(),
            schema_version: 1,
            source_revision: Some("rev-1".to_string()),
            table_version_id: 7,
            row_count: 10,
            fragment_count: 2,
            published_at: "2026-03-23T12:00:00Z".to_string(),
        },
    );
    let second = SearchRepoPublicationRecord::new(
        SearchCorpusKind::RepoEntity,
        "alpha/repo",
        SearchRepoPublicationInput {
            table_name: "repo_entity_repo_alpha".to_string(),
            schema_version: 1,
            source_revision: Some("rev-1".to_string()),
            table_version_id: 8,
            row_count: 10,
            fragment_count: 2,
            published_at: "2026-03-23T12:00:00Z".to_string(),
        },
    );

    assert_ne!(first.publication_id, second.publication_id);
    assert_ne!(first.active_epoch_value(), second.active_epoch_value());
    assert_ne!(first.cache_version(), second.cache_version());
}

#[test]
fn repo_publication_cache_version_is_stable_for_same_publication() {
    let first = SearchRepoPublicationRecord::new(
        SearchCorpusKind::RepoContentChunk,
        "alpha/repo",
        SearchRepoPublicationInput {
            table_name: "repo_content_chunk_repo_alpha".to_string(),
            schema_version: 1,
            source_revision: Some("rev-7".to_string()),
            table_version_id: 3,
            row_count: 42,
            fragment_count: 1,
            published_at: "2026-03-23T12:00:00Z".to_string(),
        },
    );
    let second = SearchRepoPublicationRecord::new(
        SearchCorpusKind::RepoContentChunk,
        "alpha/repo",
        SearchRepoPublicationInput {
            table_name: "repo_content_chunk_repo_alpha".to_string(),
            schema_version: 1,
            source_revision: Some("rev-7".to_string()),
            table_version_id: 3,
            row_count: 42,
            fragment_count: 1,
            published_at: "2026-03-23T12:00:00Z".to_string(),
        },
    );

    assert_eq!(first.publication_id, second.publication_id);
    assert_eq!(first.active_epoch_value(), second.active_epoch_value());
    assert_eq!(first.cache_version(), second.cache_version());
}

#[test]
fn repo_publication_id_changes_when_source_revision_changes() {
    let first = SearchRepoPublicationRecord::new(
        SearchCorpusKind::RepoContentChunk,
        "alpha/repo",
        SearchRepoPublicationInput {
            table_name: "repo_content_chunk_repo_alpha".to_string(),
            schema_version: 1,
            source_revision: Some("rev-1".to_string()),
            table_version_id: 3,
            row_count: 42,
            fragment_count: 1,
            published_at: "2026-03-23T12:00:00Z".to_string(),
        },
    );
    let second = SearchRepoPublicationRecord::new(
        SearchCorpusKind::RepoContentChunk,
        "alpha/repo",
        SearchRepoPublicationInput {
            table_name: "repo_content_chunk_repo_alpha".to_string(),
            schema_version: 1,
            source_revision: Some("rev-2".to_string()),
            table_version_id: 3,
            row_count: 42,
            fragment_count: 1,
            published_at: "2026-03-23T12:00:00Z".to_string(),
        },
    );

    assert_ne!(first.publication_id, second.publication_id);
    assert_ne!(first.active_epoch_value(), second.active_epoch_value());
    assert_ne!(first.cache_version(), second.cache_version());
}

#[test]
fn repo_publication_active_epoch_falls_back_for_legacy_payloads() {
    let legacy = SearchRepoPublicationRecord {
        corpus: SearchCorpusKind::RepoEntity,
        repo_id: "alpha/repo".to_string(),
        active_epoch: None,
        publication_id: "legacy-publication".to_string(),
        table_name: "repo_entity_repo_alpha".to_string(),
        table_version_id: 7,
        schema_version: 1,
        storage_format: SearchPublicationStorageFormat::Lance,
        source_revision: Some("rev-1".to_string()),
        row_count: 10,
        fragment_count: 2,
        published_at: "2026-03-23T12:00:00Z".to_string(),
    };

    assert_eq!(
        legacy.active_epoch_value(),
        build_repo_publication_epoch("legacy-publication")
    );
}

#[test]
fn repo_publication_id_changes_when_storage_format_changes() {
    let lance = SearchRepoPublicationRecord::new(
        SearchCorpusKind::RepoEntity,
        "alpha/repo",
        SearchRepoPublicationInput {
            table_name: "repo_entity_repo_alpha".to_string(),
            schema_version: 1,
            source_revision: Some("rev-1".to_string()),
            table_version_id: 7,
            row_count: 10,
            fragment_count: 2,
            published_at: "2026-03-23T12:00:00Z".to_string(),
        },
    );
    let parquet = SearchRepoPublicationRecord::new_with_storage_format(
        SearchCorpusKind::RepoEntity,
        "alpha/repo",
        SearchRepoPublicationInput {
            table_name: "repo_entity_repo_alpha".to_string(),
            schema_version: 1,
            source_revision: Some("rev-1".to_string()),
            table_version_id: 7,
            row_count: 10,
            fragment_count: 2,
            published_at: "2026-03-23T12:00:00Z".to_string(),
        },
        SearchPublicationStorageFormat::Parquet,
    );

    assert_ne!(lance.publication_id, parquet.publication_id);
    assert!(parquet.is_parquet_query_readable());
    assert!(!lance.is_parquet_query_readable());
}

#[test]
fn repo_publication_defaults_legacy_storage_format_when_field_is_missing() {
    let legacy_payload = serde_json::json!({
        "corpus": "repo_entity",
        "repo_id": "alpha/repo",
        "active_epoch": 11,
        "publication_id": "legacy-publication",
        "table_name": "repo_entity_repo_alpha",
        "table_version_id": 7,
        "schema_version": 1,
        "source_revision": "rev-1",
        "row_count": 10,
        "fragment_count": 2,
        "published_at": "2026-03-23T12:00:00Z"
    });

    let record: SearchRepoPublicationRecord = serde_json::from_value(legacy_payload)
        .unwrap_or_else(|error| panic!("legacy payload should deserialize: {error}"));

    assert_eq!(record.storage_format, SearchPublicationStorageFormat::Lance);
}
