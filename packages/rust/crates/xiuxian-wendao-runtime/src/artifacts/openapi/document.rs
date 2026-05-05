//! `OpenAPI` document rendering for runtime-owned artifact generation.

use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_json::Value;

const BUNDLED_WENDAO_GATEWAY_OPENAPI_RELATIVE_PATH: &str =
    "../xiuxian-wendao/resources/openapi/wendao_gateway.openapi.json";
const BUNDLED_WENDAO_GATEWAY_OPENAPI_TEXT: &str =
    include_str!("../../../../xiuxian-wendao/resources/openapi/wendao_gateway.openapi.json");

/// Return the checked-in `OpenAPI` document for the Wendao gateway.
#[must_use]
pub fn bundled_wendao_gateway_openapi_document() -> &'static str {
    BUNDLED_WENDAO_GATEWAY_OPENAPI_TEXT
}

/// Return the repository-local path for the checked-in Wendao gateway `OpenAPI` document.
#[must_use]
pub fn bundled_wendao_gateway_openapi_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(BUNDLED_WENDAO_GATEWAY_OPENAPI_RELATIVE_PATH)
}

/// Parse the checked-in Wendao gateway `OpenAPI` document.
///
/// # Errors
///
/// Returns an error when the bundled file cannot be parsed as JSON.
pub fn load_bundled_wendao_gateway_openapi_document() -> Result<Value> {
    serde_json::from_str(BUNDLED_WENDAO_GATEWAY_OPENAPI_TEXT)
        .context("failed to parse bundled Wendao gateway OpenAPI document")
}
