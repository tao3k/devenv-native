//! OCR shard Arrow batch decoders.

use arrow::array::{BooleanArray, Float64Array, Int32Array, StringArray};
use arrow::record_batch::RecordBatch;

use super::support::{
    bool_column, float64_column, int32_column, ocr_shard_input_contract, ocr_shard_result_contract,
    optional_f64, optional_string, required_bool, required_f64, required_string, required_u16,
    required_u32, string_column, validate_batch_schema,
};
use crate::pdf::ocr::types::{
    PDF_OCR_SHARD_INPUT_SCHEMA_VERSION, PDF_OCR_SHARD_RESULT_SCHEMA_VERSION, PdfOcrShardInput,
    PdfOcrShardResult, PdfOcrShardResultStatus,
};

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
