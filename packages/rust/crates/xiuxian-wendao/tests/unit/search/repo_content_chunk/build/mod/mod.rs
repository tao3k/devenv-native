use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use crate::repo_index::RepoCodeDocument;
use crate::search::repo_content_chunk::build::partitions::{
    repo_content_chunk_partition_count_for_document_count,
    repo_content_chunk_partition_id_for_count,
};
use crate::search::{
    SearchCorpusKind, SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlaneService,
    SearchRepoCorpusRecord, SearchRepoPublicationRecord,
};

mod incremental;
mod migration;
mod planning;
mod publication;

fn repo_document(
    path: &str,
    contents: &str,
    size_bytes: u64,
    modified_unix_ms: u64,
) -> RepoCodeDocument {
    RepoCodeDocument {
        path: path.to_string().into(),
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

#[cfg(unix)]
fn repo_content_untouched_partition_paths() -> (String, String, String) {
    let changed_path = "src/lib.rs".to_string();
    let deleted_path = "src/util.rs".to_string();
    let partition_count = repo_content_chunk_partition_count_for_document_count(3);
    let touched_partitions = std::collections::BTreeSet::from([
        repo_content_chunk_partition_id_for_count(changed_path.as_str(), partition_count),
        repo_content_chunk_partition_id_for_count(deleted_path.as_str(), partition_count),
    ]);
    for index in 0..256 {
        let candidate = format!("src/untouched_{index}.rs");
        if !touched_partitions.contains(
            repo_content_chunk_partition_id_for_count(candidate.as_str(), partition_count).as_str(),
        ) {
            return (changed_path, deleted_path, candidate);
        }
    }
    panic!("failed to find untouched repo-content path outside touched partitions");
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
