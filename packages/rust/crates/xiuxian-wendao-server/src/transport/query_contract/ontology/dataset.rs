//! Dataset-to-ontology Flight handoff contract.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

/// Stable route for dataset-to-ontology materialization handoff.
pub const ONTOLOGY_DATASET_MATERIALIZE_ROUTE: &str = "/ontology/dataset/materialize";
/// Canonical dataset-to-ontology contract-id metadata header.
pub const WENDAO_DATASET_ONTOLOGY_CONTRACT_ID_HEADER: &str =
    "x-wendao-dataset-ontology-contract-id";
/// Canonical dataset-to-ontology mapping-id metadata header.
pub const WENDAO_DATASET_ONTOLOGY_MAPPING_ID_HEADER: &str = "x-wendao-dataset-ontology-mapping-id";
/// Canonical dataset-to-ontology manifest metadata header.
pub const WENDAO_DATASET_ONTOLOGY_MANIFEST_HEADER: &str = "x-wendao-dataset-ontology-manifest";
/// Per-payload table identifier header for future multi-stream uploads.
pub const WENDAO_DATASET_ONTOLOGY_PAYLOAD_ID_HEADER: &str = "x-wendao-dataset-ontology-payload-id";
/// Dataset-to-ontology Flight handoff manifest schema version.
pub const DATASET_ONTOLOGY_HANDOFF_SCHEMA_VERSION: &str =
    "xiuxian_wendao.dataset_ontology_handoff.v1";

/// Multi-table dataset-to-ontology handoff manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetOntologyFlightManifest {
    /// Manifest schema version.
    pub schema_version: String,
    /// Accepted source-contract identifier.
    pub contract_id: String,
    /// Accepted mapping identifier within the source contract.
    pub mapping_id: String,
    /// Raw Arrow source table payloads expected by the materializer.
    pub tables: Vec<DatasetOntologySourceTablePayload>,
}

impl DatasetOntologyFlightManifest {
    /// Create a manifest with the current schema version.
    #[must_use]
    pub fn new(
        contract_id: impl Into<String>,
        mapping_id: impl Into<String>,
        tables: Vec<DatasetOntologySourceTablePayload>,
    ) -> Self {
        Self {
            schema_version: DATASET_ONTOLOGY_HANDOFF_SCHEMA_VERSION.to_string(),
            contract_id: contract_id.into(),
            mapping_id: mapping_id.into(),
            tables,
        }
    }
}

/// One raw Arrow table payload listed in a dataset-to-ontology handoff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetOntologySourceTablePayload {
    /// SQL table name to register for mapping execution.
    pub table_name: String,
    /// Stable payload identifier used to match an Arrow stream to this table.
    pub payload_id: String,
    /// Optional source row count. When provided, it must be positive.
    pub row_count: Option<u64>,
    /// Optional source content fingerprint.
    pub content_sha256: Option<String>,
    /// Optional Arrow schema fingerprint.
    pub schema_fingerprint: Option<String>,
}

impl DatasetOntologySourceTablePayload {
    /// Create one source table payload descriptor.
    #[must_use]
    pub fn new(table_name: impl Into<String>, payload_id: impl Into<String>) -> Self {
        Self {
            table_name: table_name.into(),
            payload_id: payload_id.into(),
            row_count: None,
            content_sha256: None,
            schema_fingerprint: None,
        }
    }

    /// Attach a known source row count.
    #[must_use]
    pub const fn with_row_count(mut self, row_count: u64) -> Self {
        self.row_count = Some(row_count);
        self
    }

    /// Attach a source content fingerprint.
    #[must_use]
    pub fn with_content_sha256(mut self, content_sha256: impl Into<String>) -> Self {
        self.content_sha256 = Some(content_sha256.into());
        self
    }

    /// Attach an Arrow schema fingerprint.
    #[must_use]
    pub fn with_schema_fingerprint(mut self, schema_fingerprint: impl Into<String>) -> Self {
        self.schema_fingerprint = Some(schema_fingerprint.into());
        self
    }
}

/// Encode a dataset-to-ontology handoff manifest for a metadata header.
///
/// # Errors
///
/// Returns an error when the manifest is invalid or cannot be serialized.
pub fn encode_dataset_ontology_manifest_header(
    manifest: &DatasetOntologyFlightManifest,
) -> Result<String, String> {
    validate_dataset_ontology_flight_manifest(manifest)?;
    serde_json::to_string(manifest)
        .map_err(|error| format!("failed to encode dataset ontology manifest: {error}"))
}

/// Decode a dataset-to-ontology handoff manifest from a metadata header.
///
/// # Errors
///
/// Returns an error when the header is not valid JSON or fails manifest
/// validation.
pub fn decode_dataset_ontology_manifest_header(
    header_value: &str,
) -> Result<DatasetOntologyFlightManifest, String> {
    let manifest = serde_json::from_str::<DatasetOntologyFlightManifest>(header_value)
        .map_err(|error| format!("failed to decode dataset ontology manifest: {error}"))?;
    validate_dataset_ontology_flight_manifest(&manifest)?;
    Ok(manifest)
}

/// Validate one dataset-to-ontology Flight handoff manifest.
///
/// # Errors
///
/// Returns an error when the manifest schema version is unsupported, required
/// identifiers are blank, source table payloads are missing, payload identity
/// is duplicated, or optional fingerprints are blank.
pub fn validate_dataset_ontology_flight_manifest(
    manifest: &DatasetOntologyFlightManifest,
) -> Result<(), String> {
    if manifest.schema_version != DATASET_ONTOLOGY_HANDOFF_SCHEMA_VERSION {
        return Err(format!(
            "unsupported dataset ontology manifest schema version `{}`",
            manifest.schema_version
        ));
    }
    validate_nonblank(
        &manifest.contract_id,
        "dataset ontology contract id must not be blank",
    )?;
    validate_nonblank(
        &manifest.mapping_id,
        "dataset ontology mapping id must not be blank",
    )?;
    if manifest.tables.is_empty() {
        return Err("dataset ontology manifest must include at least one source table".to_string());
    }

    let mut table_names = BTreeSet::new();
    let mut payload_ids = BTreeSet::new();
    for table in &manifest.tables {
        validate_nonblank(
            &table.table_name,
            "dataset ontology source table name must not be blank",
        )?;
        validate_nonblank(
            &table.payload_id,
            "dataset ontology source table payload id must not be blank",
        )?;
        if !table_names.insert(table.table_name.as_str()) {
            return Err(format!(
                "dataset ontology manifest contains duplicate source table `{}`",
                table.table_name
            ));
        }
        if !payload_ids.insert(table.payload_id.as_str()) {
            return Err(format!(
                "dataset ontology manifest contains duplicate payload id `{}`",
                table.payload_id
            ));
        }
        if table.row_count == Some(0) {
            return Err(format!(
                "dataset ontology source table `{}` row count must be positive when provided",
                table.table_name
            ));
        }
        validate_optional_nonblank(
            table.content_sha256.as_deref(),
            "dataset ontology source table content fingerprint must not be blank",
        )?;
        validate_optional_nonblank(
            table.schema_fingerprint.as_deref(),
            "dataset ontology source table schema fingerprint must not be blank",
        )?;
    }
    Ok(())
}

fn validate_nonblank(value: &str, message: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(message.to_string());
    }
    Ok(())
}

fn validate_optional_nonblank(value: Option<&str>, message: &str) -> Result<(), String> {
    if value.is_some_and(|value| value.trim().is_empty()) {
        return Err(message.to_string());
    }
    Ok(())
}
