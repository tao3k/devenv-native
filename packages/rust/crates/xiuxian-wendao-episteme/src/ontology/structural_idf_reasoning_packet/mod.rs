//! Deterministic reasoning-packet compiler for structural IDF rows.

mod api;
mod builder;
mod input;
mod types;
mod write;

pub use api::write_episteme_ontology_structural_idf_reasoning_packet;
pub use types::{
    EpistemeOntologyStructuralIdfReasoningPacketReport,
    EpistemeOntologyStructuralIdfReasoningPacketRequest,
};
