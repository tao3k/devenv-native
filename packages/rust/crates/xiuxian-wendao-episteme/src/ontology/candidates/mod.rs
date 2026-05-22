//! Deterministic private ontology candidate generation.

mod api;
mod generation;
mod identifiers;
mod inputs;
mod io;
mod mapping;
mod model;
mod rows;
mod writing;

pub use api::{
    EpistemeOntologyCandidateGenerationReport, EpistemeOntologyCandidateGenerationRequest,
};
pub use generation::generate_episteme_ontology_candidates;
