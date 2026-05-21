//! Document structure Arrow sidecar schema and projection API.

use std::sync::Arc;

use arrow::array::{Array, ArrayRef, Float64Array, Int32Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;

/// Stable Arrow filename for document structure sidecars.
pub const DOCUMENT_STRUCTURE_ARROW_CACHE_NAME: &str = "_structure.arrow";
/// Stable schema version for document structure sidecars.
pub const DOCUMENT_STRUCTURE_SCHEMA_VERSION: &str = "xiuxian_wendao.document_structure.v1";

/// Raw DTO boundary and stringly state boundary for document structure rows.
///
/// The row mirrors the stable Arrow structure sidecar, so block ids, resource
/// ids, block type, MIME type, status, and provenance remain serialized
/// primitive columns.
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentStructureBlock {
    pub contract_version: String,
    pub source_path: String,
    pub source_content_hash: String,
    pub block_id: String,
    pub parent_block_id: String,
    pub page_index: i32,
    pub block_index: i32,
    pub reading_order_key: String,
    pub block_type: String,
    pub resource_element_id: String,
    pub content: String,
    pub mime_type: String,
    pub status: String,
    pub engine: String,
    pub confidence: Option<f64>,
    pub bbox_left: Option<f64>,
    pub bbox_top: Option<f64>,
    pub bbox_right: Option<f64>,
    pub bbox_bottom: Option<f64>,
    pub provenance: String,
}

/// Return the stable Arrow schema for document structure rows.
#[must_use]
pub fn document_structure_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("contractVersion", DataType::Utf8, true),
        Field::new("sourcePath", DataType::Utf8, true),
        Field::new("sourceContentHash", DataType::Utf8, true),
        Field::new("blockId", DataType::Utf8, true),
        Field::new("parentBlockId", DataType::Utf8, true),
        Field::new("pageIndex", DataType::Int32, true),
        Field::new("blockIndex", DataType::Int32, true),
        Field::new("readingOrderKey", DataType::Utf8, true),
        Field::new("blockType", DataType::Utf8, true),
        Field::new("resourceElementId", DataType::Utf8, true),
        Field::new("content", DataType::Utf8, true),
        Field::new("mimeType", DataType::Utf8, true),
        Field::new("status", DataType::Utf8, true),
        Field::new("engine", DataType::Utf8, true),
        Field::new("confidence", DataType::Float64, true),
        Field::new("bboxLeft", DataType::Float64, true),
        Field::new("bboxTop", DataType::Float64, true),
        Field::new("bboxRight", DataType::Float64, true),
        Field::new("bboxBottom", DataType::Float64, true),
        Field::new("provenance", DataType::Utf8, true),
    ]))
}

/// # Errors
///
/// Returns an error if Arrow cannot build the typed structure sidecar batch.
pub fn build_document_structure_batch(
    blocks: &[DocumentStructureBlock],
) -> Result<RecordBatch, String> {
    let mut ordered = blocks.to_vec();
    ordered.sort_by(|left, right| {
        left.page_index
            .cmp(&right.page_index)
            .then(left.reading_order_key.cmp(&right.reading_order_key))
            .then(left.block_index.cmp(&right.block_index))
            .then(left.block_id.cmp(&right.block_id))
    });

    RecordBatch::try_new(
        document_structure_schema(),
        vec![
            string_column(ordered.iter().map(|block| block.contract_version.as_str())),
            string_column(ordered.iter().map(|block| block.source_path.as_str())),
            string_column(
                ordered
                    .iter()
                    .map(|block| block.source_content_hash.as_str()),
            ),
            string_column(ordered.iter().map(|block| block.block_id.as_str())),
            string_column(ordered.iter().map(|block| block.parent_block_id.as_str())),
            Arc::new(Int32Array::from(
                ordered
                    .iter()
                    .map(|block| block.page_index)
                    .collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(Int32Array::from(
                ordered
                    .iter()
                    .map(|block| block.block_index)
                    .collect::<Vec<_>>(),
            )) as ArrayRef,
            string_column(ordered.iter().map(|block| block.reading_order_key.as_str())),
            string_column(ordered.iter().map(|block| block.block_type.as_str())),
            string_column(
                ordered
                    .iter()
                    .map(|block| block.resource_element_id.as_str()),
            ),
            string_column(ordered.iter().map(|block| block.content.as_str())),
            string_column(ordered.iter().map(|block| block.mime_type.as_str())),
            string_column(ordered.iter().map(|block| block.status.as_str())),
            string_column(ordered.iter().map(|block| block.engine.as_str())),
            optional_float_column(ordered.iter().map(|block| block.confidence)),
            optional_float_column(ordered.iter().map(|block| block.bbox_left)),
            optional_float_column(ordered.iter().map(|block| block.bbox_top)),
            optional_float_column(ordered.iter().map(|block| block.bbox_right)),
            optional_float_column(ordered.iter().map(|block| block.bbox_bottom)),
            string_column(ordered.iter().map(|block| block.provenance.as_str())),
        ],
    )
    .map_err(|error| format!("build document structure Arrow batch: {error}"))
}

/// # Errors
///
/// Returns an error if the resource batch does not match the stable Wendao
/// document-resource schema.
pub fn document_resource_batch_to_structure_blocks(
    batch: &RecordBatch,
    source_content_hash: &str,
    engine: &str,
) -> Result<Vec<DocumentStructureBlock>, String> {
    let source_path = resource_string_column(batch, "sourcePath")?;
    let resource_type = resource_string_column(batch, "resourceType")?;
    let page_index = resource_i32_column(batch, "pageIndex")?;
    let content = resource_string_column(batch, "content")?;
    let mime_type = resource_string_column(batch, "mimeType")?;
    let status = resource_string_column(batch, "status")?;
    let element_id = resource_string_column(batch, "elementId")?;

    let mut blocks = Vec::with_capacity(batch.num_rows());
    for row in 0..batch.num_rows() {
        let page_index = i32_value(page_index, row);
        let block_index = i32::try_from(row).unwrap_or(i32::MAX);
        let element_id = string_value(element_id, row);
        let block_id = if element_id.trim().is_empty() {
            format!("{engine}-resource-{row:06}")
        } else {
            element_id.clone()
        };
        let provenance = serde_json::json!({
            "source": "document_resource_batch",
            "rowIndex": row,
        })
        .to_string();
        blocks.push(DocumentStructureBlock {
            contract_version: DOCUMENT_STRUCTURE_SCHEMA_VERSION.to_string(),
            source_path: string_value(source_path, row),
            source_content_hash: source_content_hash.to_string(),
            block_id,
            parent_block_id: String::new(),
            page_index,
            block_index,
            reading_order_key: format!("{:06}.{:06}", page_index.max(0), block_index.max(0)),
            block_type: string_value(resource_type, row),
            resource_element_id: element_id,
            content: string_value(content, row),
            mime_type: string_value(mime_type, row),
            status: string_value(status, row),
            engine: engine.to_string(),
            confidence: None,
            bbox_left: None,
            bbox_top: None,
            bbox_right: None,
            bbox_bottom: None,
            provenance,
        });
    }
    Ok(blocks)
}

fn string_column<'a>(values: impl IntoIterator<Item = &'a str>) -> ArrayRef {
    Arc::new(StringArray::from_iter_values(values)) as ArrayRef
}

fn optional_float_column(values: impl IntoIterator<Item = Option<f64>>) -> ArrayRef {
    Arc::new(Float64Array::from(values.into_iter().collect::<Vec<_>>())) as ArrayRef
}

fn resource_string_column<'a>(
    batch: &'a RecordBatch,
    name: &str,
) -> Result<&'a StringArray, String> {
    batch
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| format!("document resource `{name}` column is not utf8"))
}

fn resource_i32_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a Int32Array, String> {
    batch
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref::<Int32Array>())
        .ok_or_else(|| format!("document resource `{name}` column is not int32"))
}

fn string_value(column: &StringArray, row: usize) -> String {
    if column.is_null(row) {
        String::new()
    } else {
        column.value(row).to_string()
    }
}

fn i32_value(column: &Int32Array, row: usize) -> i32 {
    if column.is_null(row) {
        0
    } else {
        column.value(row)
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/pdf/structure.rs"]
mod tests;
