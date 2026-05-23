//! Deterministic structural IDF seed compiler for private Episteme source contracts.

mod api;
mod builder;
mod ids;
mod types;
mod validation;
mod write;

pub use api::write_episteme_ontology_structural_idf;
pub use types::{
    EpistemeOntologyStructuralIdfReport, EpistemeOntologyStructuralIdfRequest,
    EpistemeOntologyStructuralIdfValidationMode,
};
