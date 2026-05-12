//! Public benchmark sample records emitted by search performance fixtures.

use std::collections::BTreeMap;
use std::time::Duration;

use crate::search::repo_content_chunk::RepoContentChunkIncrementalPublishProfile;

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
/// Raw DTO boundary: this public record mirrors serialized Wendao transport fields.
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
    /// Number of rows scanned by the first query.
    pub cold_query_rows_scanned: u64,
    /// Number of hits returned by the second query.
    pub hot_query_hit_count: usize,
    /// Number of rows scanned by the second query.
    pub hot_query_rows_scanned: u64,
    /// Number of rows emitted by the Flight batch surface.
    pub flight_batch_row_count: usize,
    /// Number of rows scanned by the Flight-facing search call.
    pub flight_batch_rows_scanned: u64,
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

/// One measured repo-content query sample.
#[derive(Debug, Clone)]
pub struct RepoContentQueryBenchmarkSample {
    /// Time spent executing the sample.
    pub elapsed: Duration,
    /// Number of hits returned by the sample.
    pub hit_count: usize,
    /// Number of rows scanned by the sample.
    pub rows_scanned: u64,
    /// Number of matched rows observed by the sample.
    pub matched_rows: u64,
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
    /// Number of rows scanned by the underlying search call.
    pub rows_scanned: u64,
    /// Number of matched rows observed by the underlying search call.
    pub matched_rows: u64,
}
