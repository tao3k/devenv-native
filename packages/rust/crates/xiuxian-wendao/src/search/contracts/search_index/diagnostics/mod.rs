//! SQL-backed aggregate diagnostics for Studio search-index status payloads.

mod decode;
mod labels;
mod relations;
mod sql;
mod summary;

pub use relations::{
    QUERY_TELEMETRY_DIAGNOSTICS_TABLE, REPO_READ_PRESSURE_DIAGNOSTICS_TABLE,
    STATUS_DIAGNOSTICS_TABLE, STATUS_REASON_DIAGNOSTICS_TABLE, diagnostics_schema_ref,
    query_telemetry_contract, repo_read_pressure_contract, status_reason_contract,
    status_snapshot_contract,
};
#[cfg(all(test, feature = "duckdb"))]
pub(crate) use summary::configured_status_diagnostics_engine_kind;
pub(crate) use summary::{SearchIndexDiagnosticsRollup, summarize_status_diagnostics};
