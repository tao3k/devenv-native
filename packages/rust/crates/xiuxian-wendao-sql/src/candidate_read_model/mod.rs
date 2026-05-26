//! DuckDB inspection over Episteme ontology candidate Parquet read models.

mod constants;
mod inspect;
mod types;

pub use inspect::inspect_candidate_read_model_with_duckdb;
pub use types::{
    CandidateReadModelDuckDbInspectionReport, CandidateReadModelDuckDbInspectionRequest,
    CandidateReadModelKind, CandidateReadModelKindCount, CandidateReadModelMissingEndpoint,
};

#[cfg(test)]
#[path = "../../tests/unit/candidate_read_model/mod.rs"]
mod tests;
