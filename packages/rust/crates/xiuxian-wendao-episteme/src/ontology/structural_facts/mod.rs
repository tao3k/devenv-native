//! Deterministic structural facts seed compiler for private Episteme source contracts.

mod api;
mod builder;
mod ids;
mod rdf_seed;
mod read_model;
mod types;
mod validation;
mod write;

pub use api::{
    write_episteme_ontology_structural_facts, write_episteme_ontology_structural_facts_from_config,
};
pub use types::{
    EpistemeOntologyStructuralFactsConfiguredRequest, EpistemeOntologyStructuralFactsReport,
    EpistemeOntologyStructuralFactsRequest, EpistemeOntologyStructuralFactsValidationMode,
};
