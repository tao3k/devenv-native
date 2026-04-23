use tokio::sync::OwnedSemaphorePermit;

use crate::duckdb::ParquetQueryEngine;
use crate::search::ranking::{RetainedWindow, StreamingRerankSource, StreamingRerankTelemetry};

pub(crate) const MODULE_BUCKETS: u8 = 3;
pub(crate) const SYMBOL_BUCKETS: u8 = 7;
pub(crate) const EXAMPLE_BUCKETS: u8 = 10;
pub(crate) const IMPORT_BUCKETS: u8 = 4;
pub(crate) const MIN_RECALL_CANDIDATES: usize = 256;
pub(crate) const RECALL_TRIM_MULTIPLIER: usize = 8;

#[derive(Debug, thiserror::Error)]
pub(crate) enum RepoEntitySearchError {
    #[error(transparent)]
    Storage(#[from] xiuxian_db_store::VectorStoreError),
    #[error("{0}")]
    Decode(String),
}

#[derive(Debug, Clone)]
pub(crate) struct RepoEntityCandidate {
    pub(crate) id: String,
    pub(crate) score: f64,
    pub(crate) entity_kind: String,
    pub(crate) name: String,
    pub(crate) path: String,
}

pub(crate) struct RepoEntityQuery<'a> {
    pub(crate) query_lower: &'a str,
    pub(crate) import_package_filter: Option<&'a str>,
    pub(crate) import_module_filter: Option<&'a str>,
    pub(crate) language_filters: &'a std::collections::HashSet<String>,
    pub(crate) kind_filters: &'a std::collections::HashSet<String>,
    pub(crate) window: RetainedWindow,
}

pub(crate) struct RepoEntitySearchExecution {
    pub(crate) candidates: Vec<RepoEntityCandidate>,
    pub(crate) telemetry: StreamingRerankTelemetry,
    pub(crate) source: StreamingRerankSource,
}

pub(crate) struct PreparedRepoEntitySearch {
    pub(crate) _read_permit: OwnedSemaphorePermit,
    pub(crate) query_engine: ParquetQueryEngine,
    pub(crate) engine_table_name: String,
    pub(crate) candidates: Vec<RepoEntityCandidate>,
    pub(crate) telemetry: StreamingRerankTelemetry,
    pub(crate) source: StreamingRerankSource,
}

pub(crate) struct PreparedRepoEntityPublication {
    pub(crate) _read_permit: OwnedSemaphorePermit,
    pub(crate) query_engine: ParquetQueryEngine,
    pub(crate) engine_table_name: String,
    pub(crate) source_revision: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct HydratedRepoEntityRow {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) qualified_name: String,
    pub(crate) path: String,
    pub(crate) symbol_kind: String,
    pub(crate) module_id: Option<String>,
    pub(crate) signature: Option<String>,
    pub(crate) summary: Option<String>,
    pub(crate) line_start: Option<u32>,
    pub(crate) line_end: Option<u32>,
    pub(crate) audit_status: Option<String>,
    pub(crate) verification_state: Option<String>,
    pub(crate) attributes_json: Option<String>,
    pub(crate) hierarchical_uri: Option<String>,
    pub(crate) hierarchy: Vec<String>,
    pub(crate) implicit_backlinks: Vec<String>,
    pub(crate) implicit_backlink_items_json: Option<String>,
    pub(crate) projection_page_ids: Vec<String>,
    pub(crate) saliency_score: f64,
}
