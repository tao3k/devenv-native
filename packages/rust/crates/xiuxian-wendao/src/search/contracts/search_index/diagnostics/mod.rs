//! SQL-backed aggregate diagnostics for Studio search-index status payloads.

mod decode;
mod labels;
mod relations;
mod sql;
mod summary;

#[cfg(all(test, feature = "duckdb"))]
pub(crate) use summary::configured_status_diagnostics_engine_kind;
pub(crate) use summary::{SearchIndexDiagnosticsRollup, summarize_status_diagnostics};
