//! OCR shard metrics Arrow sidecar schema and batch builders.

use std::collections::HashMap;
use std::sync::Arc;

use arrow::array::{ArrayRef, Float64Array, Int32Array, StringArray};
use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaDataType, ArrowSchemaNullabilityPolicy,
    ArrowSchemaValidationOptions, WENDAO_TABLE_METADATA_KEY, build_arrow_schema,
    validate_record_batch_schema_with_options,
};

use super::ocr::{PdfOcrShardInput, PdfOcrShardResult};

/// Stable Arrow filename for OCR shard metrics sidecars.
pub const DOCUMENT_METRICS_ARROW_CACHE_NAME: &str = "_metrics.arrow";
/// Stable schema version for OCR shard metrics sidecars.
pub const DOCUMENT_METRICS_SCHEMA_VERSION: &str = "xiuxian_wendao.document_metrics.v1";
const DOCUMENT_METRICS_TABLE: &str = "pdf_ocr_shard_metrics";

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
    schema_ref(&document_metrics_contract())
}

/// # Errors
///
/// Returns an error if Arrow cannot build the typed metrics sidecar batch.
pub fn build_pdf_ocr_metrics_batch(metrics: &[PdfOcrShardMetric]) -> Result<RecordBatch, String> {
    record_batch(
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
        "build PDF OCR metrics Arrow batch",
    )
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

fn record_batch(columns: Vec<ArrayRef>, context: &'static str) -> Result<RecordBatch, String> {
    let contract = document_metrics_contract();
    let batch = RecordBatch::try_new(schema_ref(&contract), columns)
        .map_err(|error| format!("{context}: {error}"))?;
    validate_record_batch_schema_with_options(
        &batch,
        &contract,
        ArrowSchemaValidationOptions::new()
            .with_nullability_policy(ArrowSchemaNullabilityPolicy::Exact),
    )
    .map_err(|error| format!("{context} schema validation: {error}"))?;
    Ok(batch)
}

fn document_metrics_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        DOCUMENT_METRICS_TABLE,
        true,
        vec![
            nullable_utf8_column("contractVersion"),
            nullable_utf8_column("sourcePath"),
            nullable_utf8_column("sourceContentHash"),
            nullable_int32_column("pageIndex"),
            nullable_utf8_column("chunkId"),
            nullable_utf8_column("workerId"),
            nullable_utf8_column("shardElementId"),
            nullable_utf8_column("shardType"),
            nullable_utf8_column("ocrProfile"),
            nullable_utf8_column("status"),
            nullable_float64_column("converterInitMs"),
            nullable_float64_column("doclingConvertMs"),
            nullable_float64_column("markdownExportMs"),
            nullable_float64_column("arrowEncodeMs"),
            nullable_float64_column("cacheLookupMs"),
            nullable_float64_column("cacheWriteMs"),
            nullable_float64_column("rustSchedulerElapsedMs"),
            nullable_int32_column("pageCount"),
            nullable_int32_column("bboxCount"),
            nullable_int32_column("resultChars"),
            nullable_utf8_column("provenance"),
        ],
    )
}

fn schema_ref(contract: &ArrowSchemaContract) -> SchemaRef {
    Arc::new(build_arrow_schema(
        contract,
        [(
            WENDAO_TABLE_METADATA_KEY.to_string(),
            contract.table_name().to_string(),
        )]
        .into_iter()
        .collect::<HashMap<_, _>>(),
    ))
}

const fn nullable_utf8_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Utf8)
}

const fn nullable_int32_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Int32)
}

const fn nullable_float64_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Float64)
}

#[cfg(test)]
#[path = "../../tests/unit/pdf/metrics.rs"]
mod tests;
