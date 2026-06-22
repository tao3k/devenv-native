//! `ontology/registry.json` admission and read-model input.

mod model;
mod validation;

pub use model::{
    EpistemeOntologyRegistryActionType, EpistemeOntologyRegistryApiSurface,
    EpistemeOntologyRegistryArtifactMode, EpistemeOntologyRegistryCategory,
    EpistemeOntologyRegistryDatasetMapping, EpistemeOntologyRegistryDomain,
    EpistemeOntologyRegistryInterfaceType, EpistemeOntologyRegistryKind,
    EpistemeOntologyRegistryLinkType, EpistemeOntologyRegistryObjectPropertyTerm,
    EpistemeOntologyRegistryObjectType, EpistemeOntologyRegistryObjectTypeRef,
    EpistemeOntologyRegistryPolicy, EpistemeOntologyRegistryQueryType,
    EpistemeOntologyRegistryRdfClassTerm, EpistemeOntologyRegistryRdfTerms,
    EpistemeOntologyRegistryReadModelInput, EpistemeOntologyRegistryRule,
    EpistemeOntologyRegistrySnapshot, EpistemeOntologyRegistrySnapshotReport,
    EpistemeOntologyRegistrySourceContract, ONTOLOGY_REGISTRY_RELATIVE_PATH,
};
pub use validation::{
    EpistemeOntologyRegistryError, admit_ontology_registry_snapshot, ontology_registry_path,
    read_ontology_registry_snapshot, validate_ontology_registry_snapshot,
};
