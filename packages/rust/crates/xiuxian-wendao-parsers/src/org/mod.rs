//! Parser-owned Org-mode document, note, and section contracts.

mod document;
mod note;
mod ontology;
mod sections;
mod types;

pub use document::parse_org_document;
pub use note::parse_org_note;
pub use ontology::{
    ORG_ONTOLOGY_AUTHORING_SCHEMA_ID, OrgOntologyAuthoringDocument, OrgOntologyAuthoringError,
    OrgOntologyAuthoringKind, OrgOntologyAuthoringSection, OrgOntologyAuthoringTable,
    OrgOntologyEmbeddedArtifact, OrgOntologyLifecycleState, OrgOntologySourceSpan,
    OrgOntologyTableKind, compile_org_ontology_authoring_document,
};
pub use sections::extract_org_sections;
pub use types::{OrgNote, OrgNoteCore, OrgSection, OrgTocDocument, parse_org_toc};
