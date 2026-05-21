//! Org ledger validation for episteme source-contract mappings.

use serde::Serialize;

use super::EpistemeSourceContractParseError;
use crate::org::{
    compile_org_ontology_authoring_document, compile_org_reasoning_property_records,
    validate_org_reasoning_properties,
};

/// Validation summary for the episteme source-contract Org mapping ledger.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct EpistemeMappingLedgerValidation {
    /// Count of typed Org ontology authoring sections.
    pub section_count: usize,
    /// Count of schema-governed reasoning property records.
    pub reasoning_property_record_count: usize,
}

/// Validate the episteme source-contract Org mapping ledger through the parser-owned Org
/// authoring compiler and reasoning property schema gate.
///
/// # Errors
///
/// Returns an error when the Org authoring contract is invalid, the ledger has
/// no `corpus_mapping` section, or a schema-governed property drawer fails the
/// reasoning property contract.
pub fn validate_episteme_mapping_ledger_org(
    raw: &str,
    source_path: impl Into<String>,
) -> Result<EpistemeMappingLedgerValidation, EpistemeSourceContractParseError> {
    let document = compile_org_ontology_authoring_document(raw, source_path)
        .map_err(|source| EpistemeSourceContractParseError::OrgAuthoring { source })?;
    if !document
        .sections
        .iter()
        .any(|section| section.authoring_kind == "corpus_mapping")
    {
        return Err(EpistemeSourceContractParseError::MissingCorpusMappingSection);
    }

    let diagnostics = validate_org_reasoning_properties(&document);
    if !diagnostics.is_empty() {
        return Err(EpistemeSourceContractParseError::OrgReasoningProperties { diagnostics });
    }

    Ok(EpistemeMappingLedgerValidation {
        section_count: document.sections.len(),
        reasoning_property_record_count: compile_org_reasoning_property_records(&document).len(),
    })
}
