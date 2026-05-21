//! OCR shard metrics Arrow sidecar schema and batch builders.

use std::sync::Arc;

use arrow::array::{ArrayRef, Float64Array, Int32Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;

use super::ocr::{PdfOcrShardInput, PdfOcrShardResult};

/// Stable Arrow filename for OCR shard metrics sidecars.
pub const DOCUMENT_METRICS_ARROW_CACHE_NAME: &str = "_metrics.arrow";
/// Stable schema version for OCR shard metrics sidecars.
pub const DOCUMENT_METRICS_SCHEMA_VERSION: &str = "xiuxian_wendao.document_metrics.v1";

/// Raw DTO boundary and stringly state boundary for PDF OCR metric rows.
///
/// This struct mirrors the stable Arrow sidecar schema, so source paths,
/// shard identifiers, shard type, and status are stored as serialized
/// primitive columns rather than domain newtypes.
#[derive(Debug, Clone, PartialEq)]
pub struct PdfOcrShardMetric {
    pub contract_version: String,
    pub source_path: String,
    pub source_content_hash: String,
    pub page_index: i32,
    pub chunk_id: String,
    pub worker_id: String,
    pub shard_element_id: String,
    pub shard_type: String,
    pub ocr_profile: String,
    pub status: String,
    pub converter_init_ms: Option<f64>,
    pub docling_convert_ms: Option<f64>,
    pub markdown_export_ms: Option<f64>,
    pub arrow_encode_ms: Option<f64>,
    pub cache_lookup_ms: Option<f64>,
    pub cache_write_ms: Option<f64>,
    pub rust_scheduler_elapsed_ms: Option<f64>,
    pub page_count: i32,
    pub bbox_count: i32,
    pub result_chars: i32,
    pub provenance: String,
}

impl PdfOcrShardMetric {
    /// Build a metric row from the OCR worker input/result pair.
    #[must_use]
    pub fn from_ocr_result(
        input: &PdfOcrShardInput,
        result: &PdfOcrShardResult,
        page_count: u32,
        rust_scheduler_elapsed_ms: Option<f64>,
    ) -> Self {
        let bbox_count = i32::from(
            input.crop_right > input.crop_left
                && input.crop_top > input.crop_bottom
                && input.raster_width_px > 0
                && input.raster_height_px > 0,
        );
        let result_chars = result
            .text
            .as_deref()
            .map(|text| i32::try_from(text.chars().count()).unwrap_or(i32::MAX))
            .unwrap_or_default();
        Self {
            contract_version: DOCUMENT_METRICS_SCHEMA_VERSION.to_string(),
            source_path: input.source_path.clone(),
            source_content_hash: input.source_content_hash.clone(),
            page_index: i32::try_from(input.page_index).unwrap_or(i32::MAX),
            chunk_id: input.reading_order_key.clone(),
            worker_id: String::new(),
            shard_element_id: input.shard_element_id.clone(),
            shard_type: input.shard_type.clone(),
            ocr_profile: input.ocr_profile.clone(),
            status: result.status.as_str().to_string(),
            converter_init_ms: None,
            docling_convert_ms: None,
            markdown_export_ms: None,
            arrow_encode_ms: None,
            cache_lookup_ms: None,
            cache_write_ms: None,
            rust_scheduler_elapsed_ms,
            page_count: i32::try_from(page_count).unwrap_or(i32::MAX),
            bbox_count,
            result_chars,
            provenance: serde_json::json!({
                "source": "rust_hybrid_ocr_scheduler",
                "shardElementId": input.shard_element_id.as_str(),
                "resultElementId": result.element_id.as_str(),
                "renderProfile": input.render_profile.as_str(),
                "rasterSha256": input.raster_sha256.as_str(),
            })
            .to_string(),
        }
    }
}

/// Return the stable Arrow schema for OCR shard metrics.
#[must_use]
pub fn document_metrics_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("contractVersion", DataType::Utf8, true),
        Field::new("sourcePath", DataType::Utf8, true),
        Field::new("sourceContentHash", DataType::Utf8, true),
        Field::new("pageIndex", DataType::Int32, true),
        Field::new("chunkId", DataType::Utf8, true),
        Field::new("workerId", DataType::Utf8, true),
        Field::new("shardElementId", DataType::Utf8, true),
        Field::new("shardType", DataType::Utf8, true),
        Field::new("ocrProfile", DataType::Utf8, true),
        Field::new("status", DataType::Utf8, true),
        Field::new("converterInitMs", DataType::Float64, true),
        Field::new("doclingConvertMs", DataType::Float64, true),
        Field::new("markdownExportMs", DataType::Float64, true),
        Field::new("arrowEncodeMs", DataType::Float64, true),
        Field::new("cacheLookupMs", DataType::Float64, true),
        Field::new("cacheWriteMs", DataType::Float64, true),
        Field::new("rustSchedulerElapsedMs", DataType::Float64, true),
        Field::new("pageCount", DataType::Int32, true),
        Field::new("bboxCount", DataType::Int32, true),
        Field::new("resultChars", DataType::Int32, true),
        Field::new("provenance", DataType::Utf8, true),
    ]))
}

/// # Errors
///
/// Returns an error if Arrow cannot build the typed metrics sidecar batch.
pub fn build_pdf_ocr_metrics_batch(metrics: &[PdfOcrShardMetric]) -> Result<RecordBatch, String> {
    RecordBatch::try_new(
        document_metrics_schema(),
        vec![
            string_column(
                metrics
                    .iter()
                    .map(|metric| metric.contract_version.as_str()),
            ),
            string_column(metrics.iter().map(|metric| metric.source_path.as_str())),
            string_column(
                metrics
                    .iter()
                    .map(|metric| metric.source_content_hash.as_str()),
            ),
            int_column(metrics.iter().map(|metric| metric.page_index)),
            string_column(metrics.iter().map(|metric| metric.chunk_id.as_str())),
            string_column(metrics.iter().map(|metric| metric.worker_id.as_str())),
            string_column(
                metrics
                    .iter()
                    .map(|metric| metric.shard_element_id.as_str()),
            ),
            string_column(metrics.iter().map(|metric| metric.shard_type.as_str())),
            string_column(metrics.iter().map(|metric| metric.ocr_profile.as_str())),
            string_column(metrics.iter().map(|metric| metric.status.as_str())),
            optional_float_column(metrics.iter().map(|metric| metric.converter_init_ms)),
            optional_float_column(metrics.iter().map(|metric| metric.docling_convert_ms)),
            optional_float_column(metrics.iter().map(|metric| metric.markdown_export_ms)),
            optional_float_column(metrics.iter().map(|metric| metric.arrow_encode_ms)),
            optional_float_column(metrics.iter().map(|metric| metric.cache_lookup_ms)),
            optional_float_column(metrics.iter().map(|metric| metric.cache_write_ms)),
            optional_float_column(
                metrics
                    .iter()
                    .map(|metric| metric.rust_scheduler_elapsed_ms),
            ),
            int_column(metrics.iter().map(|metric| metric.page_count)),
            int_column(metrics.iter().map(|metric| metric.bbox_count)),
            int_column(metrics.iter().map(|metric| metric.result_chars)),
            string_column(metrics.iter().map(|metric| metric.provenance.as_str())),
        ],
    )
    .map_err(|error| format!("build PDF OCR metrics Arrow batch: {error}"))
}

fn string_column<'a>(values: impl IntoIterator<Item = &'a str>) -> ArrayRef {
    Arc::new(StringArray::from_iter_values(values)) as ArrayRef
}

fn int_column(values: impl IntoIterator<Item = i32>) -> ArrayRef {
    Arc::new(Int32Array::from(values.into_iter().collect::<Vec<_>>())) as ArrayRef
}

fn optional_float_column(values: impl IntoIterator<Item = Option<f64>>) -> ArrayRef {
    Arc::new(Float64Array::from(values.into_iter().collect::<Vec<_>>())) as ArrayRef
}

#[cfg(test)]
#[path = "../../tests/unit/pdf/metrics.rs"]
mod tests;
