//! Deterministic reasoning-packet compiler for structural facts rows.

mod api;
mod builder;
mod input;
mod types;
mod write;

pub use api::write_episteme_ontology_structural_facts_reasoning_packet;
pub use types::{
    EpistemeOntologyStructuralFactsReasoningPacketReport,
    EpistemeOntologyStructuralFactsReasoningPacketRequest,
};
