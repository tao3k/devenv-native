//! Incremental repo-content mutation benchmark fixtures and samples.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::PathBuf;
use std::time::Instant;

use crate::repo_index::RepoCodeDocument;
use crate::search::repo_content_chunk::{
    publish_repo_content_chunks_incremental_profiled,
    repo_content_chunk_partition_count_for_document_count,
    repo_content_chunk_partition_id_for_path,
};
use crate::search::{
    SearchCorpusKind, SearchFileFingerprint, SearchMaintenancePolicy, SearchManifestKeyspace,
    SearchPlaneFileFingerprintScope, SearchPlaneService,
};

use super::fixture::{
    assert_minimum_benchmark_documents, benchmark_suffix, build_runtime, copy_dir_recursive,
    create_dir_all, expected_row_count, publish_base_repo_content_fixture,
    repo_content_benchmark_paths, repo_content_benchmark_service, repo_content_document,
    repo_content_fingerprints, repo_content_path, unique_query_token,
};
use super::samples::RepoContentParquetMutationBenchmarkSnapshot;

/// Synthetic fixture for measuring repo-content Parquet clone-and-mutate cost.
#[derive(Debug)]
pub struct RepoContentParquetMutationBenchmarkFixture {
    root: PathBuf,
    project_root: PathBuf,
    template_storage_root: PathBuf,
    manifest_keyspace: SearchManifestKeyspace,
    repo_id: String,
    base_document_count: usize,
    expected_row_count: u64,
    base_fingerprints: BTreeMap<String, SearchFileFingerprint>,
    changed_documents: Vec<RepoCodeDocument>,
    deleted_paths: BTreeSet<String>,
    touched_base_documents_by_partition: BTreeMap<String, usize>,
    added_query: String,
    added_path: String,
    deleted_query: String,
}

/// One prepared benchmark iteration with a copied base publication state.
#[derive(Debug)]
pub struct RepoContentParquetMutationBenchmarkIteration {
    root: PathBuf,
    project_root: PathBuf,
    storage_root: PathBuf,
    manifest_keyspace: SearchManifestKeyspace,
    repo_id: String,
    expected_row_count: u64,
    base_document_count: usize,
    base_fingerprints: BTreeMap<String, SearchFileFingerprint>,
    changed_documents: Vec<RepoCodeDocument>,
    deleted_paths: BTreeSet<String>,
    touched_base_documents_by_partition: BTreeMap<String, usize>,
    added_query: String,
    deleted_query: String,
}

struct RepoContentMutationInputs {
    changed_documents: Vec<RepoCodeDocument>,
    deleted_paths: BTreeSet<String>,
    touched_base_documents_by_partition: BTreeMap<String, usize>,
    added_query: String,
    added_path: String,
    deleted_query: String,
}

impl RepoContentParquetMutationBenchmarkFixture {
    /// Build one synthetic base publication and keep it as the template for
    /// repeated incremental clone-and-mutate samples.
    ///
    /// # Panics
    ///
    /// Panics when the fixture directories or the base publication cannot be
    /// created.
    #[must_use]
    pub fn synthetic(base_document_count: usize) -> Self {
        assert_minimum_benchmark_documents(base_document_count, "repo-content parquet benchmark");
        let suffix = benchmark_suffix();
        let paths = repo_content_benchmark_paths(
            format!("xiuxian-wendao-repo-publication-parquet-bench-{suffix}").as_str(),
            "template_search_plane",
        );
        let manifest_keyspace =
            SearchManifestKeyspace::new(format!("xiuxian:bench:repo-publication:{suffix}"));
        let repo_id = "alpha/repo".to_string();
        let service = repo_content_benchmark_service(&paths, &manifest_keyspace);
        publish_base_repo_content_fixture(&service, repo_id.as_str(), base_document_count);
        let base_fingerprints = repo_content_fingerprints(&service, repo_id.as_str());
        let mutation_inputs = repo_content_mutation_inputs(base_document_count, &base_fingerprints);
        Self {
            root: paths.root,
            project_root: paths.project_root,
            template_storage_root: paths.storage_root,
            manifest_keyspace,
            repo_id,
            base_document_count,
            expected_row_count: expected_row_count(base_document_count),
            base_fingerprints,
            changed_documents: mutation_inputs.changed_documents,
            deleted_paths: mutation_inputs.deleted_paths,
            touched_base_documents_by_partition: mutation_inputs
                .touched_base_documents_by_partition,
            added_query: mutation_inputs.added_query,
            added_path: mutation_inputs.added_path,
            deleted_query: mutation_inputs.deleted_query,
        }
    }

    /// Prepare one benchmark iteration by copying the base publication state.
    ///
    /// This setup step is intended to stay outside the timed mutation sample.
    ///
    /// # Panics
    ///
    /// Panics when the copied benchmark state cannot be created.
    #[must_use]
    pub fn prepare_iteration(&self) -> RepoContentParquetMutationBenchmarkIteration {
        let suffix = benchmark_suffix();
        let iteration_root = self.root.join(format!("iteration-{suffix}"));
        let storage_root = iteration_root.join("search_plane");
        create_dir_all(iteration_root.as_path());
        copy_dir_recursive(self.template_storage_root.as_path(), storage_root.as_path())
            .unwrap_or_else(|error| {
                panic!(
                    "copy repo publication benchmark fixture {} -> {}: {error}",
                    self.template_storage_root.display(),
                    storage_root.display()
                )
            });
        RepoContentParquetMutationBenchmarkIteration {
            root: iteration_root,
            project_root: self.project_root.clone(),
            storage_root,
            manifest_keyspace: self.manifest_keyspace.clone(),
            repo_id: self.repo_id.clone(),
            expected_row_count: self.expected_row_count,
            base_document_count: self.base_document_count,
            base_fingerprints: self.base_fingerprints.clone(),
            changed_documents: self.changed_documents.clone(),
            deleted_paths: self.deleted_paths.clone(),
            touched_base_documents_by_partition: self.touched_base_documents_by_partition.clone(),
            added_query: self.added_query.clone(),
            deleted_query: self.deleted_query.clone(),
        }
    }

    /// Expected row count after the synthetic incremental mutation.
    #[must_use]
    pub fn expected_row_count(&self) -> u64 {
        self.expected_row_count
    }

    /// Path introduced by the added-document mutation.
    #[must_use]
    pub fn added_path(&self) -> &str {
        self.added_path.as_str()
    }
}

impl Drop for RepoContentParquetMutationBenchmarkFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

impl RepoContentParquetMutationBenchmarkIteration {
    /// Run the incremental repo-content mutation sample and return the measured
    /// publication summary plus query-based verification paths.
    ///
    /// # Panics
    ///
    /// Panics when the synthetic incremental mutation or the follow-up reads
    /// fail.
    #[must_use]
    pub fn run(self) -> RepoContentParquetMutationBenchmarkSnapshot {
        let service = SearchPlaneService::with_paths(
            self.project_root.clone(),
            self.storage_root.clone(),
            self.manifest_keyspace.clone(),
            SearchMaintenancePolicy::default(),
        );
        let runtime = build_runtime();
        runtime.block_on(async {
            service
                .set_file_fingerprints(
                    SearchPlaneFileFingerprintScope::repo_corpus(
                        SearchCorpusKind::RepoContentChunk,
                        self.repo_id.as_str(),
                    ),
                    &self.base_fingerprints,
                )
                .await;
        });
        let started = Instant::now();
        let publish_profile = runtime.block_on(async {
            publish_repo_content_chunks_incremental_profiled(
                &service,
                self.repo_id.as_str(),
                &self.changed_documents,
                &self.deleted_paths,
                Some("rev-2"),
            )
            .await
            .unwrap_or_else(|error| panic!("publish mutated repo-content fixture: {error}"))
        });
        let elapsed = started.elapsed();
        let (row_count, added_query_paths, deleted_query_paths) =
            self.collect_verification_paths(&runtime, &service);
        assert_eq!(
            row_count, self.expected_row_count,
            "repo-content parquet benchmark row count drifted from the synthetic fixture"
        );
        RepoContentParquetMutationBenchmarkSnapshot {
            base_document_count: self.base_document_count,
            changed_document_count: self.changed_documents.len(),
            deleted_path_count: self.deleted_paths.len(),
            partition_bucket_count: repo_content_chunk_partition_count_for_document_count(
                self.base_document_count,
            ),
            touched_partition_count: self.touched_base_documents_by_partition.len(),
            touched_base_document_count: self
                .touched_base_documents_by_partition
                .values()
                .copied()
                .sum(),
            touched_base_documents_by_partition: self.touched_base_documents_by_partition.clone(),
            row_count,
            elapsed,
            added_query_paths,
            deleted_query_paths,
            publish_profile,
        }
    }

    fn collect_verification_paths(
        &self,
        runtime: &tokio::runtime::Runtime,
        service: &SearchPlaneService,
    ) -> (u64, Vec<String>, Vec<String>) {
        runtime.block_on(async {
            let record = service
                .repo_corpus_record_for_reads(
                    SearchCorpusKind::RepoContentChunk,
                    self.repo_id.as_str(),
                )
                .await
                .unwrap_or_else(|| {
                    panic!(
                        "repo-content parquet benchmark missing publication for `{}`",
                        self.repo_id
                    )
                });
            let row_count = record
                .publication
                .as_ref()
                .unwrap_or_else(|| {
                    panic!(
                        "repo-content parquet benchmark missing publication payload for `{}`",
                        self.repo_id
                    )
                })
                .row_count;
            let added_query_paths = service
                .search_repo_content_chunks(
                    self.repo_id.as_str(),
                    self.added_query.as_str(),
                    &HashSet::new(),
                    5,
                )
                .await
                .unwrap_or_else(|error| panic!("query added benchmark token: {error}"))
                .into_iter()
                .map(|hit| hit.path)
                .collect::<Vec<_>>();
            let deleted_query_paths = service
                .search_repo_content_chunks(
                    self.repo_id.as_str(),
                    self.deleted_query.as_str(),
                    &HashSet::new(),
                    5,
                )
                .await
                .unwrap_or_else(|error| panic!("query deleted benchmark token: {error}"))
                .into_iter()
                .map(|hit| hit.path)
                .collect::<Vec<_>>();
            (row_count, added_query_paths, deleted_query_paths)
        })
    }
}

impl Drop for RepoContentParquetMutationBenchmarkIteration {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn repo_content_mutation_inputs(
    base_document_count: usize,
    base_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
) -> RepoContentMutationInputs {
    let changed_indexes = changed_existing_indexes(base_document_count);
    let deleted_index = deleted_index(base_document_count, &changed_indexes);
    let added_index = base_document_count;
    let changed_documents = changed_repo_content_documents(changed_indexes, added_index);
    let deleted_paths = BTreeSet::from([repo_content_path(deleted_index)]);
    let touched_partition_ids = touched_partition_ids(
        changed_documents.as_slice(),
        &deleted_paths,
        base_fingerprints,
    );
    RepoContentMutationInputs {
        touched_base_documents_by_partition: touched_base_documents_by_partition(
            base_fingerprints,
            &touched_partition_ids,
        ),
        changed_documents,
        deleted_paths,
        added_query: unique_query_token(110_000 + added_index),
        added_path: repo_content_path(added_index),
        deleted_query: unique_query_token(deleted_index),
    }
}

fn changed_repo_content_documents(
    changed_indexes: [usize; 2],
    added_index: usize,
) -> Vec<RepoCodeDocument> {
    vec![
        repo_content_document(changed_indexes[0], 70_000 + changed_indexes[0]),
        repo_content_document(changed_indexes[1], 90_000 + changed_indexes[1]),
        repo_content_document(added_index, 110_000 + added_index),
    ]
}

fn changed_existing_indexes(base_document_count: usize) -> [usize; 2] {
    let first = base_document_count / 8;
    let second = (base_document_count / 2).max(first + 1);
    [first, second]
}

fn deleted_index(base_document_count: usize, changed_indexes: &[usize; 2]) -> usize {
    let mut candidate = base_document_count / 3;
    while changed_indexes.contains(&candidate) {
        candidate += 1;
        if candidate >= base_document_count {
            candidate = 0;
        }
    }
    candidate
}

fn touched_partition_ids(
    changed_documents: &[RepoCodeDocument],
    deleted_paths: &BTreeSet<String>,
    base_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
) -> BTreeSet<String> {
    let mut touched = BTreeSet::new();
    let partition_count =
        repo_content_chunk_partition_count_for_document_count(base_fingerprints.len());
    for document in changed_documents {
        touched.insert(repo_content_chunk_partition_id_for_path(
            document.path.as_str(),
            base_fingerprints,
            partition_count,
        ));
    }
    for path in deleted_paths {
        touched.insert(repo_content_chunk_partition_id_for_path(
            path.as_str(),
            base_fingerprints,
            partition_count,
        ));
    }
    touched
}

fn touched_base_documents_by_partition(
    base_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
    touched_partition_ids: &BTreeSet<String>,
) -> BTreeMap<String, usize> {
    let mut distribution = BTreeMap::<String, usize>::new();
    for partition_id in touched_partition_ids {
        distribution.insert(partition_id.clone(), 0);
    }
    for (path, fingerprint) in base_fingerprints {
        add_fingerprint_to_touched_distribution(
            &mut distribution,
            path,
            fingerprint,
            base_fingerprints,
        );
    }
    distribution
}

fn add_fingerprint_to_touched_distribution(
    distribution: &mut BTreeMap<String, usize>,
    path: &str,
    fingerprint: &SearchFileFingerprint,
    base_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
) {
    let partition_count =
        repo_content_chunk_partition_count_for_document_count(base_fingerprints.len());
    let partition_id = fingerprint.partition_id.clone().unwrap_or_else(|| {
        repo_content_chunk_partition_id_for_path(path, base_fingerprints, partition_count)
    });
    if let Some(document_count) = distribution.get_mut(partition_id.as_str()) {
        *document_count += 1;
    }
}
