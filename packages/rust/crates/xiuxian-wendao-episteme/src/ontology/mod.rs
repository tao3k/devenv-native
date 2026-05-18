//! Ontology source-contract admission.

mod manifest;

pub use manifest::{
    EpistemeOntologyApiSurface, EpistemeOntologyArtifactMode, EpistemeOntologyBoundaries,
    EpistemeOntologyContractReport, EpistemeOntologyDomain, EpistemeOntologyDomainCategory,
    EpistemeOntologyError, EpistemeOntologyExtensionContract, EpistemeOntologyManifest,
    ONTOLOGY_MANIFEST_RELATIVE_PATH, ontology_manifest_path, read_ontology_manifest,
    validate_ontology_contract,
};
