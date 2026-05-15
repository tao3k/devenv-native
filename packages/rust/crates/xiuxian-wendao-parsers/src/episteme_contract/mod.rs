use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::org::{
    OrgOntologyAuthoringError, OrgReasoningPropertyDiagnostic,
    compile_org_ontology_authoring_document, compile_org_reasoning_property_records,
    validate_org_reasoning_properties,
};

const FILE_FIELDS: [&str; 8] = [
    "file_id",
    "relative_path",
    "extension",
    "byte_size",
    "sha256",
    "category",
    "language",
    "extraction_route",
];

const EXTRACTION_QUEUE_FIELDS: [&str; 9] = [
    "queue_id",
    "file_id",
    "relative_path",
    "category",
    "language",
    "extraction_route",
    "priority",
    "output_contract",
    "status",
];

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

/// Parser-owned episteme source-contract source manifest DTO.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct EpistemeSourceManifest {
    /// Manifest schema version.
    pub schema_version: u32,
    /// Stable source contract id.
    pub source_contract_id: String,
    /// Domain URI.
    pub domain: String,
    /// Primary source corpus language.
    pub primary_language: String,
    /// Environment variable that points at the source corpus root.
    pub corpus_root_env: String,
    /// Files table path relative to the corpus contract directory.
    pub files: String,
    /// Extraction queue path relative to the corpus contract directory.
    pub extraction_queue: String,
    /// Whether raw files may be copied into the episteme repository.
    pub copy_raw_files: bool,
    /// Whether raw rows may be promoted directly to RDF truth.
    pub raw_to_rdf_promotion_allowed: bool,
    /// Path components ignored during corpus discovery.
    #[serde(default)]
    pub ignored_names: Vec<String>,
    /// Extraction routes keyed by route id, with allowed extensions as values.
    pub routes: BTreeMap<String, Vec<String>>,
}

/// One row from the episteme source-contract `files.tsv` source contract.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct EpistemeFileRow {
    /// Stable source file id.
    pub file_id: String,
    /// Source path relative to the source corpus root.
    pub relative_path: String,
    /// Lowercase extension without dot.
    pub extension: String,
    /// Recorded source size in bytes.
    pub byte_size: u64,
    /// Recorded source SHA-256.
    pub sha256: String,
    /// Corpus category.
    pub category: String,
    /// Source language tag.
    pub language: String,
    /// Planned extraction route.
    pub extraction_route: String,
}

/// One row from the episteme source-contract `extraction_queue.tsv` source contract.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct EpistemeExtractionQueueRow {
    /// Stable queue row id.
    pub queue_id: String,
    /// Stable source file id.
    pub file_id: String,
    /// Source path relative to the source corpus root.
    pub relative_path: String,
    /// Corpus category.
    pub category: String,
    /// Source language tag.
    pub language: String,
    /// Planned extraction route.
    pub extraction_route: String,
    /// Queue priority; lower values are planned first.
    pub priority: u32,
    /// Output contract for this row.
    pub output_contract: String,
    /// Queue row status.
    pub status: String,
}

/// Validation summary for the episteme source-contract Org mapping ledger.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct EpistemeMappingLedgerValidation {
    /// Count of typed Org ontology authoring sections.
    pub section_count: usize,
    /// Count of schema-governed reasoning property records.
    pub reasoning_property_record_count: usize,
}

/// Parse a episteme source-contract source manifest from TOML text.
///
/// # Errors
///
/// Returns an error when the TOML is malformed or required fields are missing.
pub fn parse_episteme_source_manifest_toml(
    raw: &str,
) -> Result<EpistemeSourceManifest, EpistemeSourceContractParseError> {
    toml::from_str(raw).map_err(|source| EpistemeSourceContractParseError::Toml { source })
}

/// Parse episteme source-contract `files.tsv` rows from TSV text.
///
/// # Errors
///
/// Returns an error when the TSV header, row width, or numeric fields are
/// invalid.
pub fn parse_episteme_files_tsv(
    raw: &str,
) -> Result<Vec<EpistemeFileRow>, EpistemeSourceContractParseError> {
    read_tsv(raw, &FILE_FIELDS)?
        .into_iter()
        .enumerate()
        .map(|(index, row)| {
            Ok(EpistemeFileRow {
                file_id: row[0].clone(),
                relative_path: row[1].clone(),
                extension: row[2].clone(),
                byte_size: parse_number(index + 2, "byte_size", &row[3])?,
                sha256: row[4].clone(),
                category: row[5].clone(),
                language: row[6].clone(),
                extraction_route: row[7].clone(),
            })
        })
        .collect()
}

/// Parse episteme source-contract `extraction_queue.tsv` rows from TSV text.
///
/// # Errors
///
/// Returns an error when the TSV header, row width, or numeric fields are
/// invalid.
pub fn parse_episteme_extraction_queue_tsv(
    raw: &str,
) -> Result<Vec<EpistemeExtractionQueueRow>, EpistemeSourceContractParseError> {
    read_tsv(raw, &EXTRACTION_QUEUE_FIELDS)?
        .into_iter()
        .enumerate()
        .map(|(index, row)| {
            Ok(EpistemeExtractionQueueRow {
                queue_id: row[0].clone(),
                file_id: row[1].clone(),
                relative_path: row[2].clone(),
                category: row[3].clone(),
                language: row[4].clone(),
                extraction_route: row[5].clone(),
                priority: parse_number(index + 2, "priority", &row[6])?,
                output_contract: row[7].clone(),
                status: row[8].clone(),
            })
        })
        .collect()
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

fn read_tsv(
    raw: &str,
    expected_header: &[&'static str],
) -> Result<Vec<Vec<String>>, EpistemeSourceContractParseError> {
    let mut lines = raw.lines();
    let header = lines
        .next()
        .unwrap_or_default()
        .split('\t')
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let expected = expected_header
        .iter()
        .map(|field| (*field).to_string())
        .collect::<Vec<_>>();
    if header != expected {
        return Err(EpistemeSourceContractParseError::TsvHeader {
            expected,
            actual: header,
        });
    }

    lines
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            let fields = line
                .trim_end_matches('\r')
                .split('\t')
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>();
            if fields.len() != expected_header.len() {
                return Err(EpistemeSourceContractParseError::TsvRowWidth {
                    row: index + 2,
                    expected: expected_header.len(),
                    actual: fields.len(),
                });
            }
            Ok(fields)
        })
        .collect()
}

fn parse_number<T>(
    row: usize,
    field: &'static str,
    value: &str,
) -> Result<T, EpistemeSourceContractParseError>
where
    T: std::str::FromStr,
{
    value
        .parse::<T>()
        .map_err(|_| EpistemeSourceContractParseError::InvalidNumber {
            row,
            field,
            value: value.to_string(),
        })
}
