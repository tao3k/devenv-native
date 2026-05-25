//! Structural-facts artifact conversion for `WendaoGraph` ontology quality checks.

mod adapter;
mod batch;
mod convert;
mod read;
mod types;

pub use adapter::build_wendaograph_ontology_read_model_quality_request_batches_from_structural_facts_artifacts;
