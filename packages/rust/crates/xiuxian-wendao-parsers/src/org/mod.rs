//! Parser-owned Org-mode document, note, and section contracts.

mod document;
mod note;
mod ontology;
mod property_schema;
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
pub use property_schema::{
    ORG_PROP_BLANK_VALUE, ORG_PROP_INVALID_CONFIDENCE, ORG_PROP_INVALID_ENUM,
    ORG_PROP_INVALID_SHA256, ORG_PROP_INVALID_UUID, ORG_PROP_MISSING_REQUIRED,
    ORG_PROP_UNKNOWN_PROPERTY, ORG_REASONING_PROPERTY_SCHEMA_ID, OrgReasoningPropertyDiagnostic,
    OrgReasoningPropertyRecord, compile_org_reasoning_property_records,
    validate_org_reasoning_properties, validate_org_reasoning_property_records,
};
pub use sections::extract_org_sections;
pub use types::{OrgNote, OrgNoteCore, OrgSection, OrgTocDocument, parse_org_toc};
