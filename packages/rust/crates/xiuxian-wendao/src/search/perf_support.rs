use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::repo_index::RepoCodeDocument;
use crate::search::repo_content_chunk::{
    RepoContentChunkIncrementalPublishProfile, publish_repo_content_chunks,
    publish_repo_content_chunks_incremental_profiled,
    repo_content_chunk_partition_count_for_document_count,
    repo_content_chunk_partition_id_for_path,
};
use crate::search::repo_search::search_repo_content_batch;
use crate::search::{
    SearchCorpusKind, SearchFileFingerprint, SearchMaintenancePolicy, SearchManifestKeyspace,
    SearchPlaneFileFingerprintScope, SearchPlaneService,
    resolve_search_plane_cache_connection_target,
};
use xiuxian_wendao_runtime::transport::RepoSearchFlightRequest;

static REPO_PUBLICATION_BENCH_COUNTER: AtomicU64 = AtomicU64::new(1);
const BENCH_LINE_COUNT: usize = 12;

/// Result of one synthetic repo-content clone-and-mutate publication sample.
#[derive(Debug, Clone)]
pub struct RepoContentParquetMutationBenchmarkSnapshot {
    /// Number of documents in the base publication.
    pub base_document_count: usize,
    /// Number of changed documents applied during the incremental mutation.
    pub changed_document_count: usize,
    /// Number of deleted paths applied during the incremental mutation.
    pub deleted_path_count: usize,
    /// Total configured partition bucket count for repo-content publications.
    pub partition_bucket_count: usize,
    /// Number of touched partitions involved in the incremental mutation.
    pub touched_partition_count: usize,
    /// Number of base documents that already lived inside the touched partitions.
    pub touched_base_document_count: usize,
    /// Base-document distribution across the touched partitions.
    pub touched_base_documents_by_partition: BTreeMap<String, usize>,
    /// Row count reported by the resulting repo-content publication.
    pub row_count: u64,
    /// Time spent inside the incremental clone-and-mutate publish call.
    pub elapsed: Duration,
    /// Paths returned for the added-document verification query.
    pub added_query_paths: Vec<String>,
    /// Paths returned for the deleted-document verification query.
    pub deleted_query_paths: Vec<String>,
    /// Phase-level timing and count breakdown for the incremental publish call.
    pub publish_profile: RepoContentChunkIncrementalPublishProfile,
}

/// Result of one synthetic large repo-content query sample.
#[derive(Debug, Clone)]
pub struct RepoContentQueryBenchmarkSnapshot {
    /// Number of synthetic repo-content documents in the publication.
    pub base_document_count: usize,
    /// Row count reported by the published repo-content corpus.
    pub publication_row_count: u64,
    /// Unique query token used for the benchmark sample.
    pub query_token: String,
    /// Expected repo-relative path for the unique query token.
    pub expected_path: String,
    /// Time spent on the first query after one fresh service start.
    pub cold_query_elapsed: Duration,
    /// Time spent on the second query on the same service instance.
    pub hot_query_elapsed: Duration,
    /// Time spent materializing the repo-search Arrow/Flight batch.
    pub flight_batch_elapsed: Duration,
    /// Number of hits returned by the first query.
    pub cold_query_hit_count: usize,
    /// Number of hits returned by the second query.
    pub hot_query_hit_count: usize,
    /// Number of rows emitted by the Flight batch surface.
    pub flight_batch_row_count: usize,
    /// First path returned by the first query.
    pub cold_first_path: Option<String>,
    /// First path returned by the second query.
    pub hot_first_path: Option<String>,
    /// Local query-engine kind compiled into this benchmark run.
    pub query_engine_kind: &'static str,
    /// Persisted metadata surface available to cold-start reads.
    pub persisted_metadata_backend: &'static str,
    /// Whether this benchmark run resolved a Valkey metadata target from
    /// config or env.
    pub valkey_target_configured: bool,
}

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
    base_fingerprints: std::collections::BTreeMap<String, SearchFileFingerprint>,
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
    base_fingerprints: std::collections::BTreeMap<String, SearchFileFingerprint>,
    changed_documents: Vec<RepoCodeDocument>,
    deleted_paths: BTreeSet<String>,
    touched_base_documents_by_partition: BTreeMap<String, usize>,
    added_query: String,
    deleted_query: String,
}

/// Synthetic fixture for measuring steady-state repo-backed query cost.
#[derive(Debug)]
pub struct RepoContentQueryBenchmarkFixture {
    root: PathBuf,
    project_root: PathBuf,
    storage_root: PathBuf,
    manifest_keyspace: SearchManifestKeyspace,
    repo_id: String,
    base_document_count: usize,
    expected_row_count: u64,
    query_token: String,
    expected_path: String,
    valkey_target_configured: bool,
}

/// One prepared repo-content query benchmark iteration.
pub struct RepoContentQueryBenchmarkIteration {
    service: SearchPlaneService,
    runtime: tokio::runtime::Runtime,
    repo_id: String,
    base_document_count: usize,
    expected_row_count: u64,
    query_token: String,
    expected_path: String,
    valkey_target_configured: bool,
}

/// One measured repo-content query sample.
#[derive(Debug, Clone)]
pub struct RepoContentQueryBenchmarkSample {
    /// Time spent executing the sample.
    pub elapsed: Duration,
    /// Number of hits returned by the sample.
    pub hit_count: usize,
    /// First repo-relative path returned by the sample.
    pub first_path: Option<String>,
}

/// One measured repo-search Flight batch sample.
#[derive(Debug, Clone)]
pub struct RepoContentFlightBatchBenchmarkSample {
    /// Time spent executing the sample.
    pub elapsed: Duration,
    /// Number of rows emitted by the sample batch.
    pub row_count: usize,
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
        assert!(
            base_document_count >= 8,
            "repo-content parquet benchmark requires at least 8 documents"
        );
        let suffix = REPO_PUBLICATION_BENCH_COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "xiuxian-wendao-repo-publication-parquet-bench-{suffix}"
        ));
        let project_root = root.join("project");
        let template_storage_root = root.join("template_search_plane");
        let _ = std::fs::remove_dir_all(&root);
        create_dir_all(project_root.as_path());
        let manifest_keyspace =
            SearchManifestKeyspace::new(format!("xiuxian:bench:repo-publication:{suffix}"));
        let repo_id = "alpha/repo".to_string();
        let service = SearchPlaneService::with_paths(
            project_root.clone(),
            template_storage_root.clone(),
            manifest_keyspace.clone(),
            SearchMaintenancePolicy::default(),
        );
        let base_documents = (0..base_document_count)
            .map(|index| repo_content_document(index, index))
            .collect::<Vec<_>>();
        build_runtime().block_on(async {
            publish_repo_content_chunks(&service, repo_id.as_str(), &base_documents, Some("rev-1"))
                .await
                .unwrap_or_else(|error| panic!("publish base repo-content fixture: {error}"));
        });
        let base_fingerprints = build_runtime().block_on(async {
            service
                .file_fingerprints(SearchPlaneFileFingerprintScope::repo_corpus(
                    SearchCorpusKind::RepoContentChunk,
                    repo_id.as_str(),
                ))
                .await
        });

        let changed_indexes = changed_existing_indexes(base_document_count);
        let deleted_index = deleted_index(base_document_count, &changed_indexes);
        let added_index = base_document_count;
        let changed_documents = vec![
            repo_content_document(changed_indexes[0], 70_000 + changed_indexes[0]),
            repo_content_document(changed_indexes[1], 90_000 + changed_indexes[1]),
            repo_content_document(added_index, 110_000 + added_index),
        ];
        let deleted_paths = BTreeSet::from([repo_content_path(deleted_index)]);
        let touched_partition_ids = touched_partition_ids(
            changed_documents.as_slice(),
            &deleted_paths,
            &base_fingerprints,
        );
        let touched_base_documents_by_partition =
            touched_base_documents_by_partition(&base_fingerprints, &touched_partition_ids);
        Self {
            root,
            project_root,
            template_storage_root,
            manifest_keyspace,
            repo_id,
            base_document_count,
            expected_row_count: expected_row_count(base_document_count),
            base_fingerprints,
            changed_documents,
            deleted_paths,
            touched_base_documents_by_partition,
            added_query: unique_query_token(110_000 + added_index),
            added_path: repo_content_path(added_index),
            deleted_query: unique_query_token(deleted_index),
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
        let suffix = REPO_PUBLICATION_BENCH_COUNTER.fetch_add(1, Ordering::Relaxed);
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

impl RepoContentQueryBenchmarkFixture {
    /// Build one synthetic published repo-content corpus for repeated query
    /// samples.
    ///
    /// # Panics
    ///
    /// Panics when the fixture directories or publication cannot be created.
    #[must_use]
    pub fn synthetic(base_document_count: usize) -> Self {
        assert!(
            base_document_count >= 8,
            "repo-content query benchmark requires at least 8 documents"
        );
        let suffix = REPO_PUBLICATION_BENCH_COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("xiuxian-wendao-repo-query-bench-{suffix}"));
        let project_root = root.join("project");
        let storage_root = root.join("search_plane");
        let _ = std::fs::remove_dir_all(&root);
        create_dir_all(project_root.as_path());
        let manifest_keyspace =
            SearchManifestKeyspace::new(format!("xiuxian:bench:repo-query:{suffix}"));
        let repo_id = "alpha/repo".to_string();
        let service = SearchPlaneService::with_paths(
            project_root.clone(),
            storage_root.clone(),
            manifest_keyspace.clone(),
            SearchMaintenancePolicy::default(),
        );
        let base_documents = (0..base_document_count)
            .map(|index| repo_content_document(index, index))
            .collect::<Vec<_>>();
        build_runtime().block_on(async {
            publish_repo_content_chunks(&service, repo_id.as_str(), &base_documents, Some("rev-1"))
                .await
                .unwrap_or_else(|error| panic!("publish repo-content query fixture: {error}"));
        });
        let query_index = base_document_count / 2;
        Self {
            root,
            project_root,
            storage_root,
            manifest_keyspace,
            repo_id,
            base_document_count,
            expected_row_count: expected_row_count(base_document_count),
            query_token: unique_query_token(query_index),
            expected_path: repo_content_path(query_index),
            valkey_target_configured: resolve_search_plane_cache_connection_target().is_ok(),
        }
    }

    /// Prepare one query iteration with one fresh service instance.
    #[must_use]
    pub fn prepare_iteration(&self) -> RepoContentQueryBenchmarkIteration {
        RepoContentQueryBenchmarkIteration {
            service: SearchPlaneService::with_paths(
                self.project_root.clone(),
                self.storage_root.clone(),
                self.manifest_keyspace.clone(),
                SearchMaintenancePolicy::default(),
            ),
            runtime: build_runtime(),
            repo_id: self.repo_id.clone(),
            base_document_count: self.base_document_count,
            expected_row_count: self.expected_row_count,
            query_token: self.query_token.clone(),
            expected_path: self.expected_path.clone(),
            valkey_target_configured: self.valkey_target_configured,
        }
    }
}

impl Drop for RepoContentQueryBenchmarkFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
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
        let (row_count, added_query_paths, deleted_query_paths) = runtime.block_on(async {
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
                    &std::collections::HashSet::new(),
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
                    &std::collections::HashSet::new(),
                    5,
                )
                .await
                .unwrap_or_else(|error| panic!("query deleted benchmark token: {error}"))
                .into_iter()
                .map(|hit| hit.path)
                .collect::<Vec<_>>();
            (row_count, added_query_paths, deleted_query_paths)
        });
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
}

impl RepoContentQueryBenchmarkIteration {
    /// Run the full cold/hot/Flight query sequence and return one summary
    /// snapshot.
    ///
    /// # Panics
    ///
    /// Panics when any benchmark query or record read fails.
    #[must_use]
    pub fn run(mut self) -> RepoContentQueryBenchmarkSnapshot {
        let cold = self.measure_query();
        let hot = self.measure_query();
        let publication_row_count = self.publication_row_count();
        let flight_batch = self.measure_flight_batch();

        assert_eq!(
            cold.hit_count, 1,
            "cold repo-content query benchmark should return exactly one hit"
        );
        assert_eq!(
            hot.hit_count, 1,
            "hot repo-content query benchmark should return exactly one hit"
        );
        assert_eq!(
            cold.first_path.as_deref(),
            Some(self.expected_path.as_str()),
            "cold repo-content query benchmark drifted from expected path"
        );
        assert_eq!(
            hot.first_path.as_deref(),
            Some(self.expected_path.as_str()),
            "hot repo-content query benchmark drifted from expected path"
        );
        assert_eq!(
            flight_batch.row_count, 1,
            "repo-search Flight benchmark should emit exactly one row"
        );
        assert_eq!(
            publication_row_count, self.expected_row_count,
            "repo-content query benchmark row count drifted from the synthetic fixture"
        );

        RepoContentQueryBenchmarkSnapshot {
            base_document_count: self.base_document_count,
            publication_row_count,
            query_token: self.query_token.clone(),
            expected_path: self.expected_path.clone(),
            cold_query_elapsed: cold.elapsed,
            hot_query_elapsed: hot.elapsed,
            flight_batch_elapsed: flight_batch.elapsed,
            cold_query_hit_count: cold.hit_count,
            hot_query_hit_count: hot.hit_count,
            flight_batch_row_count: flight_batch.row_count,
            cold_first_path: cold.first_path,
            hot_first_path: hot.first_path,
            query_engine_kind: repo_query_engine_kind(),
            persisted_metadata_backend: persisted_metadata_backend(self.valkey_target_configured),
            valkey_target_configured: self.valkey_target_configured,
        }
    }

    /// Warm one fresh service with a cold query and then measure the steady
    /// state query path.
    ///
    /// # Panics
    ///
    /// Panics when the warmup or the measured query fails.
    #[must_use]
    pub fn measure_hot_query_after_cold_warmup(mut self) -> RepoContentQueryBenchmarkSample {
        let cold = self.measure_query();
        assert_eq!(
            cold.first_path.as_deref(),
            Some(self.expected_path.as_str()),
            "repo-content query benchmark warmup drifted from expected path"
        );
        self.measure_query()
    }

    /// Warm one fresh service with a cold query and then measure the Flight
    /// batch materialization path.
    ///
    /// # Panics
    ///
    /// Panics when the warmup query or the measured batch fails.
    #[must_use]
    pub fn measure_flight_batch_after_cold_warmup(
        mut self,
    ) -> RepoContentFlightBatchBenchmarkSample {
        let cold = self.measure_query();
        assert_eq!(
            cold.first_path.as_deref(),
            Some(self.expected_path.as_str()),
            "repo-content query benchmark warmup drifted from expected path"
        );
        self.measure_flight_batch()
    }

    fn measure_query(&mut self) -> RepoContentQueryBenchmarkSample {
        let started = Instant::now();
        let hits = self.runtime.block_on(async {
            self.service
                .search_repo_content_chunks(
                    self.repo_id.as_str(),
                    self.query_token.as_str(),
                    &HashSet::new(),
                    5,
                )
                .await
                .unwrap_or_else(|error| {
                    panic!("repo-content query benchmark query failed: {error}")
                })
        });
        RepoContentQueryBenchmarkSample {
            elapsed: started.elapsed(),
            hit_count: hits.len(),
            first_path: hits.first().map(|hit| hit.path.clone()),
        }
    }

    fn measure_flight_batch(&self) -> RepoContentFlightBatchBenchmarkSample {
        let request = RepoSearchFlightRequest {
            repo_id: self.repo_id.clone(),
            query_text: self.query_token.clone(),
            limit: 5,
            language_filters: HashSet::new(),
            path_prefixes: HashSet::new(),
            title_filters: HashSet::new(),
            tag_filters: HashSet::new(),
            filename_filters: HashSet::new(),
        };
        let started = Instant::now();
        let batch = self.runtime.block_on(async {
            search_repo_content_batch(&self.service, &request)
                .await
                .unwrap_or_else(|error| {
                    panic!("repo-content query benchmark Flight batch failed: {error}")
                })
        });
        RepoContentFlightBatchBenchmarkSample {
            elapsed: started.elapsed(),
            row_count: batch.num_rows(),
        }
    }

    fn publication_row_count(&self) -> u64 {
        self.runtime.block_on(async {
            self.service
                .repo_corpus_record_for_reads(
                    SearchCorpusKind::RepoContentChunk,
                    self.repo_id.as_str(),
                )
                .await
                .unwrap_or_else(|| {
                    panic!(
                        "repo-content query benchmark missing publication for `{}`",
                        self.repo_id
                    )
                })
                .publication
                .unwrap_or_else(|| {
                    panic!(
                        "repo-content query benchmark missing publication payload for `{}`",
                        self.repo_id
                    )
                })
                .row_count
        })
    }
}

impl Drop for RepoContentParquetMutationBenchmarkIteration {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn build_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap_or_else(|error| panic!("build repo-content parquet benchmark runtime: {error}"))
}

fn repo_content_document(index: usize, token_seed: usize) -> RepoCodeDocument {
    RepoCodeDocument {
        path: repo_content_path(index),
        language: Some("julia".to_string()),
        contents: Arc::<str>::from(repo_content_body(token_seed)),
        size_bytes: u64::try_from(BENCH_LINE_COUNT * 32).unwrap_or(u64::MAX),
        modified_unix_ms: u64::try_from(token_seed).unwrap_or(u64::MAX),
    }
}

fn repo_content_path(index: usize) -> String {
    format!("src/module_{index:05}.jl")
}

fn repo_content_body(token_seed: usize) -> String {
    (0..BENCH_LINE_COUNT)
        .map(|line| format!("value_{}_{} = {}", token_seed, line, token_seed + line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn unique_query_token(token_seed: usize) -> String {
    format!("value_{token_seed}_0")
}

fn expected_row_count(base_document_count: usize) -> u64 {
    let rows = base_document_count.saturating_mul(BENCH_LINE_COUNT);
    u64::try_from(rows).unwrap_or(u64::MAX)
}

fn persisted_metadata_backend(valkey_configured: bool) -> &'static str {
    if valkey_configured {
        "valkey_or_local_json"
    } else {
        "local_json_only"
    }
}

fn repo_query_engine_kind() -> &'static str {
    #[cfg(feature = "duckdb")]
    {
        "duckdb"
    }
    #[cfg(not(feature = "duckdb"))]
    {
        "datafusion"
    }
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
        let partition_count =
            repo_content_chunk_partition_count_for_document_count(base_fingerprints.len());
        let partition_id = fingerprint.partition_id.clone().unwrap_or_else(|| {
            repo_content_chunk_partition_id_for_path(
                path.as_str(),
                base_fingerprints,
                partition_count,
            )
        });
        if let Some(document_count) = distribution.get_mut(partition_id.as_str()) {
            *document_count += 1;
        }
    }
    distribution
}

fn create_dir_all(path: &Path) {
    std::fs::create_dir_all(path)
        .unwrap_or_else(|error| panic!("create directory {}: {error}", path.display()));
}

fn copy_dir_recursive(source: &Path, target: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(target)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir_recursive(source_path.as_path(), target_path.as_path())?;
        } else if file_type.is_file() {
            std::fs::copy(source_path.as_path(), target_path.as_path())?;
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                format!(
                    "unsupported repo-content parquet benchmark entry {}",
                    source_path.display()
                ),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/unit/search/perf_support.rs"]
mod tests;
