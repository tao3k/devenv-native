//! Error boundary for episteme source-contract parsing.

use thiserror::Error;

use crate::org::{OrgOntologyAuthoringError, OrgReasoningPropertyDiagnostic};

/// Parser-owned error for episteme source-contract inputs.
#[derive(Debug, Error)]
pub enum EpistemeSourceContractParseError {
    /// The source manifest TOML is malformed.
    #[error("failed to parse episteme source-contract source manifest TOML: {source}")]
    Toml {
        /// Underlying TOML error.
        #[source]
        source: toml::de::Error,
    },
    /// A TSV table has the wrong header.
    #[error("episteme source-contract TSV header mismatch: expected {expected:?}, got {actual:?}")]
    TsvHeader {
        /// Expected TSV fields.
        expected: Vec<String>,
        /// Actual TSV fields.
        actual: Vec<String>,
    },
    /// A TSV row has the wrong number of fields.
    #[error("episteme source-contract TSV row {row} has {actual} fields, expected {expected}")]
    TsvRowWidth {
        /// One-based TSV row number.
        row: usize,
        /// Expected field count.
        expected: usize,
        /// Actual field count.
        actual: usize,
    },
    /// A numeric field could not be parsed.
    #[error("invalid episteme source-contract numeric value `{value}` in `{field}` at row {row}")]
    InvalidNumber {
        /// One-based TSV row number.
        row: usize,
        /// Field name.
        field: &'static str,
        /// Invalid value.
        value: String,
    },
    /// The episteme source-contract mapping ledger has invalid Org ontology authoring shape.
    #[error("invalid episteme source-contract mapping ledger Org authoring contract: {source}")]
    OrgAuthoring {
        /// Underlying Org ontology compiler error.
        #[source]
        source: OrgOntologyAuthoringError,
    },
    /// The episteme source-contract mapping ledger has no corpus mapping section.
    #[error(
        "episteme source-contract mapping ledger is missing a corpus_mapping authoring section"
    )]
    MissingCorpusMappingSection,
    /// The episteme source-contract mapping ledger has invalid schema-governed Org properties.
    #[error(
        "episteme source-contract mapping ledger has invalid Org reasoning properties: {diagnostics:?}"
    )]
    OrgReasoningProperties {
        /// Parser-owned property schema diagnostics.
        diagnostics: Vec<OrgReasoningPropertyDiagnostic>,
    },
}
