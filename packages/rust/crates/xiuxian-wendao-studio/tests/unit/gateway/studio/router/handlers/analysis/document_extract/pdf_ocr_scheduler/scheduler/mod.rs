use std::sync::Arc;
use std::time::Duration;

use super::{
    DOCUMENT_EXTRACT_PDF_OCR_WORKERS_ENV, OcrSchedulerLane, PdfOcrWorkerScheduler,
    endpoint_index_for_request, local_backend_and_fast_text_results_for_tests,
    local_backend_text_error_fail_fast_results_for_tests, local_backend_text_results_for_tests,
    local_empty_backend_text_dispatch_python_results_for_tests,
    local_empty_backend_text_fail_fast_results_for_tests,
    local_partial_backend_text_error_fail_fast_results_for_tests, pdf_ocr_worker_limit_with_lookup,
    rendered_region_shard_chunks, rendered_region_shard_chunks_with_composite_size,
    scheduler_shard_groups, scheduler_trace_for_chunk,
    source_pdf_page_range_chunk_endpoint_index_with_lookup,
    source_pdf_page_range_chunk_prefers_first_endpoint_with_lookup, source_pdf_page_range_chunks,
    source_pdf_page_range_chunks_with_fast_text_split, source_pdf_page_range_chunks_with_weights,
    source_pdf_page_range_dispatch_budget,
    source_pdf_page_range_dispatch_budget_with_region_pipeline,
    source_pdf_page_range_dispatch_budget_with_region_pipeline_and_fast_text_split,
    source_pdf_page_range_dispatch_chunks,
};
use crate::studio::router::handlers::analysis::document_extract::pdf_ocr_cache::PdfOcrShardCache;
use xiuxian_wendao_attachments::pdf::ocr::{
    PDF_OCR_BACKEND_TEXT_PROFILE, PDF_OCR_FAST_TEXT_PROFILE, PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
    PDF_OCR_SHARD_INPUT_SCHEMA_VERSION, PdfOcrShardInput, PdfOcrShardResultStatus,
};

#[path = "../chunks/mod.rs"]
mod chunks;

include!("core.rs");
include!("local_text.rs");

fn sample_ocr_input(source_path: &str, page_index: u32, shard_type: &str) -> PdfOcrShardInput {
    PdfOcrShardInput {
        contract_version: PDF_OCR_SHARD_INPUT_SCHEMA_VERSION.to_string(),
        source_path: source_path.to_string(),
        source_content_hash: "sourcehash".to_string(),
        page_index,
        image_path: format!("/tmp/page-{page_index:05}.png"),
        image_mime_type: "image/png".to_string(),
        raster_sha256: format!("rasterhash-{page_index}"),
        render_profile: "pdfium-render-page-shards-v1".to_string(),
        ocr_profile: "docling-compatible-page-ocr-v1".to_string(),
        ocr_engine: "docling-compatible-ocr".to_string(),
        preferred_languages: vec!["auto".to_string()],
        min_confidence: 0.0,
        preserve_layout: true,
        raster_width_px: 2400,
        raster_height_px: 3100,
        render_dpi: 300,
        rotation_degrees: 0,
        crop_left: 0.0,
        crop_bottom: 0.0,
        crop_right: 612.0,
        crop_top: 792.0,
        point_to_pixel_scale_x: 3.921_568_627,
        point_to_pixel_scale_y: 3.914_141_414,
        shard_element_id: format!("shard-{page_index}"),
        shard_type: shard_type.to_string(),
        region_index: 0,
        parent_shard_element_id: String::new(),
        reading_order_key: format!("{page_index:06}.000000"),
        source_page_pixel_left: 0,
        source_page_pixel_top: 0,
        source_page_pixel_right: 2400,
        source_page_pixel_bottom: 3100,
    }
}

fn sample_source_page_range_input(source_path: &str, page_index: u32) -> PdfOcrShardInput {
    let mut input = sample_ocr_input(source_path, page_index, "page");
    input.image_path = format!("/tmp/source-page-range-{page_index:05}.source-page-range");
    input.image_mime_type = "application/x-wendao-source-pdf-page".to_string();
    input.render_profile = "source-pdf-page-range-shards-v1".to_string();
    input
}

fn autosearch_fixture() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../tests/fixtures/document-extract/milestones/autosearch-2604.17337.pdf")
}
