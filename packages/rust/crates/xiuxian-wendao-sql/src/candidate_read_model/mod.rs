//! DuckDB inspection over Episteme ontology candidate Parquet read models.

mod constants;
#[cfg(feature = "duckdb")]
mod inspect;
#[cfg(not(feature = "duckdb"))]
mod inspect_disabled;
mod types;

#[cfg(feature = "duckdb")]
pub use inspect::inspect_candidate_read_model_with_duckdb;
#[cfg(not(feature = "duckdb"))]
pub use inspect_disabled::inspect_candidate_read_model_with_duckdb_disabled as inspect_candidate_read_model_with_duckdb;
pub use types::{
    CandidateReadModelDuckDbInspectionReport, CandidateReadModelDuckDbInspectionRequest,
    CandidateReadModelKind, CandidateReadModelKindCount, CandidateReadModelMissingEndpoint,
};

#[cfg(all(test, feature = "duckdb"))]
#[path = "../../tests/unit/candidate_read_model/mod.rs"]
mod tests;
