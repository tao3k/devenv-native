use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use arrow::array::{Array, ArrayRef, Int32Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use xiuxian_wendao_attachments::pdf::ocr::{
    PDF_OCR_BACKEND_TEXT_PROFILE, PDF_OCR_DEFAULT_PROFILE, PDF_OCR_FAST_TEXT_PROFILE,
    PDF_OCR_HOSTED_VLM_DIRECT_PROFILE, PDF_OCR_SHARD_INPUT_SCHEMA_VERSION, PdfOcrShardInput,
    PdfOcrShardResult,
};
use xiuxian_wendao_attachments::pdf::profile::PdfSourcePageProfile;
use xiuxian_wendao_attachments::pdf::render::{
    PdfPageBox, PdfPageRegionRenderRequest, PdfPageRenderProfile, PdfPageRenderShardReport,
};

use super::{
    DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_PROFILE_ENV,
    DOCUMENT_EXTRACT_PDF_FAILED_PAGE_RECOVERY_ENV, DOCUMENT_RESOURCE_ARROW_CACHE_NAME,
    HybridPdfFailedPageRecoveryMode, OCR2_REGION_SCAFFOLD_FILE_NAME,
    Ocr2RegionMaterializationStats, Ocr2RegionPipelineBatchKind, cached_ocr2_region_render_report,
    contiguous_page_ranges, direct_docling_structure_recovery_page_range_enabled_with_lookup,
    direct_docling_structure_recovery_source_inputs_for_profiles,
    direct_docling_structure_recovery_source_inputs_for_profiles_with_lookup,
    docling_centered_structure_authority_page_count,
    docling_page_range_chunk_concurrency_limit_with_lookup,
    docling_page_range_chunk_concurrency_with_lookup, docling_page_range_chunk_plan_with_lookup,
    docling_page_range_chunk_size_for_pages_with_lookup,
    docling_page_range_chunk_size_for_planner_with_lookup,
    docling_page_range_chunk_size_with_lookup,
    docling_page_range_document_extract_endpoint_count_with_lookup,
    docling_page_range_fallback_page_indices,
    docling_page_range_fallback_plan_for_source_with_lookup,
    docling_page_range_fallback_profile_with_lookup, docling_page_range_fallback_ranges,
    docling_page_range_fallback_ranges_with_lookup, docling_page_range_hedge_delay_ms_with_lookup,
    docling_page_range_target_chunk_count, docling_structure_recovery_page_range_fallback_pages,
    failed_page_recovery_candidates, failed_page_recovery_input,
    failed_page_recovery_mode_with_lookup, has_ocr2_recovery_page_candidates,
    has_unhandled_non_success_result, hybrid_page_ocr_artifact_cache_key_for_test,
    hybrid_page_ocr_artifact_cache_response_for_test,
    hybrid_page_ocr_render_failure_requires_fail_fast_with_lookup,
    materialize_hybrid_page_ocr_resource_batch_from_results,
    normalize_docling_page_range_wrapper_rows,
    ocr2_region_render_ahead_limit_for_capacity_with_lookup, ocr2_region_render_cache_key,
    ocr2_region_render_cache_key_with_source_hash, ocr2_region_scaffold_payload,
    page_range_docling_fallback_chunk_summary, read_arrow_file,
    record_ocr_scheduler_or_docling_fallback_phase, record_ocr2_region_pipeline_batch_result,
    scheduled_inputs_without_docling_page_range_fallback_pages,
    store_hybrid_page_ocr_artifact_cache_for_test,
    structure_cost_budgeted_docling_page_range_fallback_ranges,
    structure_cost_budgeted_docling_page_range_fallback_ranges_with_limit,
    weighted_docling_page_range_fallback_ranges, write_arrow_file,
    write_ocr2_region_scaffold_sidecar_with_lookup,
};
use crate::studio::router::handlers::analysis::document_extract::provider::hybrid::profile::HybridPdfOcrProfilePlanner;
use crate::studio::router::handlers::analysis::document_extract::provider::hybrid::types::DOCUMENT_EXTRACT_PDF_HOSTED_VLM_SCAFFOLD_MODE_ENV;
use crate::studio::router::handlers::analysis::document_extract::provider::hybrid::types::{
    HybridDocumentResourceBatch, PageRangeDoclingFallbackChunkTiming,
    PageRangeDoclingFallbackSourceProfileSummary,
};

include!("scaffold.rs");
include!("failed_page.rs");
include!("docling_controls.rs");
include!("docling_fallback.rs");
include!("docling_source_inputs.rs");
include!("docling_chunks.rs");
include!("docling_structure_budget.rs");
include!("resource_rows.rs");
include!("region_cache_pipeline.rs");
include!("artifact_cache.rs");
include!("render_fail_fast.rs");

fn scaffold_enabled_lookup(key: &str) -> Option<String> {
    (key == DOCUMENT_EXTRACT_PDF_HOSTED_VLM_SCAFFOLD_MODE_ENV)
        .then(|| "region-table-json".to_string())
}

fn direct_docling_lookup(key: &str) -> Option<String> {
    (key == "WENDAO_DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER")
        .then(|| "docling-structure-recovery".to_string())
}

fn sample_region_request(region_index: u32) -> PdfPageRegionRenderRequest {
    sample_region_request_for_page(1, region_index)
}

fn sample_region_request_for_page(
    page_index: u32,
    region_index: u32,
) -> PdfPageRegionRenderRequest {
    PdfPageRegionRenderRequest::new(
        page_index,
        region_index,
        PdfPageBox::new(10.0, 20.0, 110.0, 220.0),
        Some(format!("{page_index:06}.{region_index:06}")),
    )
}

fn sample_render_report() -> PdfPageRenderShardReport {
    PdfPageRenderShardReport {
        source_path: "/tmp/source.pdf".to_string(),
        output_dir: "/tmp/out".to_string(),
        page_count: 1,
        shard_count: 2,
        manifest_arrow_path: None,
        ocr_input_arrow_path: None,
        pending_resource_arrow_path: None,
        render_profile: "pdfium-render-page-shards-v1".to_string(),
        render_selection: "region_shards".to_string(),
        status: "rendered".to_string(),
        routing_decision: "hybrid_page_ocr_candidate".to_string(),
        elapsed_ms: 0.0,
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

fn sample_document_resource_batch(rows: &[(&str, i32, &str, &str)]) -> Result<RecordBatch, String> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("sourcePath", DataType::Utf8, true),
        Field::new("resourceType", DataType::Utf8, true),
        Field::new("resourcePath", DataType::Utf8, true),
        Field::new("pageIndex", DataType::Int32, true),
        Field::new("caption", DataType::Utf8, true),
        Field::new("content", DataType::Utf8, true),
        Field::new("mimeType", DataType::Utf8, true),
        Field::new("status", DataType::Utf8, true),
        Field::new("elementId", DataType::Utf8, true),
    ]));
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|_| "/tmp/source.pdf"),
            )) as ArrayRef,
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|(resource_type, _, _, _)| *resource_type),
            )) as ArrayRef,
            Arc::new(StringArray::from_iter_values(
                rows.iter()
                    .map(|(resource_type, page_index, _, _)| {
                        format!("/tmp/{resource_type}-{page_index}.md")
                    })
                    .collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(Int32Array::from_iter_values(
                rows.iter().map(|(_, page_index, _, _)| *page_index),
            )) as ArrayRef,
            Arc::new(StringArray::from_iter_values(rows.iter().map(|_| ""))) as ArrayRef,
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|(_, _, content, _)| *content),
            )) as ArrayRef,
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|_| "text/markdown"),
            )) as ArrayRef,
            Arc::new(StringArray::from_iter_values(rows.iter().map(|_| "ok"))) as ArrayRef,
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|(_, _, _, element_id)| *element_id),
            )) as ArrayRef,
        ],
    )
    .map_err(|error| error.to_string())
}

fn test_string_value(batch: &RecordBatch, name: &str, row: usize) -> Result<String, String> {
    let column = batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing {name} column"))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| format!("{name} column is not utf8"))?;
    if column.is_null(row) {
        return Ok(String::new());
    }
    Ok(column.value(row).to_string())
}

fn sample_region_input() -> PdfOcrShardInput {
    PdfOcrShardInput {
        contract_version: PDF_OCR_SHARD_INPUT_SCHEMA_VERSION.to_string(),
        source_path: "/tmp/source.pdf".to_string(),
        source_content_hash: "sourcehash".to_string(),
        page_index: 1,
        image_path: "/tmp/out/_ocr2-region-renders/page-00001-region-00001.png".to_string(),
        image_mime_type: "image/png".to_string(),
        raster_sha256: "raster-1".to_string(),
        render_profile: "pdfium-render-page-shards-v1".to_string(),
        ocr_profile: PDF_OCR_HOSTED_VLM_DIRECT_PROFILE.to_string(),
        ocr_engine: "hosted-vlm-direct-ocr".to_string(),
        preferred_languages: vec!["auto".to_string()],
        min_confidence: 0.0,
        preserve_layout: true,
        raster_width_px: 1000,
        raster_height_px: 1000,
        render_dpi: 300,
        rotation_degrees: 0,
        crop_left: 10.0,
        crop_bottom: 20.0,
        crop_right: 110.0,
        crop_top: 220.0,
        point_to_pixel_scale_x: 3.0,
        point_to_pixel_scale_y: 3.0,
        shard_element_id: "region-shard".to_string(),
        shard_type: "region".to_string(),
        region_index: 1,
        parent_shard_element_id: "parent-shard".to_string(),
        reading_order_key: "000001.000050".to_string(),
        source_page_pixel_left: 0,
        source_page_pixel_top: 100,
        source_page_pixel_right: 1000,
        source_page_pixel_bottom: 900,
    }
}

fn sample_page_input(page_index: u32, ocr_profile: &str, ocr_engine: &str) -> PdfOcrShardInput {
    let mut input = sample_region_input();
    input.page_index = page_index;
    input.shard_type = "page".to_string();
    input.region_index = 0;
    input.parent_shard_element_id.clear();
    input.shard_element_id = format!("page-{page_index}");
    input.reading_order_key = format!("{page_index:06}");
    input.ocr_profile = ocr_profile.to_string();
    input.ocr_engine = ocr_engine.to_string();
    input
}
