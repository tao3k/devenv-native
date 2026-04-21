//! REST endpoint for retrieving cached PDF extraction results.

use std::sync::Arc;

use axum::{Json, extract::Query};
use serde::Deserialize;

use crate::gateway::studio::router::{GatewayState, StudioApiError};
use crate::gateway::studio::types::{PdfExtractResource, PdfExtractResult};
use crate::gateway::studio::vfs;

/// Query parameters for PDF extract result retrieval.
#[derive(Debug, Deserialize)]
pub struct PdfExtractResultQuery {
    /// VFS path to the source PDF.
    pub path: String,
}

/// Reads the cached `_metadata.json` for a PDF and returns structured results.
///
/// # Errors
///
/// Returns `BAD_REQUEST` if path is missing, `NOT_FOUND` if the PDF or its
/// extraction metadata does not exist.
pub async fn get_pdf_extract_result(
    Query(query): Query<PdfExtractResultQuery>,
    axum::extract::State(state): axum::extract::State<Arc<GatewayState>>,
) -> Result<Json<PdfExtractResult>, StudioApiError> {
    let pdf_path = query.path.trim();
    if pdf_path.is_empty() {
        return Err(StudioApiError::bad_request(
            "MISSING_PATH",
            "`path` query parameter is required",
        ));
    }

    // Resolve the PDF path to a filesystem path so we can locate the .extracted dir
    let pdf_full_path = vfs::resolve_vfs_file_path(&state.studio, pdf_path).map_err(|_error| {
        StudioApiError::not_found(format!("PDF not found: {pdf_path}"))
    })?;

    let extracted_dir = format!("{}.extracted", pdf_full_path.display());
    let metadata_path = std::path::Path::new(&extracted_dir).join("_metadata.json");

    if !metadata_path.exists() {
        return Err(StudioApiError::not_found(format!(
            "No extraction metadata found for `{pdf_path}`. Run PDF extraction first."
        )));
    }

    // Read and parse metadata
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

    let mut resources = Vec::with_capacity(raw_resources.len());
    let mut max_page = 0usize;

    for raw in raw_resources {
        let page_index = raw
            .get("pageIndex")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as usize;
        if page_index > max_page {
            max_page = page_index;
        }

        resources.push(PdfExtractResource {
            resource_type: raw
                .get("resourceType")
                .and_then(|v| v.as_str())
                .unwrap_or("document")
                .to_string(),
            resource_path: raw
                .get("resourcePath")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            page_index,
            caption: raw
                .get("caption")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            content: raw
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            mime_type: raw
                .get("mimeType")
                .and_then(|v| v.as_str())
                .unwrap_or("text/plain")
                .to_string(),
            status: raw
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("ok")
                .to_string(),
            element_id: raw
                .get("elementId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        });
    }

    // Extraction timestamp from marker file
    let extracted_at = std::path::Path::new(&extracted_dir)
        .join("_complete.marker")
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64);

    Ok(Json(PdfExtractResult {
        source_path: pdf_path.to_string(),
        total_pages: max_page.saturating_add(1),
        extracted_at,
        resources,
    }))
}
