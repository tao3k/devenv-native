//! REST endpoint for projecting cached Arrow document extraction results.

use std::fs::File;
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use arrow::array::{Array, Int32Array, StringArray};
use arrow::ipc::reader::FileReader;
use arrow::record_batch::RecordBatch;
use axum::{Json, extract::Query};
use serde::Deserialize;

use crate::gateway::studio::router::{GatewayState, StudioApiError};
use crate::gateway::studio::types::{DocumentExtractResource, DocumentExtractResult};
use crate::gateway::studio::vfs;

const DOCUMENT_RESOURCE_ARROW_CACHE_NAME: &str = "_resources.arrow";

/// Query parameters for document extract result retrieval.
#[derive(Debug, Deserialize)]
pub struct DocumentExtractResultQuery {
    /// VFS path to the source document.
    pub path: String,
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

    // Resolve the document path to a filesystem path so we can locate the .extracted dir
    let document_full_path =
        vfs::resolve_vfs_file_path(&state.studio, document_path).map_err(|_error| {
            StudioApiError::not_found(format!("Document not found: {document_path}"))
        })?;

    let extracted_dir = format!("{}.extracted", document_full_path.display());
    let resources_path =
        std::path::Path::new(&extracted_dir).join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME);

    if !resources_path.exists() {
        return Err(StudioApiError::not_found(format!(
            "No extraction resources found for `{document_path}`. Run document extraction first."
        )));
    }

    let resources = read_document_extract_resources(&resources_path)?;
    let total_pages = total_pages_from_resources(resources.as_slice());

    // Extraction timestamp from marker file
    let extracted_at = std::path::Path::new(&extracted_dir)
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
            resource_type: string_value(resource_type, row, "document").to_string(),
            resource_path: string_value(resource_path, row, "").to_string(),
            page_index: usize::try_from(page_index_value(page_index, row)).unwrap_or_default(),
            caption: string_value(caption, row, "").to_string(),
            content: string_value(content, row, "").to_string(),
            mime_type: string_value(mime_type, row, "text/plain").to_string(),
            status: string_value(status, row, "ok").to_string(),
            element_id: string_value(element_id, row, "").to_string(),
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
mod tests {
    use std::sync::Arc as StdArc;

    use arrow::array::{ArrayRef, Int32Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::ipc::writer::FileWriter;

    use super::*;

    #[test]
    fn document_source_format_normalizes_common_docling_suffixes() {
        assert_eq!(document_source_format("docs/manual.DOCX"), "docx");
        assert_eq!(document_source_format("slides/report.pptx"), "pptx");
        assert_eq!(document_source_format("no-extension"), "unknown");
    }

    #[test]
    fn document_extract_resources_preserve_resource_count_and_page_span() {
        let batch = document_resource_batch();

        let resources = document_extract_resources_from_batch(&batch)
            .unwrap_or_else(|error| panic!("decode resources: {error:?}"));

        assert_eq!(resources.len(), 2);
        assert_eq!(total_pages_from_resources(resources.as_slice()), 3);
        assert_eq!(resources[1].resource_type, "table");
    }

    #[test]
    fn read_document_extract_resources_decodes_arrow_ipc_file() {
        let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
        let resources_path = temp_dir.path().join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME);
        let batch = document_resource_batch();
        let file = File::create(&resources_path).unwrap_or_else(|error| panic!("file: {error}"));
        let mut writer = FileWriter::try_new(file, &batch.schema())
            .unwrap_or_else(|error| panic!("writer: {error}"));
        writer
            .write(&batch)
            .unwrap_or_else(|error| panic!("write batch: {error}"));
        writer
            .finish()
            .unwrap_or_else(|error| panic!("finish: {error}"));

        let resources = read_document_extract_resources(&resources_path)
            .unwrap_or_else(|error| panic!("read resources: {error:?}"));

        assert_eq!(resources.len(), 2);
        assert_eq!(resources[0].element_id, "_main");
    }

    fn document_resource_batch() -> RecordBatch {
        let schema = StdArc::new(Schema::new(vec![
            Field::new("sourcePath", DataType::Utf8, false),
            Field::new("resourceType", DataType::Utf8, false),
            Field::new("resourcePath", DataType::Utf8, false),
            Field::new("pageIndex", DataType::Int32, false),
            Field::new("caption", DataType::Utf8, false),
            Field::new("content", DataType::Utf8, false),
            Field::new("mimeType", DataType::Utf8, false),
            Field::new("status", DataType::Utf8, false),
            Field::new("elementId", DataType::Utf8, false),
        ]));
        RecordBatch::try_new(
            schema,
            vec![
                StdArc::new(StringArray::from(vec!["manual.docx", "manual.docx"])) as ArrayRef,
                StdArc::new(StringArray::from(vec!["document", "table"])) as ArrayRef,
                StdArc::new(StringArray::from(vec![
                    "manual.docx.extracted/manual.md",
                    "manual.docx.extracted/table-2.csv",
                ])) as ArrayRef,
                StdArc::new(Int32Array::from(vec![0, 2])) as ArrayRef,
                StdArc::new(StringArray::from(vec!["", ""])) as ArrayRef,
                StdArc::new(StringArray::from(vec!["# Manual", ""])) as ArrayRef,
                StdArc::new(StringArray::from(vec!["text/markdown", "text/csv"])) as ArrayRef,
                StdArc::new(StringArray::from(vec!["ok", "ok"])) as ArrayRef,
                StdArc::new(StringArray::from(vec!["_main", "table-2"])) as ArrayRef,
            ],
        )
        .unwrap_or_else(|error| panic!("batch: {error}"))
    }
}
