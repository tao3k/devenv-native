//! Parquet-backed semantic preview readers for `WendaoGraph` quality checks.

mod adapter;
mod batch;
mod read;
mod types;

pub use adapter::{
    build_wendaograph_ontology_read_model_quality_request_batches_from_rdf_source_artifacts,
    build_wendaograph_ontology_read_model_quality_request_batches_from_semantic_preview_artifacts,
};
