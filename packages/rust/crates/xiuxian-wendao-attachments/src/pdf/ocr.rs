use std::sync::Arc;

use arrow::array::{Array, ArrayRef, BooleanArray, Float64Array, Int32Array, StringArray};
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

    /// Decode a stable OCR result status value.
    ///
    /// # Errors
    ///
    /// Returns an error when the status is outside the stable result contract.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "skipped" => Ok(Self::Skipped),
            other => Err(format!("unsupported OCR shard result status `{other}`")),
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

/// Decode stable OCR worker input rows from Arrow batches.
///
/// # Errors
///
/// Returns an error if any batch does not match the OCR shard input schema or
/// contains unsupported contract values.
pub fn decode_ocr_shard_input_batches(
    batches: &[RecordBatch],
) -> Result<Vec<PdfOcrShardInput>, String> {
    let mut inputs = Vec::new();
    for batch in batches {
        inputs.extend(decode_ocr_shard_input_batch(batch)?);
    }
    Ok(inputs)
}

/// Decode stable OCR worker input rows from one Arrow batch.
///
/// # Errors
///
/// Returns an error if the batch does not match the OCR shard input schema or
/// contains unsupported contract values.
pub fn decode_ocr_shard_input_batch(batch: &RecordBatch) -> Result<Vec<PdfOcrShardInput>, String> {
    validate_schema_compatible(
        batch.schema().as_ref(),
        ocr_shard_input_schema().as_ref(),
        "OCR shard input",
    )?;

    let contract_version = string_column(batch, "contractVersion")?;
    let source_path = string_column(batch, "sourcePath")?;
    let source_content_hash = string_column(batch, "sourceContentHash")?;
    let page_index = int32_column(batch, "pageIndex")?;
    let image_path = string_column(batch, "imagePath")?;
    let image_mime_type = string_column(batch, "imageMimeType")?;
    let raster_sha256 = string_column(batch, "rasterSha256")?;
    let render_profile = string_column(batch, "renderProfile")?;
    let ocr_profile = string_column(batch, "ocrProfile")?;
    let ocr_engine = string_column(batch, "ocrEngine")?;
    let preferred_languages = string_column(batch, "preferredLanguages")?;
    let min_confidence = float64_column(batch, "minConfidence")?;
    let preserve_layout = bool_column(batch, "preserveLayout")?;
    let raster_width_px = int32_column(batch, "rasterWidthPx")?;
    let raster_height_px = int32_column(batch, "rasterHeightPx")?;
    let render_dpi = int32_column(batch, "renderDpi")?;
    let rotation_degrees = int32_column(batch, "rotationDegrees")?;
    let crop_left = float64_column(batch, "cropLeft")?;
    let crop_bottom = float64_column(batch, "cropBottom")?;
    let crop_right = float64_column(batch, "cropRight")?;
    let crop_top = float64_column(batch, "cropTop")?;
    let point_to_pixel_scale_x = float64_column(batch, "pointToPixelScaleX")?;
    let point_to_pixel_scale_y = float64_column(batch, "pointToPixelScaleY")?;
    let shard_element_id = string_column(batch, "shardElementId")?;

    let mut inputs = Vec::with_capacity(batch.num_rows());
    for row in 0..batch.num_rows() {
        let version = required_string(contract_version, row, "contractVersion")?;
        if version != PDF_OCR_SHARD_INPUT_SCHEMA_VERSION {
            return Err(format!(
                "unexpected OCR shard input contract version `{version}`"
            ));
        }

        let languages = required_string(preferred_languages, row, "preferredLanguages")?
            .split(',')
            .filter_map(|value| {
                let trimmed = value.trim();
                (!trimmed.is_empty()).then(|| trimmed.to_string())
            })
            .collect();

        inputs.push(PdfOcrShardInput {
            contract_version: version,
            source_path: required_string(source_path, row, "sourcePath")?,
            source_content_hash: required_string(source_content_hash, row, "sourceContentHash")?,
            page_index: required_u32(page_index, row, "pageIndex")?,
            image_path: required_string(image_path, row, "imagePath")?,
            image_mime_type: required_string(image_mime_type, row, "imageMimeType")?,
            raster_sha256: required_string(raster_sha256, row, "rasterSha256")?,
            render_profile: required_string(render_profile, row, "renderProfile")?,
            ocr_profile: required_string(ocr_profile, row, "ocrProfile")?,
            ocr_engine: required_string(ocr_engine, row, "ocrEngine")?,
            preferred_languages: languages,
            min_confidence: required_f64(min_confidence, row, "minConfidence")?,
            preserve_layout: required_bool(preserve_layout, row, "preserveLayout")?,
            raster_width_px: required_u32(raster_width_px, row, "rasterWidthPx")?,
            raster_height_px: required_u32(raster_height_px, row, "rasterHeightPx")?,
            render_dpi: required_u32(render_dpi, row, "renderDpi")?,
            rotation_degrees: required_u16(rotation_degrees, row, "rotationDegrees")?,
            crop_left: required_f64(crop_left, row, "cropLeft")?,
            crop_bottom: required_f64(crop_bottom, row, "cropBottom")?,
            crop_right: required_f64(crop_right, row, "cropRight")?,
            crop_top: required_f64(crop_top, row, "cropTop")?,
            point_to_pixel_scale_x: required_f64(
                point_to_pixel_scale_x,
                row,
                "pointToPixelScaleX",
            )?,
            point_to_pixel_scale_y: required_f64(
                point_to_pixel_scale_y,
                row,
                "pointToPixelScaleY",
            )?,
            shard_element_id: required_string(shard_element_id, row, "shardElementId")?,
        });
    }
    Ok(inputs)
}

/// Decode stable OCR worker result rows from Arrow batches.
///
/// # Errors
///
/// Returns an error if any batch does not match the OCR shard result schema or
/// contains unsupported contract values.
pub fn decode_ocr_shard_result_batches(
    batches: &[RecordBatch],
) -> Result<Vec<PdfOcrShardResult>, String> {
    let mut results = Vec::new();
    for batch in batches {
        results.extend(decode_ocr_shard_result_batch(batch)?);
    }
    Ok(results)
}

/// Decode stable OCR worker result rows from one Arrow batch.
///
/// # Errors
///
/// Returns an error if the batch does not match the OCR shard result schema or
/// contains unsupported contract values.
pub fn decode_ocr_shard_result_batch(
    batch: &RecordBatch,
) -> Result<Vec<PdfOcrShardResult>, String> {
    validate_schema_compatible(
        batch.schema().as_ref(),
        ocr_shard_result_schema().as_ref(),
        "OCR shard result",
    )?;

    let contract_version = string_column(batch, "contractVersion")?;
    let source_path = string_column(batch, "sourcePath")?;
    let source_content_hash = string_column(batch, "sourceContentHash")?;
    let page_index = int32_column(batch, "pageIndex")?;
    let image_path = string_column(batch, "imagePath")?;
    let image_mime_type = string_column(batch, "imageMimeType")?;
    let raster_sha256 = string_column(batch, "rasterSha256")?;
    let render_profile = string_column(batch, "renderProfile")?;
    let ocr_profile = string_column(batch, "ocrProfile")?;
    let status = string_column(batch, "status")?;
    let text = string_column(batch, "text")?;
    let text_mime_type = string_column(batch, "textMimeType")?;
    let confidence = float64_column(batch, "confidence")?;
    let error_message = string_column(batch, "errorMessage")?;
    let shard_element_id = string_column(batch, "shardElementId")?;
    let element_id = string_column(batch, "elementId")?;

    let mut results = Vec::with_capacity(batch.num_rows());
    for row in 0..batch.num_rows() {
        let version = required_string(contract_version, row, "contractVersion")?;
        if version != PDF_OCR_SHARD_RESULT_SCHEMA_VERSION {
            return Err(format!(
                "unexpected OCR shard result contract version `{version}`"
            ));
        }

        results.push(PdfOcrShardResult {
            contract_version: version,
            source_path: required_string(source_path, row, "sourcePath")?,
            source_content_hash: required_string(source_content_hash, row, "sourceContentHash")?,
            page_index: required_u32(page_index, row, "pageIndex")?,
            image_path: required_string(image_path, row, "imagePath")?,
            image_mime_type: required_string(image_mime_type, row, "imageMimeType")?,
            raster_sha256: required_string(raster_sha256, row, "rasterSha256")?,
            render_profile: required_string(render_profile, row, "renderProfile")?,
            ocr_profile: required_string(ocr_profile, row, "ocrProfile")?,
            status: PdfOcrShardResultStatus::parse(
                required_string(status, row, "status")?.as_str(),
            )?,
            text: optional_string(text, row),
            text_mime_type: required_string(text_mime_type, row, "textMimeType")?,
            confidence: optional_f64(confidence, row),
            error_message: optional_string(error_message, row),
            shard_element_id: required_string(shard_element_id, row, "shardElementId")?,
            element_id: required_string(element_id, row, "elementId")?,
        });
    }
    Ok(results)
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

fn validate_schema_compatible(
    actual: &Schema,
    expected: &Schema,
    label: &str,
) -> Result<(), String> {
    if actual.fields().len() != expected.fields().len() {
        return Err(format!(
            "unexpected {label} column count: {}",
            actual.fields().len()
        ));
    }
    for (actual_field, expected_field) in actual.fields().iter().zip(expected.fields()) {
        if actual_field.name() != expected_field.name() {
            return Err(format!(
                "unexpected {label} column `{}`; expected `{}`",
                actual_field.name(),
                expected_field.name()
            ));
        }
        if actual_field.data_type() != expected_field.data_type() {
            return Err(format!(
                "unexpected {label} type for `{}`: {:?}",
                expected_field.name(),
                actual_field.data_type()
            ));
        }
    }
    Ok(())
}

fn string_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a StringArray, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing OCR shard result `{name}` column"))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| format!("OCR shard result `{name}` column is not Utf8"))
}

fn int32_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a Int32Array, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing OCR shard result `{name}` column"))?
        .as_any()
        .downcast_ref::<Int32Array>()
        .ok_or_else(|| format!("OCR shard result `{name}` column is not Int32"))
}

fn float64_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a Float64Array, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing OCR shard result `{name}` column"))?
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| format!("OCR shard result `{name}` column is not Float64"))
}

fn bool_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a BooleanArray, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing OCR shard result `{name}` column"))?
        .as_any()
        .downcast_ref::<BooleanArray>()
        .ok_or_else(|| format!("OCR shard result `{name}` column is not Boolean"))
}

fn required_string(column: &StringArray, row: usize, name: &str) -> Result<String, String> {
    if column.is_null(row) {
        return Err(format!("OCR shard result `{name}` is null at row {row}"));
    }
    Ok(column.value(row).to_string())
}

fn required_bool(column: &BooleanArray, row: usize, name: &str) -> Result<bool, String> {
    if column.is_null(row) {
        return Err(format!("OCR shard result `{name}` is null at row {row}"));
    }
    Ok(column.value(row))
}

fn optional_string(column: &StringArray, row: usize) -> Option<String> {
    (!column.is_null(row)).then(|| column.value(row).to_string())
}

fn required_u32(column: &Int32Array, row: usize, name: &str) -> Result<u32, String> {
    if column.is_null(row) {
        return Err(format!("OCR shard result `{name}` is null at row {row}"));
    }
    u32::try_from(column.value(row)).map_err(|_| {
        format!(
            "OCR shard result `{name}` must be non-negative at row {row}: {}",
            column.value(row)
        )
    })
}

fn required_u16(column: &Int32Array, row: usize, name: &str) -> Result<u16, String> {
    let value = required_u32(column, row, name)?;
    u16::try_from(value)
        .map_err(|_| format!("OCR shard result `{name}` must fit into u16 at row {row}: {value}"))
}

fn required_f64(column: &Float64Array, row: usize, name: &str) -> Result<f64, String> {
    if column.is_null(row) {
        return Err(format!("OCR shard result `{name}` is null at row {row}"));
    }
    Ok(column.value(row))
}

fn optional_f64(column: &Float64Array, row: usize) -> Option<f64> {
    (!column.is_null(row)).then(|| column.value(row))
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
