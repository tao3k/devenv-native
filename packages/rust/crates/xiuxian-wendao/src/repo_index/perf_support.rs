use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::repo_index::state::{collect_code_documents, collect_incremental_code_documents};
use crate::search::{
    SearchCorpusKind, SearchFileFingerprint, SearchMaintenancePolicy, SearchManifestKeyspace,
    SearchPlaneService, SearchPublicationStorageFormat, SearchRepoCorpusRecord,
    SearchRepoPublicationInput, SearchRepoPublicationRecord, repo_content_chunk_file_fingerprints,
};

static REPO_BOOTSTRAP_BENCH_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Summary of one full repo code-document collection benchmark sample.
#[derive(Debug, Clone)]
pub struct RepoCodeDocumentBenchmarkSnapshot {
    /// Number of supported code documents read from the checkout.
    pub document_count: usize,
    /// Repo-content fingerprints produced from the collected documents.
    pub file_fingerprints: BTreeMap<String, SearchFileFingerprint>,
}

/// Summary of one incremental repo code-document collection benchmark sample.
#[derive(Debug, Clone)]
pub struct RepoCodeDocumentIncrementalBenchmarkSnapshot {
    /// Number of changed supported code documents reread from the checkout.
    pub changed_document_count: usize,
    /// Number of paths treated as deleted or removed from the current snapshot.
    pub deleted_path_count: usize,
    /// Repo-content fingerprints after merging unchanged prior state with the
    /// changed-document snapshot.
    pub file_fingerprints: BTreeMap<String, SearchFileFingerprint>,
}

/// Synthetic bootstrap fixture for measuring repo-index status recovery from
/// per-repo durable records without a global snapshot.
#[derive(Debug)]
pub struct RepoBootstrapBenchmarkFixture {
    root: PathBuf,
    project_root: PathBuf,
    storage_root: PathBuf,
    manifest_keyspace: SearchManifestKeyspace,
    repo_ids: Vec<String>,
}

/// Collect the full supported repo code-document set for benchmark sampling.
#[must_use]
pub fn benchmark_collect_full_repo_code_documents(
    root: &Path,
) -> RepoCodeDocumentBenchmarkSnapshot {
    let documents = collect_code_documents(root, || false).unwrap_or_default();
    RepoCodeDocumentBenchmarkSnapshot {
        document_count: documents.len(),
        file_fingerprints: repo_content_chunk_file_fingerprints(&documents),
    }
}

/// Collect only changed supported repo code documents and merge them onto a
/// prior repo-content fingerprint map for benchmark sampling.
#[must_use]
pub fn benchmark_collect_incremental_repo_code_documents(
    root: &Path,
    changed_paths: &BTreeSet<String>,
    deleted_paths: &BTreeSet<String>,
    previous_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
) -> RepoCodeDocumentIncrementalBenchmarkSnapshot {
    let collection =
        collect_incremental_code_documents(root, changed_paths, deleted_paths, || false)
            .unwrap_or_else(|| unreachable!("benchmark collection does not cancel"));
    let mut file_fingerprints = previous_fingerprints.clone();
    for path in &collection.deleted_paths {
        file_fingerprints.remove(path);
    }
    file_fingerprints.extend(repo_content_chunk_file_fingerprints(
        &collection.changed_documents,
    ));
    RepoCodeDocumentIncrementalBenchmarkSnapshot {
        changed_document_count: collection.changed_documents.len(),
        deleted_path_count: collection.deleted_paths.len(),
        file_fingerprints,
    }
}

impl RepoBootstrapBenchmarkFixture {
    /// Build one synthetic repo-bootstrap fixture with per-repo durable records
    /// and no global snapshot file.
    ///
    /// # Panics
    ///
    /// Panics when the synthetic benchmark project root cannot be created.
    #[must_use]
    pub fn synthetic(repo_count: usize) -> Self {
        let suffix = REPO_BOOTSTRAP_BENCH_COUNTER.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("xiuxian-wendao-repo-bootstrap-bench-{suffix}"));
        let project_root = root.join("project");
        let storage_root = root.join("search_plane");
        let _ = std::fs::remove_dir_all(&root);
        if let Err(error) = std::fs::create_dir_all(&project_root) {
            panic!(
                "create benchmark project root {}: {error}",
                project_root.display()
            );
        }
        let manifest_keyspace =
            SearchManifestKeyspace::new(format!("xiuxian:bench:repo-bootstrap:{suffix}"));
        let service = SearchPlaneService::with_paths(
            project_root.clone(),
            storage_root.clone(),
            manifest_keyspace.clone(),
            SearchMaintenancePolicy::default(),
        );
        let repo_ids = (0..repo_count)
            .map(|index| format!("repo-{index:05}"))
            .collect::<Vec<_>>();
        for (index, repo_id) in repo_ids.iter().enumerate() {
            for corpus in [
                SearchCorpusKind::RepoEntity,
                SearchCorpusKind::RepoContentChunk,
            ] {
                service.persist_local_repo_corpus_record(&SearchRepoCorpusRecord::new(
                    corpus,
                    repo_id.clone(),
                    None,
                    Some(SearchRepoPublicationRecord::new_with_storage_format(
                        corpus,
                        repo_id.clone(),
                        SearchRepoPublicationInput {
                            table_name: format!("{}_table_{index:05}", corpus.as_str()),
                            schema_version: 1,
                            source_revision: Some(format!("rev-{index:05}")),
                            table_version_id: 1,
                            row_count: 32,
                            fragment_count: 1,
                            published_at: "2026-04-15T00:00:00Z".to_string(),
                        },
                        SearchPublicationStorageFormat::Parquet,
                    )),
                ));
            }
        }
        Self {
            root,
            project_root,
            storage_root,
            manifest_keyspace,
            repo_ids,
        }
    }

    /// Count the repo-index bootstrap statuses recovered from durable records.
    #[must_use]
    pub fn bootstrap_status_count(&self) -> usize {
        SearchPlaneService::with_paths(
            self.project_root.clone(),
            self.storage_root.clone(),
            self.manifest_keyspace.clone(),
            SearchMaintenancePolicy::default(),
        )
        .repo_index_bootstrap_statuses(self.repo_ids.as_slice())
        .len()
    }

    /// Number of synthetic repos materialized in the fixture.
    #[must_use]
    pub fn repo_count(&self) -> usize {
        self.repo_ids.len()
    }

    /// Whether a legacy global snapshot file exists for this fixture.
    #[must_use]
    pub fn snapshot_file_exists(&self) -> bool {
        SearchPlaneService::with_paths(
            self.project_root.clone(),
            self.storage_root.clone(),
            self.manifest_keyspace.clone(),
            SearchMaintenancePolicy::default(),
        )
        .repo_corpus_snapshot_json_path()
        .exists()
    }
}

impl Drop for RepoBootstrapBenchmarkFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[cfg(test)]
#[path = "../../tests/unit/repo_index/perf_support.rs"]
mod tests;
