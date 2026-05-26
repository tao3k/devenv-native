//! OCR shard Arrow batch builders and decoders.

use std::{collections::HashMap, sync::Arc};

use arrow::array::{Array, ArrayRef, BooleanArray, Float64Array, Int32Array, StringArray};
use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaDataType, ArrowSchemaNullabilityPolicy,
    ArrowSchemaValidationOptions, WENDAO_TABLE_METADATA_KEY, build_arrow_schema,
    validate_record_batch_schema_with_options,
};

use super::types::{
    OcrShardManifestSource, PDF_OCR_SHARD_INPUT_SCHEMA_VERSION,
    PDF_OCR_SHARD_RESULT_SCHEMA_VERSION, PdfOcrShardInput, PdfOcrShardResult,
    PdfOcrShardResultStatus, PdfOcrWorkerProfile,
};

const OCR_SHARD_INPUT_TABLE: &str = "pdf_ocr_shard_input";
const OCR_SHARD_RESULT_TABLE: &str = "pdf_ocr_shard_result";
const OCR_RESULT_RESOURCE_TABLE: &str = "pdf_ocr_result_resource";

/// Build OCR worker input rows from rendered shard manifests.
#[must_use]
pub fn build_ocr_shard_inputs(
    manifests: &[impl OcrShardManifestSource],
    profile: &PdfOcrWorkerProfile,
) -> Vec<PdfOcrShardInput> {
    manifests
        .iter()
        .map(|manifest| PdfOcrShardInput {
            contract_version: PDF_OCR_SHARD_INPUT_SCHEMA_VERSION.to_string(),
            source_path: manifest.source_path().to_string(),
            source_content_hash: manifest.source_content_hash().to_string(),
            page_index: manifest.page_index(),
            image_path: manifest.image_path().to_string(),
            image_mime_type: manifest.image_mime_type().to_string(),
            raster_sha256: manifest.raster_sha256().to_string(),
            render_profile: manifest.render_profile().to_string(),
            ocr_profile: profile.profile_id.clone(),
            ocr_engine: profile.engine.clone(),
            preferred_languages: profile.preferred_languages.clone(),
            min_confidence: profile.min_confidence,
            preserve_layout: profile.preserve_layout,
            raster_width_px: manifest.raster_width_px(),
            raster_height_px: manifest.raster_height_px(),
            render_dpi: manifest.render_dpi(),
            rotation_degrees: manifest.rotation_degrees(),
            crop_left: manifest.crop_left(),
            crop_bottom: manifest.crop_bottom(),
            crop_right: manifest.crop_right(),
            crop_top: manifest.crop_top(),
            point_to_pixel_scale_x: manifest.point_to_pixel_scale_x(),
            point_to_pixel_scale_y: manifest.point_to_pixel_scale_y(),
            shard_element_id: manifest.shard_element_id().to_string(),
            shard_type: manifest.shard_type().to_string(),
            region_index: manifest.region_index(),
            parent_shard_element_id: manifest.parent_shard_element_id().to_string(),
            reading_order_key: manifest.reading_order_key().to_string(),
            source_page_pixel_left: manifest.source_page_pixel_left(),
            source_page_pixel_top: manifest.source_page_pixel_top(),
            source_page_pixel_right: manifest.source_page_pixel_right(),
            source_page_pixel_bottom: manifest.source_page_pixel_bottom(),
        })
        .collect()
}

/// # Errors
///
/// Returns an error if Arrow cannot build the OCR worker input batch.
pub fn build_ocr_shard_input_batch(inputs: &[PdfOcrShardInput]) -> Result<RecordBatch, String> {
    record_batch(
        &ocr_shard_input_contract(),
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
            input_string_column(inputs, |input| input.shard_type.clone()),
            input_int_column(inputs, |input| input.region_index),
            input_string_column(inputs, |input| input.parent_shard_element_id.clone()),
            input_string_column(inputs, |input| input.reading_order_key.clone()),
            input_int_column(inputs, |input| input.source_page_pixel_left),
            input_int_column(inputs, |input| input.source_page_pixel_top),
            input_int_column(inputs, |input| input.source_page_pixel_right),
            input_int_column(inputs, |input| input.source_page_pixel_bottom),
        ],
        "build OCR shard input Arrow batch",
    )
}

/// # Errors
///
/// Returns an error if Arrow cannot build the OCR worker result batch.
pub fn build_ocr_shard_result_batch(results: &[PdfOcrShardResult]) -> Result<RecordBatch, String> {
    record_batch(
        &ocr_shard_result_contract(),
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
        "build OCR shard result Arrow batch",
    )
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
    validate_batch_schema(batch, &ocr_shard_input_contract(), "OCR shard input")?;
    let columns = OcrShardInputColumns::from_batch(batch)?;
    (0..batch.num_rows())
        .map(|row| columns.decode_row(row))
        .collect()
}

struct OcrShardInputColumns<'a> {
    contract_version: &'a StringArray,
    source_path: &'a StringArray,
    source_content_hash: &'a StringArray,
    page_index: &'a Int32Array,
    image_path: &'a StringArray,
    image_mime_type: &'a StringArray,
    raster_sha256: &'a StringArray,
    render_profile: &'a StringArray,
    ocr_profile: &'a StringArray,
    ocr_engine: &'a StringArray,
    preferred_languages: &'a StringArray,
    min_confidence: &'a Float64Array,
    preserve_layout: &'a BooleanArray,
    raster_width_px: &'a Int32Array,
    raster_height_px: &'a Int32Array,
    render_dpi: &'a Int32Array,
    rotation_degrees: &'a Int32Array,
    crop_left: &'a Float64Array,
    crop_bottom: &'a Float64Array,
    crop_right: &'a Float64Array,
    crop_top: &'a Float64Array,
    point_to_pixel_scale_x: &'a Float64Array,
    point_to_pixel_scale_y: &'a Float64Array,
    shard_element_id: &'a StringArray,
    shard_type: &'a StringArray,
    region_index: &'a Int32Array,
    parent_shard_element_id: &'a StringArray,
    reading_order_key: &'a StringArray,
    source_page_pixel_left: &'a Int32Array,
    source_page_pixel_top: &'a Int32Array,
    source_page_pixel_right: &'a Int32Array,
    source_page_pixel_bottom: &'a Int32Array,
}

impl<'a> OcrShardInputColumns<'a> {
    fn from_batch(batch: &'a RecordBatch) -> Result<Self, String> {
        Ok(Self {
            contract_version: string_column(batch, "contractVersion")?,
            source_path: string_column(batch, "sourcePath")?,
            source_content_hash: string_column(batch, "sourceContentHash")?,
            page_index: int32_column(batch, "pageIndex")?,
            image_path: string_column(batch, "imagePath")?,
            image_mime_type: string_column(batch, "imageMimeType")?,
            raster_sha256: string_column(batch, "rasterSha256")?,
            render_profile: string_column(batch, "renderProfile")?,
            ocr_profile: string_column(batch, "ocrProfile")?,
            ocr_engine: string_column(batch, "ocrEngine")?,
            preferred_languages: string_column(batch, "preferredLanguages")?,
            min_confidence: float64_column(batch, "minConfidence")?,
            preserve_layout: bool_column(batch, "preserveLayout")?,
            raster_width_px: int32_column(batch, "rasterWidthPx")?,
            raster_height_px: int32_column(batch, "rasterHeightPx")?,
            render_dpi: int32_column(batch, "renderDpi")?,
            rotation_degrees: int32_column(batch, "rotationDegrees")?,
            crop_left: float64_column(batch, "cropLeft")?,
            crop_bottom: float64_column(batch, "cropBottom")?,
            crop_right: float64_column(batch, "cropRight")?,
            crop_top: float64_column(batch, "cropTop")?,
            point_to_pixel_scale_x: float64_column(batch, "pointToPixelScaleX")?,
            point_to_pixel_scale_y: float64_column(batch, "pointToPixelScaleY")?,
            shard_element_id: string_column(batch, "shardElementId")?,
            shard_type: string_column(batch, "shardType")?,
            region_index: int32_column(batch, "regionIndex")?,
            parent_shard_element_id: string_column(batch, "parentShardElementId")?,
            reading_order_key: string_column(batch, "readingOrderKey")?,
            source_page_pixel_left: int32_column(batch, "sourcePagePixelLeft")?,
            source_page_pixel_top: int32_column(batch, "sourcePagePixelTop")?,
            source_page_pixel_right: int32_column(batch, "sourcePagePixelRight")?,
            source_page_pixel_bottom: int32_column(batch, "sourcePagePixelBottom")?,
        })
    }

    fn decode_row(&self, row: usize) -> Result<PdfOcrShardInput, String> {
        let version = required_string(self.contract_version, row, "contractVersion")?;
        if version != PDF_OCR_SHARD_INPUT_SCHEMA_VERSION {
            return Err(format!(
                "unexpected OCR shard input contract version `{version}`"
            ));
        }

        let languages = required_string(self.preferred_languages, row, "preferredLanguages")?
            .split(',')
            .filter_map(|value| {
                let trimmed = value.trim();
                (!trimmed.is_empty()).then(|| trimmed.to_string())
            })
            .collect();

        let shard_type_value = required_string(self.shard_type, row, "shardType")?;
        if !matches!(shard_type_value.as_str(), "page" | "region") {
            return Err(format!(
                "unsupported OCR shard input type `{shard_type_value}` at row {row}"
            ));
        }

        Ok(PdfOcrShardInput {
            contract_version: version,
            source_path: required_string(self.source_path, row, "sourcePath")?,
            source_content_hash: required_string(
                self.source_content_hash,
                row,
                "sourceContentHash",
            )?,
            page_index: required_u32(self.page_index, row, "pageIndex")?,
            image_path: required_string(self.image_path, row, "imagePath")?,
            image_mime_type: required_string(self.image_mime_type, row, "imageMimeType")?,
            raster_sha256: required_string(self.raster_sha256, row, "rasterSha256")?,
            render_profile: required_string(self.render_profile, row, "renderProfile")?,
            ocr_profile: required_string(self.ocr_profile, row, "ocrProfile")?,
            ocr_engine: required_string(self.ocr_engine, row, "ocrEngine")?,
            preferred_languages: languages,
            min_confidence: required_f64(self.min_confidence, row, "minConfidence")?,
            preserve_layout: required_bool(self.preserve_layout, row, "preserveLayout")?,
            raster_width_px: required_u32(self.raster_width_px, row, "rasterWidthPx")?,
            raster_height_px: required_u32(self.raster_height_px, row, "rasterHeightPx")?,
            render_dpi: required_u32(self.render_dpi, row, "renderDpi")?,
            rotation_degrees: required_u16(self.rotation_degrees, row, "rotationDegrees")?,
            crop_left: required_f64(self.crop_left, row, "cropLeft")?,
            crop_bottom: required_f64(self.crop_bottom, row, "cropBottom")?,
            crop_right: required_f64(self.crop_right, row, "cropRight")?,
            crop_top: required_f64(self.crop_top, row, "cropTop")?,
            point_to_pixel_scale_x: required_f64(
                self.point_to_pixel_scale_x,
                row,
                "pointToPixelScaleX",
            )?,
            point_to_pixel_scale_y: required_f64(
                self.point_to_pixel_scale_y,
                row,
                "pointToPixelScaleY",
            )?,
            shard_element_id: required_string(self.shard_element_id, row, "shardElementId")?,
            shard_type: shard_type_value,
            region_index: required_u32(self.region_index, row, "regionIndex")?,
            parent_shard_element_id: required_string(
                self.parent_shard_element_id,
                row,
                "parentShardElementId",
            )?,
            reading_order_key: required_string(self.reading_order_key, row, "readingOrderKey")?,
            source_page_pixel_left: required_u32(
                self.source_page_pixel_left,
                row,
                "sourcePagePixelLeft",
            )?,
            source_page_pixel_top: required_u32(
                self.source_page_pixel_top,
                row,
                "sourcePagePixelTop",
            )?,
            source_page_pixel_right: required_u32(
                self.source_page_pixel_right,
                row,
                "sourcePagePixelRight",
            )?,
            source_page_pixel_bottom: required_u32(
                self.source_page_pixel_bottom,
                row,
                "sourcePagePixelBottom",
            )?,
        })
    }
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
    validate_batch_schema(batch, &ocr_shard_result_contract(), "OCR shard result")?;

    let columns = OcrShardResultColumns::from_batch(batch)?;
    (0..batch.num_rows())
        .map(|row| decode_ocr_shard_result_row(&columns, row))
        .collect()
}

struct OcrShardResultColumns<'a> {
    contract_version: &'a StringArray,
    source_path: &'a StringArray,
    source_content_hash: &'a StringArray,
    page_index: &'a Int32Array,
    image_path: &'a StringArray,
    image_mime_type: &'a StringArray,
    raster_sha256: &'a StringArray,
    render_profile: &'a StringArray,
    ocr_profile: &'a StringArray,
    status: &'a StringArray,
    text: &'a StringArray,
    text_mime_type: &'a StringArray,
    confidence: &'a Float64Array,
    error_message: &'a StringArray,
    shard_element_id: &'a StringArray,
    element_id: &'a StringArray,
}

impl<'a> OcrShardResultColumns<'a> {
    fn from_batch(batch: &'a RecordBatch) -> Result<Self, String> {
        Ok(Self {
            contract_version: string_column(batch, "contractVersion")?,
            source_path: string_column(batch, "sourcePath")?,
            source_content_hash: string_column(batch, "sourceContentHash")?,
            page_index: int32_column(batch, "pageIndex")?,
            image_path: string_column(batch, "imagePath")?,
            image_mime_type: string_column(batch, "imageMimeType")?,
            raster_sha256: string_column(batch, "rasterSha256")?,
            render_profile: string_column(batch, "renderProfile")?,
            ocr_profile: string_column(batch, "ocrProfile")?,
            status: string_column(batch, "status")?,
            text: string_column(batch, "text")?,
            text_mime_type: string_column(batch, "textMimeType")?,
            confidence: float64_column(batch, "confidence")?,
            error_message: string_column(batch, "errorMessage")?,
            shard_element_id: string_column(batch, "shardElementId")?,
            element_id: string_column(batch, "elementId")?,
        })
    }
}

fn decode_ocr_shard_result_row(
    columns: &OcrShardResultColumns<'_>,
    row: usize,
) -> Result<PdfOcrShardResult, String> {
    let version = required_string(columns.contract_version, row, "contractVersion")?;
    if version != PDF_OCR_SHARD_RESULT_SCHEMA_VERSION {
        return Err(format!(
            "unexpected OCR shard result contract version `{version}`"
        ));
    }

    Ok(PdfOcrShardResult {
        contract_version: version,
        source_path: required_string(columns.source_path, row, "sourcePath")?,
        source_content_hash: required_string(
            columns.source_content_hash,
            row,
            "sourceContentHash",
        )?,
        page_index: required_u32(columns.page_index, row, "pageIndex")?,
        image_path: required_string(columns.image_path, row, "imagePath")?,
        image_mime_type: required_string(columns.image_mime_type, row, "imageMimeType")?,
        raster_sha256: required_string(columns.raster_sha256, row, "rasterSha256")?,
        render_profile: required_string(columns.render_profile, row, "renderProfile")?,
        ocr_profile: required_string(columns.ocr_profile, row, "ocrProfile")?,
        status: PdfOcrShardResultStatus::parse(
            required_string(columns.status, row, "status")?.as_str(),
        )?,
        text: optional_string(columns.text, row),
        text_mime_type: required_string(columns.text_mime_type, row, "textMimeType")?,
        confidence: optional_f64(columns.confidence, row),
        error_message: optional_string(columns.error_message, row),
        shard_element_id: required_string(columns.shard_element_id, row, "shardElementId")?,
        element_id: required_string(columns.element_id, row, "elementId")?,
    })
}

/// # Errors
///
/// Returns an error if Arrow cannot build the stable document-resource batch.
pub fn build_ocr_result_resource_batch(
    results: &[PdfOcrShardResult],
) -> Result<RecordBatch, String> {
    record_batch(
        &document_resource_contract(),
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
        "build OCR result resource Arrow batch",
    )
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

fn validate_batch_schema(
    batch: &RecordBatch,
    contract: &ArrowSchemaContract,
    label: &str,
) -> Result<(), String> {
    validate_record_batch_schema_with_options(batch, contract, exact_schema_options())
        .map_err(|error| format!("{label} schema validation: {error}"))
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

fn ocr_shard_input_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        OCR_SHARD_INPUT_TABLE,
        true,
        vec![
            utf8_contract_column("contractVersion"),
            utf8_contract_column("sourcePath"),
            utf8_contract_column("sourceContentHash"),
            int32_contract_column("pageIndex"),
            utf8_contract_column("imagePath"),
            utf8_contract_column("imageMimeType"),
            utf8_contract_column("rasterSha256"),
            utf8_contract_column("renderProfile"),
            utf8_contract_column("ocrProfile"),
            utf8_contract_column("ocrEngine"),
            utf8_contract_column("preferredLanguages"),
            float64_contract_column("minConfidence"),
            bool_contract_column("preserveLayout"),
            int32_contract_column("rasterWidthPx"),
            int32_contract_column("rasterHeightPx"),
            int32_contract_column("renderDpi"),
            int32_contract_column("rotationDegrees"),
            float64_contract_column("cropLeft"),
            float64_contract_column("cropBottom"),
            float64_contract_column("cropRight"),
            float64_contract_column("cropTop"),
            float64_contract_column("pointToPixelScaleX"),
            float64_contract_column("pointToPixelScaleY"),
            utf8_contract_column("shardElementId"),
            utf8_contract_column("shardType"),
            int32_contract_column("regionIndex"),
            utf8_contract_column("parentShardElementId"),
            utf8_contract_column("readingOrderKey"),
            int32_contract_column("sourcePagePixelLeft"),
            int32_contract_column("sourcePagePixelTop"),
            int32_contract_column("sourcePagePixelRight"),
            int32_contract_column("sourcePagePixelBottom"),
        ],
    )
}

fn ocr_shard_result_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        OCR_SHARD_RESULT_TABLE,
        true,
        vec![
            utf8_contract_column("contractVersion"),
            utf8_contract_column("sourcePath"),
            utf8_contract_column("sourceContentHash"),
            int32_contract_column("pageIndex"),
            utf8_contract_column("imagePath"),
            utf8_contract_column("imageMimeType"),
            utf8_contract_column("rasterSha256"),
            utf8_contract_column("renderProfile"),
            utf8_contract_column("ocrProfile"),
            utf8_contract_column("status"),
            nullable_utf8_contract_column("text"),
            utf8_contract_column("textMimeType"),
            nullable_float64_contract_column("confidence"),
            nullable_utf8_contract_column("errorMessage"),
            utf8_contract_column("shardElementId"),
            utf8_contract_column("elementId"),
        ],
    )
}

fn document_resource_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        OCR_RESULT_RESOURCE_TABLE,
        true,
        vec![
            nullable_utf8_contract_column("sourcePath"),
            nullable_utf8_contract_column("resourceType"),
            nullable_utf8_contract_column("resourcePath"),
            nullable_int32_contract_column("pageIndex"),
            nullable_utf8_contract_column("caption"),
            nullable_utf8_contract_column("content"),
            nullable_utf8_contract_column("mimeType"),
            nullable_utf8_contract_column("status"),
            nullable_utf8_contract_column("elementId"),
        ],
    )
}

fn record_batch(
    contract: &ArrowSchemaContract,
    columns: Vec<ArrayRef>,
    context: &'static str,
) -> Result<RecordBatch, String> {
    let batch = RecordBatch::try_new(schema_ref(contract), columns)
        .map_err(|error| format!("{context}: {error}"))?;
    validate_batch_schema(&batch, contract, context)?;
    Ok(batch)
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

const fn exact_schema_options() -> ArrowSchemaValidationOptions {
    ArrowSchemaValidationOptions::new().with_nullability_policy(ArrowSchemaNullabilityPolicy::Exact)
}

const fn utf8_contract_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Utf8)
}

const fn nullable_utf8_contract_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Utf8)
}

const fn int32_contract_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Int32)
}

const fn nullable_int32_contract_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Int32)
}

const fn float64_contract_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Float64)
}

const fn nullable_float64_contract_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Float64)
}

const fn bool_contract_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Boolean)
}
