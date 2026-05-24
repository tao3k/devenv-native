//! Deterministic private ontology candidate generation.

mod api;
mod generation;
mod identifiers;
mod inputs;
mod io;
mod mapping;
mod model;
mod read_model;
mod rows;
mod writing;

pub use api::{
    EpistemeOntologyCandidateGenerationReport, EpistemeOntologyCandidateGenerationRequest,
    EpistemeOntologyCandidateReadModelMissingEndpoint,
    EpistemeOntologyCandidateReadModelSummaryReport,
    EpistemeOntologyCandidateReadModelSummaryRequest,
};
pub use generation::generate_episteme_ontology_candidates;
pub use read_model::summarize_episteme_ontology_candidate_read_model;
