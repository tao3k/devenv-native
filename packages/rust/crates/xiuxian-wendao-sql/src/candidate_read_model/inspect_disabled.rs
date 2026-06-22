//! Feature-disabled candidate read-model inspection entrypoint.

use super::types::{
    CandidateReadModelDuckDbInspectionReport, CandidateReadModelDuckDbInspectionRequest,
};

/// Feature-disabled candidate Parquet read-model inspection placeholder.
///
/// # Errors
///
/// Always returns an error when the `duckdb` feature is not enabled.
pub fn inspect_candidate_read_model_with_duckdb_disabled(
    _request: &CandidateReadModelDuckDbInspectionRequest,
) -> Result<CandidateReadModelDuckDbInspectionReport, String> {
    Err(
        "candidate read-model DuckDB inspection requires the `duckdb` feature on xiuxian-wendao-sql"
            .to_string(),
    )
}
