//! REST endpoint for projecting cached Arrow document extraction results.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use arrow::array::{Array, Int32Array, StringArray};
use arrow::ipc::reader::FileReader;
use arrow::record_batch::RecordBatch;
use axum::{Json, extract::Query};
use serde::Deserialize;

use super::analysis::{StudioDocumentExtractFlightRouteProvider, default_output_dir};
use crate::studio::router::{GatewayState, StudioApiError};
use crate::studio::types::{DocumentExtractResource, DocumentExtractResult};
use crate::studio::vfs;

const DOCUMENT_RESOURCE_ARROW_CACHE_NAME: &str = "_resources.arrow";

/// Query parameters for document extract result retrieval.
#[derive(Debug, Deserialize)]
pub struct DocumentExtractResultQuery {
    /// VFS path to the source document.
    pub path: String,
}

#[derive(Debug, Clone)]
pub(super) struct DocumentExtractCacheLocation {
    pub(super) output_dir: PathBuf,
    pub(super) resources_path: PathBuf,
}

/// Reads the cached Arrow IPC resource table for a document and returns structured results.
///
/// # Errors
///
/// Returns `BAD_REQUEST` if path is missing, `NOT_FOUND` if the document or its
/// Arrow resource cache does not exist.
pub async fn get_document_extract_result(
    Query(query): Query<DocumentExtractResultQuery>,
    axum::extract::State(state): axum::extract::State<Arc<GatewayState>>,
) -> Result<Json<DocumentExtractResult>, StudioApiError> {
    let document_path = query.path.trim();
    if document_path.is_empty() {
        return Err(StudioApiError::bad_request(
            "MISSING_PATH",
            "`path` query parameter is required",
        ));
    }

    let cache_location = document_extract_cache_location(&state, document_path)?;
    let resources = read_document_extract_resources(&cache_location.resources_path)?;
    let total_pages = total_pages_from_resources(resources.as_slice());

    // Extraction timestamp from marker file
    let extracted_at = cache_location
        .output_dir
        .join("_complete.marker")
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|duration| i64::try_from(duration.as_secs()).unwrap_or(i64::MAX));

    Ok(Json(DocumentExtractResult {
        source_path: document_path.to_string(),
        source_format: document_source_format(document_path),
        total_resources: resources.len(),
        total_pages,
        extracted_at,
        resources,
    }))
}

pub(super) fn document_extract_cache_location(
    state: &GatewayState,
    document_path: &str,
) -> Result<DocumentExtractCacheLocation, StudioApiError> {
    let document_full_path =
        vfs::resolve_vfs_file_path(&state.studio, document_path).map_err(|_error| {
            StudioApiError::not_found(format!("Document not found: {document_path}"))
        })?;
    let output_dir = document_extract_output_dir_for_source(state, document_full_path.as_path())?;
    let resources_path = output_dir.join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME);
    if resources_path.exists() {
        return Ok(DocumentExtractCacheLocation {
            output_dir,
            resources_path,
        });
    }
    Err(StudioApiError::not_found(format!(
        "No extraction resources found for `{document_path}`. Run document extraction first."
    )))
}

fn document_extract_output_dir_for_source(
    state: &GatewayState,
    source_path: &Path,
) -> Result<PathBuf, StudioApiError> {
    let provider = StudioDocumentExtractFlightRouteProvider::new(state);
    provider
        .succeeded_output_dir_for_source(source_path)
        .map_err(|error| {
            StudioApiError::internal(
                "DOCUMENT_EXTRACT_JOB_LOOKUP_FAILED",
                "Failed to read document extraction job registry",
                Some(error),
            )
        })
        .map(|output_dir| output_dir.unwrap_or_else(|| default_output_dir(source_path)))
}

pub(super) fn read_document_extract_resources(
    resources_path: &std::path::Path,
) -> Result<Vec<DocumentExtractResource>, StudioApiError> {
    let file = File::open(resources_path).map_err(|error| {
        StudioApiError::internal(
            "RESOURCE_ARROW_READ_ERROR",
            format!("Failed to read extraction Arrow cache: {error}"),
            None,
        )
    })?;
    let reader = FileReader::try_new(file, None).map_err(|error| {
        StudioApiError::internal(
            "RESOURCE_ARROW_DECODE_ERROR",
            format!("Failed to decode extraction Arrow cache: {error}"),
            None,
        )
    })?;

    let mut resources = Vec::new();
    for batch in reader {
        let batch = batch.map_err(|error| {
            StudioApiError::internal(
                "RESOURCE_ARROW_BATCH_ERROR",
                format!("Failed to read extraction Arrow batch: {error}"),
                None,
            )
        })?;
        resources.extend(document_extract_resources_from_batch(&batch)?);
    }
    Ok(resources)
}

fn document_extract_resources_from_batch(
    batch: &RecordBatch,
) -> Result<Vec<DocumentExtractResource>, StudioApiError> {
    let resource_type = string_column(batch, "resourceType")?;
    let resource_path = string_column(batch, "resourcePath")?;
    let page_index = i32_column(batch, "pageIndex")?;
    let caption = string_column(batch, "caption")?;
    let content = string_column(batch, "content")?;
    let mime_type = string_column(batch, "mimeType")?;
    let status = string_column(batch, "status")?;
    let element_id = string_column(batch, "elementId")?;

    let mut resources = Vec::with_capacity(batch.num_rows());
    for row in 0..batch.num_rows() {
        resources.push(DocumentExtractResource {
            resource_type: string_value(resource_type, row, "document")
                .to_string()
                .into(),
            resource_path: string_value(resource_path, row, "").to_string().into(),
            page_index: usize::try_from(page_index_value(page_index, row)).unwrap_or_default(),
            caption: string_value(caption, row, "").to_string(),
            content: string_value(content, row, "").to_string(),
            mime_type: string_value(mime_type, row, "text/plain")
                .to_string()
                .into(),
            status: string_value(status, row, "ok").to_string().into(),
            element_id: string_value(element_id, row, "").to_string().into(),
        });
    }
    Ok(resources)
}

fn string_column<'a>(
    batch: &'a RecordBatch,
    name: &str,
) -> Result<&'a StringArray, StudioApiError> {
    batch
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| {
            StudioApiError::internal(
                "RESOURCE_ARROW_SCHEMA_ERROR",
                format!("Extraction Arrow cache is missing string column `{name}`"),
                None,
            )
        })
}

fn i32_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a Int32Array, StudioApiError> {
    batch
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref::<Int32Array>())
        .ok_or_else(|| {
            StudioApiError::internal(
                "RESOURCE_ARROW_SCHEMA_ERROR",
                format!("Extraction Arrow cache is missing int32 column `{name}`"),
                None,
            )
        })
}

fn string_value<'a>(array: &'a StringArray, row: usize, default: &'a str) -> &'a str {
    if array.is_null(row) {
        default
    } else {
        array.value(row)
    }
}

fn page_index_value(array: &Int32Array, row: usize) -> i32 {
    if array.is_null(row) {
        0
    } else {
        array.value(row)
    }
}

fn total_pages_from_resources(resources: &[DocumentExtractResource]) -> usize {
    resources
        .iter()
        .map(|resource| resource.page_index)
        .max()
        .map_or(0, |page_index| page_index.saturating_add(1))
}

fn document_source_format(document_path: &str) -> String {
    std::path::Path::new(document_path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.trim_start_matches('.').to_ascii_lowercase())
        .filter(|extension| !extension.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
#[path = "../../../../tests/unit/gateway/studio/router/handlers/document_extract_result.rs"]
mod tests;
