use std::fs;

use tokio::time::{Duration, sleep};

#[cfg(any(
    feature = "document-extract-legacy-office",
    feature = "document-extract-pdf-source-range"
))]
use super::DOCUMENT_RESOURCE_ARROW_CACHE_NAME;
#[cfg(feature = "document-extract-legacy-office")]
use super::legacy_office::{is_legacy_office_source, write_legacy_office_document_extract_output};
#[cfg(feature = "document-extract-pdf-source-range")]
use super::validate_successful_ocr_results_for_inputs_with_lookup;
use super::{
    Arc, DOCUMENT_EXTRACT_ENDPOINT_ENV, DOCUMENT_EXTRACT_ENDPOINTS_ENV,
    DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS_ENV, DocumentExtractJobRegistry, EngineRecordBatch,
    ImageDocumentExtractRouteConfig, StudioDocumentExtractFlightRouteProvider,
    document_extract_batches_are_cacheable,
    document_extract_conversion_concurrency_limit_with_lookup,
    gateway_document_extract_mode_for_source, gateway_document_extract_profile_for_source,
    image_document_extract_model_route_with_config, read_arrow_file,
    shared_document_extract_provider_runtime, write_arrow_file,
};
#[cfg(all(
    feature = "document-extract-pdf-source-range",
    feature = "document-extract-pdf-render"
))]
use super::{
    DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_TOPUP_ENV,
    DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_MAX_SLICES_ENV,
    DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER_ENV,
    DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_TARGET_PIXELS_ENV,
    DOCUMENT_EXTRACT_PDF_HOSTED_VLM_RENDER_DPI_ENV, DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER_ENV,
    DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO_ENV, DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV,
    DOCUMENT_EXTRACT_PDF_RENDER_SELECTION_ENV, HybridPdfBackendTextTopup,
    HybridPdfOcr2RegionPlanner, HybridPdfOcrProfilePlanner, PdfPageRenderSelection,
    PdfPageRenderShardReport, PdfRenderRoutingDecision, PdfRenderStatus,
    apply_hybrid_page_docling_structure_recovery_profile_plan_for_profiles,
    apply_hybrid_page_hosted_vlm_backend_text_profile_plan_for_profiles,
    apply_hybrid_page_hosted_vlm_backend_text_profile_plan_for_profiles_with_lookup,
    apply_hybrid_page_hosted_vlm_profile_plan_for_profiles,
    apply_hybrid_page_ocr_profile_plan_for_profiles,
    automatic_ocr2_recovery_region_requests_for_profiles_with_lookup,
    automatic_ocr2_recovery_region_requests_with_lookup, hybrid_page_ocr_input_arrow_path,
    hybrid_page_ocr_profile_planner_with_lookup, hybrid_page_ocr_region_context_ratio_with_lookup,
    hybrid_page_ocr_region_requests_for_source_with_lookup,
    hybrid_page_ocr_render_selection_with_lookup, hybrid_page_ocr2_region_patch_sizing_with_lookup,
    hybrid_page_ocr2_region_planner_with_lookup, hybrid_pdf_backend_text_topup_with_lookup,
};
#[cfg(feature = "document-extract-pdf-source-range")]
use super::{
    HybridDocumentResourceBatch, PdfOcrShardInput, PdfOcrShardResult, PdfOcrShardResultStatus,
    build_document_structure_batch, hybrid_document_structure_blocks,
    validate_hybrid_page_coverage, validate_hybrid_precision_gate, validate_hybrid_shard_coverage,
    validate_ocr_results_match_inputs, validate_successful_ocr_results,
    write_hybrid_document_resource_artifacts,
};
#[cfg(all(
    feature = "document-extract-pdf-source-range",
    feature = "document-extract-pdf-render"
))]
use super::{
    has_ocr2_recovery_page_candidates, hybrid_page_ocr_render_profile_with_lookup,
    merge_ocr2_recovery_page_inputs,
};

#[cfg(feature = "document-extract-audio-shards")]
mod audio;
mod config;
mod image;
#[cfg(feature = "document-extract-legacy-office")]
mod legacy_office;
mod routing;
mod runtime;
mod structure;
mod transport;

#[cfg(all(
    feature = "document-extract-pdf-source-range",
    feature = "document-extract-pdf-render"
))]
fn sample_hybrid_page_ocr_report(
    status: PdfRenderStatus,
    routing_decision: PdfRenderRoutingDecision,
    page_count: u32,
    shard_count: u32,
    ocr_input_arrow_path: Option<&str>,
) -> PdfPageRenderShardReport {
    PdfPageRenderShardReport {
        source_path: "/tmp/source.pdf".to_string(),
        output_dir: "/tmp/out".to_string(),
        page_count,
        shard_count,
        manifest_arrow_path: None,
        ocr_input_arrow_path: ocr_input_arrow_path.map(ToString::to_string),
        pending_resource_arrow_path: None,
        render_profile: "pdfium-render-page-shards-v1".to_string(),
        render_selection: PdfPageRenderSelection::ShardFallbackPages
            .as_str()
            .to_string(),
        status: status.as_str().to_string(),
        routing_decision: routing_decision.as_str().to_string(),
        elapsed_ms: 1.0,
        error_message: None,
        artifact_cache_backend: None,
        artifact_cache_hit_count: 0,
        artifact_cache_miss_count: 0,
        artifact_cache_throttled_count: 0,
        artifact_cache_byte_count: 0,
        artifact_cache_page_raster_hit_count: 0,
        artifact_cache_page_raster_miss_count: 0,
        artifact_cache_page_raster_throttled_count: 0,
        artifact_cache_page_raster_byte_count: 0,
        artifact_cache_region_crop_hit_count: 0,
        artifact_cache_region_crop_miss_count: 0,
        artifact_cache_region_crop_throttled_count: 0,
        artifact_cache_region_crop_byte_count: 0,
        artifact_cache_region_manifest_projection_hit_count: 0,
        artifact_cache_region_manifest_projection_miss_count: 0,
        artifact_cache_region_manifest_projection_throttled_count: 0,
        artifact_cache_region_manifest_projection_byte_count: 0,
        artifact_cache_region_manifest_projection_row_hit_count: 0,
        artifact_cache_region_manifest_projection_row_miss_count: 0,
        artifact_cache_region_manifest_projection_row_throttled_count: 0,
        artifact_cache_region_manifest_projection_row_byte_count: 0,
    }
}

#[cfg(feature = "document-extract-pdf-source-range")]
fn sample_ocr_result(page_index: u32, succeeded: bool) -> PdfOcrShardResult {
    PdfOcrShardResult {
        contract_version: xiuxian_wendao_attachments::pdf::ocr::PDF_OCR_SHARD_RESULT_SCHEMA_VERSION
            .to_string(),
        source_path: "/tmp/source.pdf".to_string(),
        source_content_hash: "sourcehash".to_string(),
        page_index,
        image_path: format!("/tmp/out/page-{page_index}.png"),
        image_mime_type: "image/png".to_string(),
        raster_sha256: format!("raster-{page_index}"),
        render_profile: "pdfium-render-page-shards-v1".to_string(),
        ocr_profile: "docling-compatible-page-ocr-v1".to_string(),
        status: if succeeded {
            PdfOcrShardResultStatus::Succeeded
        } else {
            PdfOcrShardResultStatus::Skipped
        },
        text: succeeded.then(|| format!("OCR text {page_index}")),
        text_mime_type: "text/plain".to_string(),
        confidence: succeeded.then_some(0.99),
        error_message: (!succeeded).then(|| "worker skipped".to_string()),
        shard_element_id: format!("shard-{page_index}"),
        element_id: format!("ocr-{page_index}"),
    }
}

#[cfg(feature = "document-extract-pdf-source-range")]
fn sample_ocr_input(page_index: u32, shard_type: &str) -> PdfOcrShardInput {
    PdfOcrShardInput {
        contract_version: xiuxian_wendao_attachments::pdf::ocr::PDF_OCR_SHARD_INPUT_SCHEMA_VERSION
            .to_string(),
        source_path: "/tmp/source.pdf".to_string(),
        source_content_hash: "sourcehash".to_string(),
        page_index,
        image_path: format!("/tmp/out/page-{page_index}.png"),
        image_mime_type: "image/png".to_string(),
        raster_sha256: format!("raster-{page_index}"),
        render_profile: "pdfium-render-page-shards-v1".to_string(),
        ocr_profile: "docling-compatible-page-ocr-v1".to_string(),
        ocr_engine: "docling-compatible-ocr".to_string(),
        preferred_languages: vec!["auto".to_string()],
        min_confidence: 0.0,
        preserve_layout: true,
        raster_width_px: 1000,
        raster_height_px: 1000,
        render_dpi: 216,
        rotation_degrees: 0,
        crop_left: 0.0,
        crop_bottom: 0.0,
        crop_right: 612.0,
        crop_top: 792.0,
        point_to_pixel_scale_x: 3.0,
        point_to_pixel_scale_y: 3.0,
        shard_element_id: format!("shard-{page_index}"),
        shard_type: shard_type.to_string(),
        region_index: u32::from(shard_type == "region"),
        parent_shard_element_id: if shard_type == "region" {
            format!("page-shard-{page_index}")
        } else {
            String::new()
        },
        reading_order_key: format!("{page_index:06}.000001"),
        source_page_pixel_left: 0,
        source_page_pixel_top: 0,
        source_page_pixel_right: 1000,
        source_page_pixel_bottom: 1000,
    }
}

#[cfg(feature = "document-extract-pdf-source-range")]
fn test_resource_batch(rows: &[(&str, i32, &str)]) -> Result<EngineRecordBatch, String> {
    arrow::record_batch::RecordBatch::try_new(
        std::sync::Arc::new(arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("sourcePath", arrow::datatypes::DataType::Utf8, true),
            arrow::datatypes::Field::new("resourceType", arrow::datatypes::DataType::Utf8, true),
            arrow::datatypes::Field::new("resourcePath", arrow::datatypes::DataType::Utf8, true),
            arrow::datatypes::Field::new("pageIndex", arrow::datatypes::DataType::Int32, true),
            arrow::datatypes::Field::new("caption", arrow::datatypes::DataType::Utf8, true),
            arrow::datatypes::Field::new("content", arrow::datatypes::DataType::Utf8, true),
            arrow::datatypes::Field::new("mimeType", arrow::datatypes::DataType::Utf8, true),
            arrow::datatypes::Field::new("status", arrow::datatypes::DataType::Utf8, true),
            arrow::datatypes::Field::new("elementId", arrow::datatypes::DataType::Utf8, true),
        ])),
        vec![
            std::sync::Arc::new(arrow::array::StringArray::from(vec![
                "/tmp/source.pdf";
                rows.len()
            ])) as arrow::array::ArrayRef,
            std::sync::Arc::new(arrow::array::StringArray::from(
                rows.iter().map(|row| row.0).collect::<Vec<_>>(),
            )) as arrow::array::ArrayRef,
            std::sync::Arc::new(arrow::array::StringArray::from(vec![
                "/tmp/source.pdf";
                rows.len()
            ])) as arrow::array::ArrayRef,
            std::sync::Arc::new(arrow::array::Int32Array::from(
                rows.iter().map(|row| row.1).collect::<Vec<_>>(),
            )) as arrow::array::ArrayRef,
            std::sync::Arc::new(arrow::array::StringArray::from(vec![""; rows.len()]))
                as arrow::array::ArrayRef,
            std::sync::Arc::new(arrow::array::StringArray::from(vec!["content"; rows.len()]))
                as arrow::array::ArrayRef,
            std::sync::Arc::new(arrow::array::StringArray::from(vec![
                "text/markdown";
                rows.len()
            ])) as arrow::array::ArrayRef,
            std::sync::Arc::new(arrow::array::StringArray::from(vec!["ok"; rows.len()]))
                as arrow::array::ArrayRef,
            std::sync::Arc::new(arrow::array::StringArray::from(
                rows.iter().map(|row| row.2).collect::<Vec<_>>(),
            )) as arrow::array::ArrayRef,
        ],
    )
    .map_err(|error| error.to_string())
}

fn test_document_resource_batch(
    source_path: &str,
    resource_path: &str,
) -> Result<EngineRecordBatch, String> {
    arrow::record_batch::RecordBatch::try_new(
        std::sync::Arc::new(arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("sourcePath", arrow::datatypes::DataType::Utf8, true),
            arrow::datatypes::Field::new("resourceType", arrow::datatypes::DataType::Utf8, true),
            arrow::datatypes::Field::new("resourcePath", arrow::datatypes::DataType::Utf8, true),
            arrow::datatypes::Field::new("pageIndex", arrow::datatypes::DataType::Int32, true),
            arrow::datatypes::Field::new("caption", arrow::datatypes::DataType::Utf8, true),
            arrow::datatypes::Field::new("content", arrow::datatypes::DataType::Utf8, true),
            arrow::datatypes::Field::new("mimeType", arrow::datatypes::DataType::Utf8, true),
            arrow::datatypes::Field::new("status", arrow::datatypes::DataType::Utf8, true),
            arrow::datatypes::Field::new("elementId", arrow::datatypes::DataType::Utf8, true),
        ])),
        vec![
            std::sync::Arc::new(arrow::array::StringArray::from(vec![source_path]))
                as arrow::array::ArrayRef,
            std::sync::Arc::new(arrow::array::StringArray::from(vec!["document"]))
                as arrow::array::ArrayRef,
            std::sync::Arc::new(arrow::array::StringArray::from(vec![resource_path]))
                as arrow::array::ArrayRef,
            std::sync::Arc::new(arrow::array::Int32Array::from(vec![0])) as arrow::array::ArrayRef,
            std::sync::Arc::new(arrow::array::StringArray::from(vec![""]))
                as arrow::array::ArrayRef,
            std::sync::Arc::new(arrow::array::StringArray::from(vec!["content"]))
                as arrow::array::ArrayRef,
            std::sync::Arc::new(arrow::array::StringArray::from(vec!["text/markdown"]))
                as arrow::array::ArrayRef,
            std::sync::Arc::new(arrow::array::StringArray::from(vec!["ok"]))
                as arrow::array::ArrayRef,
            std::sync::Arc::new(arrow::array::StringArray::from(vec!["_main"]))
                as arrow::array::ArrayRef,
        ],
    )
    .map_err(|error| error.to_string())
}

#[cfg(feature = "document-extract-pdf-source-range")]
fn structure_string_column<'a>(
    batch: &'a EngineRecordBatch,
    name: &str,
) -> Result<&'a arrow::array::StringArray, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing structure `{name}` column"))?
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()
        .ok_or_else(|| format!("structure `{name}` column is not Utf8"))
}

#[cfg(feature = "document-extract-pdf-source-range")]
fn structure_float64_column<'a>(
    batch: &'a EngineRecordBatch,
    name: &str,
) -> Result<&'a arrow::array::Float64Array, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing structure `{name}` column"))?
        .as_any()
        .downcast_ref::<arrow::array::Float64Array>()
        .ok_or_else(|| format!("structure `{name}` column is not Float64"))
}

#[cfg(feature = "document-extract-pdf-source-range")]
fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 0.000_001,
        "expected {actual} to be close to {expected}"
    );
}
