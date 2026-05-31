//! OCR shard Arrow batch builders.

use std::sync::Arc;

use arrow::array::{BooleanArray, Float64Array, Int32Array, StringArray};
use arrow::record_batch::RecordBatch;

use super::support::{
    input_float_column, input_int_column, input_string_column, ocr_shard_input_contract,
    ocr_shard_result_contract, record_batch, result_int_column, result_string_column,
};
use crate::pdf::ocr::types::{
    OcrShardManifestSource, PDF_OCR_SHARD_INPUT_SCHEMA_VERSION, PdfOcrShardInput,
    PdfOcrShardResult, PdfOcrWorkerProfile,
};

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
