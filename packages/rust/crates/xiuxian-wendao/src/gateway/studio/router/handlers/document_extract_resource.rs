//! REST endpoint for retrieving individual document extraction resources.
//!
//! Front-end must **never** use `resourcePath` (a filesystem path) directly.
//! All extracted resources are served through this endpoint keyed by the
//! source document VFS path and the element id.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::{
    extract::Query,
    http::{HeaderValue, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};
use serde::Deserialize;

use super::document_extract_result::{
    document_extract_cache_location, read_document_extract_resources,
};
use crate::gateway::studio::router::{GatewayState, StudioApiError};

/// Query parameters for document extract resource retrieval.
#[derive(Debug, Deserialize)]
pub struct DocumentExtractResourceQuery {
    /// VFS path to the source document.
    pub path: String,
    /// Element id within the extracted metadata (e.g. `_main`, `img_001`).
    pub element_id: String,
}

/// Streams a single extracted resource (text or binary) by VFS path + element id.
///
/// # Errors
///
/// Returns `BAD_REQUEST` for missing parameters, `NOT_FOUND` when the document,
/// its Arrow resource cache, or the requested element does not exist.
pub async fn get_document_extract_resource(
    Query(query): Query<DocumentExtractResourceQuery>,
    axum::extract::State(state): axum::extract::State<Arc<GatewayState>>,
) -> Result<Response, StudioApiError> {
    let (document_path, element_id) = validate_resource_query(&query)?;
    let cache_location = document_extract_cache_location(&state, document_path)?;
    let resources = read_document_extract_resources(&cache_location.resources_path)?;
    let resource = find_resource(resources, document_path, element_id)?;

    let mime_type = if resource.mime_type.is_empty() {
        "application/octet-stream"
    } else {
        resource.mime_type.as_str()
    };
    let content = resource.content.as_str();
    let resource_path = resource.resource_path.as_str();

    // For the canonical _main document, the full text is already in `content`.
    // Return it directly without touching the filesystem again.
    if element_id == "_main" && !content.is_empty() {
        return response_with_content_type(mime_type, content.to_string());
    }

    // For binary resources (images, etc.), read the file from disk.
    if resource_path.is_empty() {
        return Err(StudioApiError::not_found(format!(
            "Resource `{element_id}` has no file path."
        )));
    }

    let file_path = resolve_resource_file(cache_location.output_dir.as_path(), resource_path)?;

    let bytes = std::fs::read(file_path).map_err(|error| {
        StudioApiError::internal(
            "FILE_READ_ERROR",
            format!("Failed to read extracted file: {error}"),
            None,
        )
    })?;

    response_with_content_type(mime_type, bytes)
}

fn validate_resource_query(
    query: &DocumentExtractResourceQuery,
) -> Result<(&str, &str), StudioApiError> {
    let document_path = query.path.trim();
    let element_id = query.element_id.trim();

    if document_path.is_empty() {
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
    Ok((document_path, element_id))
}

fn find_resource(
    resources: Vec<crate::gateway::studio::types::DocumentExtractResource>,
    document_path: &str,
    element_id: &str,
) -> Result<crate::gateway::studio::types::DocumentExtractResource, StudioApiError> {
    resources
        .into_iter()
        .find(|resource| resource.element_id == element_id)
        .ok_or_else(|| {
            StudioApiError::not_found(format!(
                "Element `{element_id}` not found in extraction resources for `{document_path}`."
            ))
        })
}

fn resolve_resource_file(
    extraction_root: &Path,
    resource_path: &str,
) -> Result<PathBuf, StudioApiError> {
    let raw_path = Path::new(resource_path);
    let candidate = if raw_path.is_absolute() {
        raw_path.to_path_buf()
    } else {
        extraction_root.join(raw_path)
    };
    if !candidate.exists() {
        return Err(StudioApiError::not_found(format!(
            "Extracted file not found: {resource_path}"
        )));
    }
    let root = extraction_root.canonicalize().map_err(|error| {
        StudioApiError::internal(
            "EXTRACTION_ROOT_RESOLVE_ERROR",
            "Failed to resolve extraction output directory",
            Some(error.to_string()),
        )
    })?;
    let file = candidate.canonicalize().map_err(|error| {
        StudioApiError::internal(
            "RESOURCE_PATH_RESOLVE_ERROR",
            "Failed to resolve extracted resource path",
            Some(error.to_string()),
        )
    })?;
    if !file.starts_with(root.as_path()) {
        return Err(StudioApiError::bad_request(
            "RESOURCE_OUTSIDE_EXTRACTION_ROOT",
            "Extracted resource path is outside the extraction output directory",
        ));
    }
    if !file.is_file() {
        return Err(StudioApiError::bad_request(
            "RESOURCE_NOT_FILE",
            "Extracted resource path does not point to a file",
        ));
    }
    Ok(file)
}

fn response_with_content_type(
    mime_type: &str,
    body: impl IntoResponse,
) -> Result<Response, StudioApiError> {
    let content_type = HeaderValue::from_str(mime_type).map_err(|error| {
        StudioApiError::internal(
            "INVALID_CONTENT_TYPE",
            "Failed to render content type header",
            Some(error.to_string()),
        )
    })?;
    Ok((StatusCode::OK, [(CONTENT_TYPE, content_type)], body).into_response())
}

#[cfg(test)]
#[path = "../../../../../tests/unit/gateway/studio/router/handlers/document_extract_resource.rs"]
mod tests;
