//! Manifest DTO and parser for episteme source-contract inputs.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::EpistemeSourceContractParseError;

/// Raw DTO boundary: source manifest fields mirror the portable TOML contract.
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
