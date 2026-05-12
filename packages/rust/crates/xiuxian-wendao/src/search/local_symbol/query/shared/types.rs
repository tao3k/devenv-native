//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
//! `search::local_symbol::query::shared::types` owns Wendao query shared types behavior.

use crate::duckdb::ParquetQueryEngine;
use crate::search::contracts::AutocompleteSuggestion;
use crate::search::ranking::{StreamingRerankSource, StreamingRerankTelemetry};
use xiuxian_db_store::VectorStoreError;

#[derive(Debug, thiserror::Error)]
/// Errors returned while querying the local-symbol search index.
pub enum LocalSymbolSearchError {
    /// The local-symbol index has not published a readable epoch yet.
    #[error("local symbol index has no published epoch")]
    NotReady,
    /// The vector-store or query-engine layer failed.
    #[error(transparent)]
    Storage(#[from] VectorStoreError),
    /// Stored local-symbol rows could not be decoded into search hits.
    #[error("{0}")]
    Decode(String),
}

#[derive(Debug)]
pub(crate) struct LocalSymbolSearchExecution {
    pub(crate) candidates: Vec<LocalSymbolCandidate>,
    pub(crate) telemetry: StreamingRerankTelemetry,
    pub(crate) source: StreamingRerankSource,
}

#[derive(Debug)]
pub(crate) struct LocalSymbolAutocompleteExecution {
    pub(crate) suggestions: Vec<AutocompleteSuggestion>,
    pub(crate) telemetry: StreamingRerankTelemetry,
    pub(crate) source: StreamingRerankSource,
}

#[derive(Clone)]
pub(crate) struct PreparedLocalSymbolRead {
    pub(crate) query_engine: ParquetQueryEngine,
    pub(crate) table_names: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct LocalSymbolCandidate {
    pub(crate) table_name: String,
    pub(crate) id: String,
    pub(crate) score: f64,
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) line_start: usize,
}
