//! REST endpoint for retrieving individual PDF extraction resources.
//!
//! Front-end must **never** use `resourcePath` (a filesystem path) directly.
//! All extracted resources are served through this endpoint keyed by the
//! source PDF VFS path and the element id.

use std::sync::Arc;

use axum::{
    extract::Query,
    http::{HeaderValue, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};
use serde::Deserialize;

use crate::gateway::studio::router::{GatewayState, StudioApiError};
use crate::gateway::studio::vfs;

/// Query parameters for PDF extract resource retrieval.
#[derive(Debug, Deserialize)]
pub struct PdfExtractResourceQuery {
    /// VFS path to the source PDF.
    pub path: String,
    /// Element id within the extracted metadata (e.g. `_main`, `img_001`).
    pub element_id: String,
}

/// Streams a single extracted resource (text or binary) by VFS path + element id.
///
/// # Errors
///
/// Returns `BAD_REQUEST` for missing parameters, `NOT_FOUND` when the PDF,
/// its extraction metadata, or the requested element does not exist.
pub async fn get_pdf_extract_resource(
    Query(query): Query<PdfExtractResourceQuery>,
    axum::extract::State(state): axum::extract::State<Arc<GatewayState>>,
) -> Result<Response, StudioApiError> {
    let pdf_path = query.path.trim();
    let element_id = query.element_id.trim();

    if pdf_path.is_empty() {
        return Err(StudioApiError::bad_request(
            "MISSING_PATH",
            "`path` query parameter is required",
        ));
    }
    if element_id.is_empty() {
        return Err(StudioApiError::bad_request(
            "MISSING_ELEMENT_ID",
            "`element_id` query parameter is required",
        ));
    }

    // Resolve VFS path to filesystem path so we can locate the .extracted dir
    let pdf_full_path =
        vfs::resolve_vfs_file_path(&state.studio, pdf_path).map_err(|_error| {
            StudioApiError::not_found(format!("PDF not found: {pdf_path}"))
        })?;

    let extracted_dir = format!("{}.extracted", pdf_full_path.display());
    let metadata_path = std::path::Path::new(&extracted_dir).join("_metadata.json");

    if !metadata_path.exists() {
        return Err(StudioApiError::not_found(format!(
            "No extraction metadata found for `{pdf_path}`. Run PDF extraction first."
        )));
    }

    let metadata_json = std::fs::read_to_string(&metadata_path).map_err(|error| {
        StudioApiError::internal(
            "METADATA_READ_ERROR",
            format!("Failed to read extraction metadata: {error}"),
            None,
        )
    })?;

    let raw_resources: Vec<serde_json::Map<String, serde_json::Value>> =
        serde_json::from_str(&metadata_json).map_err(|error| {
            StudioApiError::internal(
                "METADATA_PARSE_ERROR",
                format!("Failed to parse extraction metadata: {error}"),
                None,
            )
        })?;

    let resource = raw_resources
        .into_iter()
        .find(|r| {
            r.get("elementId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                == element_id
        })
        .ok_or_else(|| {
            StudioApiError::not_found(format!(
                "Element `{element_id}` not found in extraction metadata for `{pdf_path}`."
            ))
        })?;

    let _resource_type = resource
        .get("resourceType")
        .and_then(|v| v.as_str())
        .unwrap_or("document");
    let mime_type = resource
        .get("mimeType")
        .and_then(|v| v.as_str())
        .unwrap_or("application/octet-stream");
    let content = resource
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let resource_path = resource
        .get("resourcePath")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // For the canonical _main document, the full text is already in `content`.
    // Return it directly without touching the filesystem again.
    if element_id == "_main" && !content.is_empty() {
        let content_type = HeaderValue::from_str(mime_type).map_err(|error| {
            StudioApiError::internal(
                "INVALID_CONTENT_TYPE",
                "Failed to render content type header",
                Some(error.to_string()),
            )
        })?;
        return Ok((
            StatusCode::OK,
            [(CONTENT_TYPE, content_type)],
            content.to_string(),
        )
            .into_response());
    }

    // For binary resources (images, etc.), read the file from disk.
    if resource_path.is_empty() {
        return Err(StudioApiError::not_found(format!(
            "Resource `{element_id}` has no file path."
        )));
    }

    let file_path = std::path::Path::new(resource_path);
    if !file_path.exists() {
        return Err(StudioApiError::not_found(format!(
            "Extracted file not found: {resource_path}"
        )));
    }

    let bytes = std::fs::read(file_path).map_err(|error| {
        StudioApiError::internal(
            "FILE_READ_ERROR",
            format!("Failed to read extracted file: {error}"),
            None,
        )
    })?;

    let content_type = HeaderValue::from_str(mime_type).map_err(|error| {
        StudioApiError::internal(
            "INVALID_CONTENT_TYPE",
            "Failed to render content type header",
            Some(error.to_string()),
        )
    })?;

    Ok((StatusCode::OK, [(CONTENT_TYPE, content_type)], bytes).into_response())
}
