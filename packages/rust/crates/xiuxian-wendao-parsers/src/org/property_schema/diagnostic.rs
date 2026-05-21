//! Diagnostic DTOs for Org property schema validation.

use serde::{Deserialize, Serialize};

use crate::org::ontology::OrgOntologySourceSpan;

/// Raw DTO boundary: parser-owned diagnostic for schema-governed Org reasoning properties.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgReasoningPropertyDiagnostic {
    /// Stable diagnostic code.
    pub code: String,
    /// Human-readable validation message.
    pub message: String,
    /// Source document id.
    pub document_id: String,
    /// Source section id.
    pub section_id: String,
    /// Heading path for the offending section.
    pub heading_path: Vec<String>,
    /// Repository-relative source identity supplied to the compiler.
    pub source_path: String,
    /// Offending property key when the diagnostic is property-specific.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property: Option<String>,
    /// Reopenable source span for the offending section.
    pub source_span: OrgOntologySourceSpan,
}
