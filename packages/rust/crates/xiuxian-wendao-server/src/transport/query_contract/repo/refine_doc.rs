//! Refine-doc route contract and metadata validation.

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};

/// Canonical refine-doc repository metadata header for Wendao Flight requests.
pub const WENDAO_REFINE_DOC_REPO_HEADER: &str = "x-wendao-refine-doc-repo";
/// Canonical refine-doc entity identifier metadata header for Wendao Flight requests.
pub const WENDAO_REFINE_DOC_ENTITY_ID_HEADER: &str = "x-wendao-refine-doc-entity-id";
/// Canonical refine-doc user hints metadata header for Wendao Flight requests.
pub const WENDAO_REFINE_DOC_USER_HINTS_HEADER: &str = "x-wendao-refine-doc-user-hints-b64";
/// Stable route for the refine-doc analysis contract.
pub const ANALYSIS_REFINE_DOC_ROUTE: &str = "/analysis/refine-doc";

/// Validate the stable refine-doc request contract.
///
/// # Errors
///
/// Returns an error when the repository identifier or entity identifier is
/// blank, or when the optional Base64-encoded user hints cannot be decoded
/// into valid UTF-8.
pub fn validate_refine_doc_request(
    repo_id: &str,
    entity_id: &str,
    user_hints_base64: Option<&str>,
) -> Result<(String, String, Option<String>), String> {
    let normalized_repo_id = repo_id.trim();
    if normalized_repo_id.is_empty() {
        return Err("refine doc repo must not be blank".to_string());
    }
    let normalized_entity_id = entity_id.trim();
    if normalized_entity_id.is_empty() {
        return Err("refine doc entity_id must not be blank".to_string());
    }
    let normalized_user_hints = user_hints_base64
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            let decoded = BASE64_STANDARD
                .decode(value)
                .map_err(|error| format!("refine doc user_hints must be valid Base64: {error}"))?;
            String::from_utf8(decoded)
                .map_err(|error| format!("refine doc user_hints must be valid UTF-8: {error}"))
        })
        .transpose()?;
    Ok((
        normalized_repo_id.to_string(),
        normalized_entity_id.to_string(),
        normalized_user_hints,
    ))
}
