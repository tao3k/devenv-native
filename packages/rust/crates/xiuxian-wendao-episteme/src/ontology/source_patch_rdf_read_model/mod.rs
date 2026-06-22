//! Semantic read-model generation from applied source-patch RDF.

mod engine;
mod parse;
mod projection;
mod types;
mod write;

pub use engine::write_episteme_ontology_source_patch_rdf_read_model;
pub use types::{
    EpistemeOntologySourcePatchRdfReadModelReport, EpistemeOntologySourcePatchRdfReadModelRequest,
};
