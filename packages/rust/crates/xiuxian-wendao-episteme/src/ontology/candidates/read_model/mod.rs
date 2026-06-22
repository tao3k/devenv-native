//! Arrow/Parquet read-model support for ontology candidate rows.

mod publish;
mod summary;

pub(super) use publish::write_candidate_read_model;
pub use summary::summarize_episteme_ontology_candidate_read_model;
