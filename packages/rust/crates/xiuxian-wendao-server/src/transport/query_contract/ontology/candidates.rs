//! Ontology candidate inspection Flight contract.

use serde::{Deserialize, Serialize};

/// Stable route for ontology candidate inspection over Arrow Flight.
pub const ONTOLOGY_CANDIDATE_INSPECT_ROUTE: &str = "/ontology/candidates/inspect";
/// Canonical ontology candidate inspection metadata header.
pub const WENDAO_ONTOLOGY_CANDIDATE_INSPECTION_REQUEST_HEADER: &str =
    "x-wendao-ontology-candidate-inspection";
/// Ontology candidate inspection request schema version.
pub const ONTOLOGY_CANDIDATE_INSPECTION_SCHEMA_VERSION: &str =
    "xiuxian_wendao.ontology_candidate_inspection.v1";

/// Flight request for inspecting one ontology candidate generation run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OntologyCandidateInspectionFlightRequest {
    /// Request schema version.
    pub schema_version: String,
    /// Episteme registry id configured by the Gateway host.
    pub episteme_registry_id: String,
    /// Safe ontology-generation run id.
    pub run_id: String,
}

impl OntologyCandidateInspectionFlightRequest {
    /// Create a request with the current schema version.
    #[must_use]
    pub fn new(episteme_registry_id: impl Into<String>, run_id: impl Into<String>) -> Self {
        Self {
            schema_version: ONTOLOGY_CANDIDATE_INSPECTION_SCHEMA_VERSION.to_string(),
            episteme_registry_id: episteme_registry_id.into(),
            run_id: run_id.into(),
        }
    }
}

/// Encode an ontology candidate inspection request for metadata.
///
/// # Errors
///
/// Returns an error when the request is invalid or cannot be serialized.
pub fn encode_ontology_candidate_inspection_request_header(
    request: &OntologyCandidateInspectionFlightRequest,
) -> Result<String, String> {
    validate_ontology_candidate_inspection_request(request)?;
    serde_json::to_string(request)
        .map_err(|error| format!("failed to encode ontology candidate inspection request: {error}"))
}

/// Decode an ontology candidate inspection request from metadata.
///
/// # Errors
///
/// Returns an error when the metadata is not valid JSON or fails request
/// validation.
pub fn decode_ontology_candidate_inspection_request_header(
    header_value: &str,
) -> Result<OntologyCandidateInspectionFlightRequest, String> {
    let request = serde_json::from_str::<OntologyCandidateInspectionFlightRequest>(header_value)
        .map_err(|error| {
            format!("failed to decode ontology candidate inspection request: {error}")
        })?;
    validate_ontology_candidate_inspection_request(&request)?;
    Ok(request)
}

/// Validate one ontology candidate inspection Flight request.
///
/// # Errors
///
/// Returns an error when the schema version is unsupported or identifiers are
/// blank or unsafe for run-directory resolution.
pub fn validate_ontology_candidate_inspection_request(
    request: &OntologyCandidateInspectionFlightRequest,
) -> Result<(), String> {
    if request.schema_version != ONTOLOGY_CANDIDATE_INSPECTION_SCHEMA_VERSION {
        return Err(format!(
            "unsupported ontology candidate inspection schema version `{}`",
            request.schema_version
        ));
    }
    validate_safe_identifier(
        &request.episteme_registry_id,
        "ontology candidate inspection episteme registry id",
    )?;
    validate_safe_identifier(&request.run_id, "ontology candidate inspection run id")?;
    Ok(())
}

fn validate_safe_identifier(value: &str, label: &str) -> Result<(), String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{label} must not be blank"));
    }
    if trimmed == "." || trimmed == ".." {
        return Err(format!("{label} `{value}` is not safe"));
    }
    if !trimmed
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'))
    {
        return Err(format!("{label} `{value}` is not safe"));
    }
    Ok(())
}
