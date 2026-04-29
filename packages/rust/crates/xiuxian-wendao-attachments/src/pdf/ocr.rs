use std::sync::Arc;

use arrow::array::{ArrayRef, BooleanArray, Float64Array, Int32Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::render::PdfPageShardManifest;

pub const PDF_OCR_SHARD_INPUT_SCHEMA_VERSION: &str = "xiuxian_wendao.pdf_ocr_shard_input.v1";
pub const PDF_OCR_SHARD_RESULT_SCHEMA_VERSION: &str = "xiuxian_wendao.pdf_ocr_shard_result.v1";
pub const PDF_OCR_DEFAULT_PROFILE: &str = "docling-compatible-page-ocr-v1";

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfOcrWorkerProfile {
    pub profile_id: String,
    pub engine: String,
    pub preferred_languages: Vec<String>,
    pub min_confidence: f64,
    pub preserve_layout: bool,
}

impl PdfOcrWorkerProfile {
    #[must_use]
    pub fn docling_compatible() -> Self {
        Self {
            profile_id: PDF_OCR_DEFAULT_PROFILE.to_string(),
            engine: "docling-compatible-ocr".to_string(),
            preferred_languages: vec!["auto".to_string()],
            min_confidence: 0.0,
            preserve_layout: true,
        }
    }
}

impl Default for PdfOcrWorkerProfile {
    fn default() -> Self {
        Self::docling_compatible()
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfOcrShardInput {
    pub contract_version: String,
    pub source_path: String,
    pub source_content_hash: String,
    pub page_index: u32,
    pub image_path: String,
    pub image_mime_type: String,
    pub raster_sha256: String,
    pub render_profile: String,
    pub ocr_profile: String,
    pub ocr_engine: String,
    pub preferred_languages: Vec<String>,
    pub min_confidence: f64,
    pub preserve_layout: bool,
    pub raster_width_px: u32,
    pub raster_height_px: u32,
    pub render_dpi: u32,
    pub rotation_degrees: u16,
    pub crop_left: f64,
    pub crop_bottom: f64,
    pub crop_right: f64,
    pub crop_top: f64,
    pub point_to_pixel_scale_x: f64,
    pub point_to_pixel_scale_y: f64,
    pub shard_element_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PdfOcrShardResultStatus {
    Succeeded,
    Failed,
    Skipped,
}

impl PdfOcrShardResultStatus {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfOcrShardResult {
    pub contract_version: String,
    pub source_path: String,
    pub source_content_hash: String,
    pub page_index: u32,
    pub image_path: String,
    pub image_mime_type: String,
    pub raster_sha256: String,
    pub render_profile: String,
    pub ocr_profile: String,
    pub status: PdfOcrShardResultStatus,
    pub text: Option<String>,
    pub text_mime_type: String,
    pub confidence: Option<f64>,
    pub error_message: Option<String>,
    pub shard_element_id: String,
    pub element_id: String,
}

impl PdfOcrShardResult {
    #[must_use]
    pub fn succeeded(input: &PdfOcrShardInput, text: impl Into<String>, confidence: f64) -> Self {
        Self::from_input(
            input,
            PdfOcrShardResultStatus::Succeeded,
            Some(text.into()),
            Some(confidence),
            None,
        )
    }

    #[must_use]
    pub fn failed(input: &PdfOcrShardInput, error_message: impl Into<String>) -> Self {
        Self::from_input(
            input,
            PdfOcrShardResultStatus::Failed,
            None,
            None,
            Some(error_message.into()),
        )
    }

    #[must_use]
    pub fn skipped(input: &PdfOcrShardInput, reason: impl Into<String>) -> Self {
        Self::from_input(
            input,
            PdfOcrShardResultStatus::Skipped,
            None,
            None,
            Some(reason.into()),
        )
    }

    fn from_input(
        input: &PdfOcrShardInput,
        status: PdfOcrShardResultStatus,
        text: Option<String>,
        confidence: Option<f64>,
        error_message: Option<String>,
    ) -> Self {
        Self {
            contract_version: PDF_OCR_SHARD_RESULT_SCHEMA_VERSION.to_string(),
            source_path: input.source_path.clone(),
            source_content_hash: input.source_content_hash.clone(),
            page_index: input.page_index,
            image_path: input.image_path.clone(),
            image_mime_type: input.image_mime_type.clone(),
            raster_sha256: input.raster_sha256.clone(),
            render_profile: input.render_profile.clone(),
            ocr_profile: input.ocr_profile.clone(),
            status,
            text,
            text_mime_type: "text/plain".to_string(),
            confidence,
            error_message,
            shard_element_id: input.shard_element_id.clone(),
            element_id: ocr_result_element_id(input),
        }
    }
}

#[must_use]
pub fn build_ocr_shard_inputs(
    manifests: &[PdfPageShardManifest],
    profile: &PdfOcrWorkerProfile,
) -> Vec<PdfOcrShardInput> {
    manifests
        .iter()
        .map(|manifest| PdfOcrShardInput {
            contract_version: PDF_OCR_SHARD_INPUT_SCHEMA_VERSION.to_string(),
            source_path: manifest.source_path.clone(),
            source_content_hash: manifest.source_content_hash.clone(),
            page_index: manifest.page_index,
            image_path: manifest.image_path.clone(),
            image_mime_type: manifest.image_mime_type.clone(),
            raster_sha256: manifest.raster_sha256.clone(),
            render_profile: manifest.render_profile.clone(),
            ocr_profile: profile.profile_id.clone(),
            ocr_engine: profile.engine.clone(),
            preferred_languages: profile.preferred_languages.clone(),
            min_confidence: profile.min_confidence,
            preserve_layout: profile.preserve_layout,
            raster_width_px: manifest.geometry.raster_width_px,
            raster_height_px: manifest.geometry.raster_height_px,
            render_dpi: manifest.geometry.render_dpi,
            rotation_degrees: manifest.geometry.rotation_degrees,
            crop_left: manifest.geometry.crop_box.left,
            crop_bottom: manifest.geometry.crop_box.bottom,
            crop_right: manifest.geometry.crop_box.right,
            crop_top: manifest.geometry.crop_box.top,
            point_to_pixel_scale_x: manifest.geometry.point_to_pixel_scale_x,
            point_to_pixel_scale_y: manifest.geometry.point_to_pixel_scale_y,
            shard_element_id: manifest.element_id.clone(),
        })
        .collect()
}

/// # Errors
///
/// Returns an error if Arrow cannot build the OCR worker input batch.
pub fn build_ocr_shard_input_batch(inputs: &[PdfOcrShardInput]) -> Result<RecordBatch, String> {
    RecordBatch::try_new(
        ocr_shard_input_schema(),
        vec![
            input_string_column(inputs, |input| input.contract_version.clone()),
            input_string_column(inputs, |input| input.source_path.clone()),
            input_string_column(inputs, |input| input.source_content_hash.clone()),
            input_int_column(inputs, |input| input.page_index),
            input_string_column(inputs, |input| input.image_path.clone()),
            input_string_column(inputs, |input| input.image_mime_type.clone()),
            input_string_column(inputs, |input| input.raster_sha256.clone()),
            input_string_column(inputs, |input| input.render_profile.clone()),
            input_string_column(inputs, |input| input.ocr_profile.clone()),
            input_string_column(inputs, |input| input.ocr_engine.clone()),
            input_string_column(inputs, |input| input.preferred_languages.join(",")),
            input_float_column(inputs, |input| input.min_confidence),
            Arc::new(BooleanArray::from(
                inputs
                    .iter()
                    .map(|input| input.preserve_layout)
                    .collect::<Vec<_>>(),
            )),
            input_int_column(inputs, |input| input.raster_width_px),
            input_int_column(inputs, |input| input.raster_height_px),
            input_int_column(inputs, |input| input.render_dpi),
            Arc::new(Int32Array::from(
                inputs
                    .iter()
                    .map(|input| i32::from(input.rotation_degrees))
                    .collect::<Vec<_>>(),
            )),
            input_float_column(inputs, |input| input.crop_left),
            input_float_column(inputs, |input| input.crop_bottom),
            input_float_column(inputs, |input| input.crop_right),
            input_float_column(inputs, |input| input.crop_top),
            input_float_column(inputs, |input| input.point_to_pixel_scale_x),
            input_float_column(inputs, |input| input.point_to_pixel_scale_y),
            input_string_column(inputs, |input| input.shard_element_id.clone()),
        ],
    )
    .map_err(|error| format!("build OCR shard input Arrow batch: {error}"))
}

/// # Errors
///
/// Returns an error if Arrow cannot build the OCR worker result batch.
pub fn build_ocr_shard_result_batch(results: &[PdfOcrShardResult]) -> Result<RecordBatch, String> {
    RecordBatch::try_new(
        ocr_shard_result_schema(),
        vec![
            result_string_column(results, |result| result.contract_version.clone()),
            result_string_column(results, |result| result.source_path.clone()),
            result_string_column(results, |result| result.source_content_hash.clone()),
            result_int_column(results, |result| result.page_index),
            result_string_column(results, |result| result.image_path.clone()),
            result_string_column(results, |result| result.image_mime_type.clone()),
            result_string_column(results, |result| result.raster_sha256.clone()),
            result_string_column(results, |result| result.render_profile.clone()),
            result_string_column(results, |result| result.ocr_profile.clone()),
            result_string_column(results, |result| result.status.as_str().to_string()),
            Arc::new(StringArray::from(
                results
                    .iter()
                    .map(|result| result.text.as_deref())
                    .collect::<Vec<_>>(),
            )),
            result_string_column(results, |result| result.text_mime_type.clone()),
            Arc::new(Float64Array::from(
                results
                    .iter()
                    .map(|result| result.confidence)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                results
                    .iter()
                    .map(|result| result.error_message.as_deref())
                    .collect::<Vec<_>>(),
            )),
            result_string_column(results, |result| result.shard_element_id.clone()),
            result_string_column(results, |result| result.element_id.clone()),
        ],
    )
    .map_err(|error| format!("build OCR shard result Arrow batch: {error}"))
}

/// # Errors
///
/// Returns an error if Arrow cannot build the stable document-resource batch.
pub fn build_ocr_result_resource_batch(
    results: &[PdfOcrShardResult],
) -> Result<RecordBatch, String> {
    RecordBatch::try_new(
        document_resource_schema(),
        vec![
            result_string_column(results, |result| result.source_path.clone()),
            result_string_column(results, |result| resource_type(result).to_string()),
            result_string_column(results, |result| result.image_path.clone()),
            result_int_column(results, |result| result.page_index),
            result_string_column(results, |result| {
                format!("OCR PDF page {}", result.page_index + 1)
            }),
            result_string_column(results, resource_content),
            result_string_column(results, |result| resource_mime_type(result).to_string()),
            result_string_column(results, |result| result.status.as_str().to_string()),
            result_string_column(results, |result| result.element_id.clone()),
        ],
    )
    .map_err(|error| format!("build OCR result resource Arrow batch: {error}"))
}

fn resource_type(result: &PdfOcrShardResult) -> &'static str {
    match result.status {
        PdfOcrShardResultStatus::Succeeded => "ocr_text",
        PdfOcrShardResultStatus::Failed => "ocr_error",
        PdfOcrShardResultStatus::Skipped => "ocr_skipped",
    }
}

fn resource_mime_type(result: &PdfOcrShardResult) -> &str {
    match result.status {
        PdfOcrShardResultStatus::Succeeded => result.text_mime_type.as_str(),
        PdfOcrShardResultStatus::Failed | PdfOcrShardResultStatus::Skipped => "text/plain",
    }
}

fn resource_content(result: &PdfOcrShardResult) -> String {
    match result.status {
        PdfOcrShardResultStatus::Succeeded => result.text.clone().unwrap_or_default(),
        PdfOcrShardResultStatus::Failed | PdfOcrShardResultStatus::Skipped => {
            result.error_message.clone().unwrap_or_default()
        }
    }
}

fn ocr_result_element_id(input: &PdfOcrShardInput) -> String {
    sha256_hex(
        format!(
            "{}:{}:{}:{}:{}",
            input.source_content_hash,
            input.page_index,
            input.render_profile,
            input.ocr_profile,
            input.raster_sha256
        )
        .as_bytes(),
    )
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn input_string_column<F>(inputs: &[PdfOcrShardInput], value: F) -> ArrayRef
where
    F: Fn(&PdfOcrShardInput) -> String,
{
    Arc::new(StringArray::from(
        inputs.iter().map(value).collect::<Vec<_>>(),
    ))
}

fn input_int_column<F>(inputs: &[PdfOcrShardInput], value: F) -> ArrayRef
where
    F: Fn(&PdfOcrShardInput) -> u32,
{
    Arc::new(Int32Array::from(
        inputs
            .iter()
            .map(|input| i32::try_from(value(input)).unwrap_or(i32::MAX))
            .collect::<Vec<_>>(),
    ))
}

fn input_float_column<F>(inputs: &[PdfOcrShardInput], value: F) -> ArrayRef
where
    F: Fn(&PdfOcrShardInput) -> f64,
{
    Arc::new(Float64Array::from(
        inputs.iter().map(value).collect::<Vec<_>>(),
    ))
}

fn result_string_column<F>(results: &[PdfOcrShardResult], value: F) -> ArrayRef
where
    F: Fn(&PdfOcrShardResult) -> String,
{
    Arc::new(StringArray::from(
        results.iter().map(value).collect::<Vec<_>>(),
    ))
}

fn result_int_column<F>(results: &[PdfOcrShardResult], value: F) -> ArrayRef
where
    F: Fn(&PdfOcrShardResult) -> u32,
{
    Arc::new(Int32Array::from(
        results
            .iter()
            .map(|result| i32::try_from(value(result)).unwrap_or(i32::MAX))
            .collect::<Vec<_>>(),
    ))
}

fn ocr_shard_input_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("contractVersion", DataType::Utf8, false),
        Field::new("sourcePath", DataType::Utf8, false),
        Field::new("sourceContentHash", DataType::Utf8, false),
        Field::new("pageIndex", DataType::Int32, false),
        Field::new("imagePath", DataType::Utf8, false),
        Field::new("imageMimeType", DataType::Utf8, false),
        Field::new("rasterSha256", DataType::Utf8, false),
        Field::new("renderProfile", DataType::Utf8, false),
        Field::new("ocrProfile", DataType::Utf8, false),
        Field::new("ocrEngine", DataType::Utf8, false),
        Field::new("preferredLanguages", DataType::Utf8, false),
        Field::new("minConfidence", DataType::Float64, false),
        Field::new("preserveLayout", DataType::Boolean, false),
        Field::new("rasterWidthPx", DataType::Int32, false),
        Field::new("rasterHeightPx", DataType::Int32, false),
        Field::new("renderDpi", DataType::Int32, false),
        Field::new("rotationDegrees", DataType::Int32, false),
        Field::new("cropLeft", DataType::Float64, false),
        Field::new("cropBottom", DataType::Float64, false),
        Field::new("cropRight", DataType::Float64, false),
        Field::new("cropTop", DataType::Float64, false),
        Field::new("pointToPixelScaleX", DataType::Float64, false),
        Field::new("pointToPixelScaleY", DataType::Float64, false),
        Field::new("shardElementId", DataType::Utf8, false),
    ]))
}

fn ocr_shard_result_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("contractVersion", DataType::Utf8, false),
        Field::new("sourcePath", DataType::Utf8, false),
        Field::new("sourceContentHash", DataType::Utf8, false),
        Field::new("pageIndex", DataType::Int32, false),
        Field::new("imagePath", DataType::Utf8, false),
        Field::new("imageMimeType", DataType::Utf8, false),
        Field::new("rasterSha256", DataType::Utf8, false),
        Field::new("renderProfile", DataType::Utf8, false),
        Field::new("ocrProfile", DataType::Utf8, false),
        Field::new("status", DataType::Utf8, false),
        Field::new("text", DataType::Utf8, true),
        Field::new("textMimeType", DataType::Utf8, false),
        Field::new("confidence", DataType::Float64, true),
        Field::new("errorMessage", DataType::Utf8, true),
        Field::new("shardElementId", DataType::Utf8, false),
        Field::new("elementId", DataType::Utf8, false),
    ]))
}

fn document_resource_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("sourcePath", DataType::Utf8, true),
        Field::new("resourceType", DataType::Utf8, true),
        Field::new("resourcePath", DataType::Utf8, true),
        Field::new("pageIndex", DataType::Int32, true),
        Field::new("caption", DataType::Utf8, true),
        Field::new("content", DataType::Utf8, true),
        Field::new("mimeType", DataType::Utf8, true),
        Field::new("status", DataType::Utf8, true),
        Field::new("elementId", DataType::Utf8, true),
    ]))
}

#[cfg(test)]
#[path = "../../tests/unit/pdf/ocr.rs"]
mod tests;
