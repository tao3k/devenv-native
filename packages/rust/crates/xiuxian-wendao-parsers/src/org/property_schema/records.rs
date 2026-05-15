//! Record projection for Org reasoning property schema validation.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::super::ontology::{OrgOntologyAuthoringDocument, OrgOntologySourceSpan};

/// Draft schema id for Org reasoning property drawer records.
pub const ORG_REASONING_PROPERTY_SCHEMA_ID: &str = "xiuxian_wendao.org_reasoning_property.v0.draft";

/// A compiled Org property drawer record selected for Wendao reasoning
/// property validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgReasoningPropertyRecord {
    /// Draft schema identifier for the compiled property record.
    pub schema: String,
    /// Source document id.
    pub document_id: String,
    /// Repository-relative source identity supplied to the compiler.
    pub source_path: String,
    /// Source document hash.
    pub source_hash: String,
    /// Source section id.
    pub section_id: String,
    /// Heading path for the source section.
    pub heading_path: Vec<String>,
    /// Property drawer values normalized to upper-case keys.
    pub properties: BTreeMap<String, String>,
    /// Reopenable source span for the source section.
    pub source_span: OrgOntologySourceSpan,
}

/// Project schema-governed Org reasoning property records from a compiled
/// authoring document.
#[must_use]
pub fn compile_org_reasoning_property_records(
    document: &OrgOntologyAuthoringDocument,
) -> Vec<OrgReasoningPropertyRecord> {
    document
        .sections
        .iter()
        .filter(|section| section.properties.contains_key("WENDAO_KIND"))
        .map(|section| OrgReasoningPropertyRecord {
            schema: ORG_REASONING_PROPERTY_SCHEMA_ID.to_string(),
            document_id: document.document_id.clone(),
            source_path: document.source_path.clone(),
            source_hash: document.source_hash.clone(),
            section_id: section.section_id.clone(),
            heading_path: section.heading_path.clone(),
            properties: section.properties.clone(),
            source_span: section.source_span.clone(),
        })
        .collect()
}
