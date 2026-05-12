//! Repo-content query benchmark fixtures and measured query samples.

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Instant;

use crate::search::repo_search::search_repo_content_batch;
use crate::search::{
    SearchCorpusKind, SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlaneService,
    resolve_search_plane_cache_connection_target,
};
use xiuxian_wendao_runtime::transport::RepoSearchFlightRequest;

use super::fixture::{
    assert_minimum_benchmark_documents, benchmark_suffix, build_runtime, expected_row_count,
    persisted_metadata_backend, publish_base_repo_content_fixture, repo_content_benchmark_paths,
    repo_content_benchmark_service, repo_content_path, repo_query_engine_kind, unique_query_token,
};
use super::samples::{
    RepoContentFlightBatchBenchmarkSample, RepoContentQueryBenchmarkSample,
    RepoContentQueryBenchmarkSnapshot,
};

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

impl RepoContentQueryBenchmarkFixture {
    /// Build one synthetic published repo-content corpus for repeated query
    /// samples.
    ///
    /// # Panics
    ///
    /// Panics when the fixture directories or publication cannot be created.
    #[must_use]
    pub fn synthetic(base_document_count: usize) -> Self {
        assert_minimum_benchmark_documents(base_document_count, "repo-content query benchmark");
        let suffix = benchmark_suffix();
        let paths = repo_content_benchmark_paths(
            format!("xiuxian-wendao-repo-query-bench-{suffix}").as_str(),
            "search_plane",
        );
        let manifest_keyspace =
            SearchManifestKeyspace::new(format!("xiuxian:bench:repo-query:{suffix}"));
        let repo_id = "alpha/repo".to_string();
        let service = repo_content_benchmark_service(&paths, &manifest_keyspace);
        publish_base_repo_content_fixture(&service, repo_id.as_str(), base_document_count);
        let query_index = base_document_count / 2;
        Self {
            root: paths.root,
            project_root: paths.project_root,
            storage_root: paths.storage_root,
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

        self.assert_standard_query_snapshot(&cold, &hot, &flight_batch, publication_row_count);

        RepoContentQueryBenchmarkSnapshot {
            base_document_count: self.base_document_count,
            publication_row_count,
            query_token: self.query_token.clone(),
            expected_path: self.expected_path.clone(),
            cold_query_elapsed: cold.elapsed,
            hot_query_elapsed: hot.elapsed,
            flight_batch_elapsed: flight_batch.elapsed,
            cold_query_hit_count: cold.hit_count,
            cold_query_rows_scanned: cold.rows_scanned,
            hot_query_hit_count: hot.hit_count,
            hot_query_rows_scanned: hot.rows_scanned,
            flight_batch_row_count: flight_batch.row_count,
            flight_batch_rows_scanned: flight_batch.rows_scanned,
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
        let query_token = self.query_token.clone();
        let cold = self.measure_query_for_token(query_token.as_str());
        assert_eq!(
            cold.first_path.as_deref(),
            Some(self.expected_path.as_str()),
            "repo-content query benchmark warmup drifted from expected path"
        );
        self.measure_query_for_token(query_token.as_str())
    }

    /// Measure one warmed repo-content query for one explicit token after one
    /// cold warmup on the same fresh service instance.
    ///
    /// # Panics
    ///
    /// Panics when the warmup or measured query fails.
    #[must_use]
    pub fn measure_hot_query_for_token_after_cold_warmup(
        mut self,
        query_token: &str,
    ) -> RepoContentQueryBenchmarkSample {
        let cold = self.measure_query_for_token(query_token);
        assert!(
            cold.hit_count > 0,
            "repo-content query benchmark warmup should return at least one hit"
        );
        self.measure_query_for_token(query_token)
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
        let query_token = self.query_token.clone();
        let cold = self.measure_query_for_token(query_token.as_str());
        assert_eq!(
            cold.first_path.as_deref(),
            Some(self.expected_path.as_str()),
            "repo-content query benchmark warmup drifted from expected path"
        );
        self.measure_flight_batch_for_token(query_token.as_str())
    }

    /// Measure one warmed repo-search Flight batch for one explicit token
    /// after one cold warmup on the same fresh service instance.
    ///
    /// # Panics
    ///
    /// Panics when the warmup query or the measured batch fails.
    #[must_use]
    pub fn measure_flight_batch_for_token_after_cold_warmup(
        mut self,
        query_token: &str,
    ) -> RepoContentFlightBatchBenchmarkSample {
        let cold = self.measure_query_for_token(query_token);
        assert!(
            cold.hit_count > 0,
            "repo-content query benchmark warmup should return at least one hit"
        );
        self.measure_flight_batch_for_token(query_token)
    }

    fn assert_standard_query_snapshot(
        &self,
        cold: &RepoContentQueryBenchmarkSample,
        hot: &RepoContentQueryBenchmarkSample,
        flight_batch: &RepoContentFlightBatchBenchmarkSample,
        publication_row_count: u64,
    ) {
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
    }

    fn measure_query(&mut self) -> RepoContentQueryBenchmarkSample {
        let query_token = self.query_token.clone();
        self.measure_query_for_token(query_token.as_str())
    }

    fn measure_query_for_token(&mut self, query_token: &str) -> RepoContentQueryBenchmarkSample {
        let started = Instant::now();
        let hits = self.runtime.block_on(async {
            self.service
                .search_repo_content_chunks(self.repo_id.as_str(), query_token, &HashSet::new(), 5)
                .await
                .unwrap_or_else(|error| {
                    panic!("repo-content query benchmark query failed: {error}")
                })
        });
        let telemetry = self
            .service
            .query_telemetry_for(SearchCorpusKind::RepoContentChunk)
            .unwrap_or_else(|| {
                panic!("repo-content query benchmark missing query telemetry after search")
            });
        RepoContentQueryBenchmarkSample {
            elapsed: started.elapsed(),
            hit_count: hits.len(),
            rows_scanned: telemetry.rows_scanned,
            matched_rows: telemetry.matched_rows,
            first_path: hits.first().map(|hit| hit.path.clone()),
        }
    }

    fn measure_flight_batch(&self) -> RepoContentFlightBatchBenchmarkSample {
        let query_token = self.query_token.clone();
        self.measure_flight_batch_for_token(query_token.as_str())
    }

    fn measure_flight_batch_for_token(
        &self,
        query_token: &str,
    ) -> RepoContentFlightBatchBenchmarkSample {
        let request = RepoSearchFlightRequest {
            repo_id: self.repo_id.clone(),
            query_text: query_token.to_string(),
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
        let telemetry = self
            .service
            .query_telemetry_for(SearchCorpusKind::RepoContentChunk)
            .unwrap_or_else(|| {
                panic!("repo-content query benchmark missing query telemetry after Flight batch")
            });
        RepoContentFlightBatchBenchmarkSample {
            elapsed: started.elapsed(),
            row_count: batch.num_rows(),
            rows_scanned: telemetry.rows_scanned,
            matched_rows: telemetry.matched_rows,
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
