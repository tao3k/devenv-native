use std::sync::Arc;
use std::time::Duration;

use super::{
    DOCUMENT_EXTRACT_PDF_OCR_WORKERS_ENV, OcrSchedulerLane, PdfOcrWorkerScheduler,
    endpoint_index_for_request, local_backend_and_fast_text_results_for_tests,
    local_backend_text_results_for_tests, pdf_ocr_worker_limit_with_lookup,
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
    PDF_OCR_SHARD_INPUT_SCHEMA_VERSION, PdfOcrShardInput,
};

mod chunks;

#[test]
fn pdf_ocr_worker_limit_defaults_to_available_parallelism() {
    let limit = pdf_ocr_worker_limit_with_lookup(&|_| None, Some(12));

    assert_eq!(limit, 12);
}

#[test]
fn pdf_ocr_worker_limit_accepts_deployment_upper_bound() {
    let limit = pdf_ocr_worker_limit_with_lookup(
        &|key| (key == DOCUMENT_EXTRACT_PDF_OCR_WORKERS_ENV).then(|| "6".to_string()),
        Some(12),
    );

    assert_eq!(limit, 6);
}

#[tokio::test]
async fn pdf_ocr_worker_permits_share_global_runtime_budget() -> Result<(), String> {
    let scheduler = PdfOcrWorkerScheduler::with_limit(3);
    let held_permits = Arc::clone(&scheduler.permits)
        .acquire_many_owned(2)
        .await
        .map_err(|error| error.to_string())?;

    let (permits, _) = scheduler.acquire_worker_permits(3).await?;

    assert_eq!(permits.len(), 1);
    assert_eq!(scheduler.available_permits(), 0);
    drop(permits);
    drop(held_permits);
    assert_eq!(scheduler.available_permits(), 3);
    Ok(())
}

#[tokio::test]
async fn pdf_ocr_worker_permits_take_requested_adaptive_budget() -> Result<(), String> {
    let scheduler = PdfOcrWorkerScheduler::with_limit(4);

    let (permits, _) = scheduler.acquire_worker_permits(2).await?;

    assert_eq!(permits.len(), 2);
    assert_eq!(scheduler.available_permits(), 2);
    Ok(())
}

#[test]
fn pdf_ocr_scheduler_endpoint_cursor_spans_independent_batches() -> Result<(), String> {
    let scheduler = PdfOcrWorkerScheduler::with_limit(4);

    assert_eq!(scheduler.endpoint_index_for_next_request(4)?, 0);
    assert_eq!(scheduler.endpoint_index_for_next_request(4)?, 1);
    assert_eq!(scheduler.endpoint_index_for_next_request(4)?, 2);
    assert_eq!(scheduler.endpoint_index_for_next_request(4)?, 3);
    assert_eq!(scheduler.endpoint_index_for_next_request(4)?, 0);
    assert!(scheduler.endpoint_index_for_next_request(0).is_err());
    Ok(())
}

#[test]
fn pdf_ocr_scheduler_snapshot_reports_adaptive_initial_budget() {
    let scheduler = PdfOcrWorkerScheduler::with_limit(9);

    let snapshot = scheduler.snapshot();

    assert_eq!(snapshot.max_worker_bound, 9);
    assert_eq!(snapshot.current_worker_budget, 3);
    assert_eq!(snapshot.available_worker_permits, 9);
    assert_eq!(snapshot.in_process_workers, 0);
}

#[tokio::test]
async fn pdf_ocr_scheduler_returns_full_cache_hits_without_python_endpoint() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let cache = PdfOcrShardCache::new(temp.path().to_path_buf());
    let inputs = vec![
        sample_ocr_input("/tmp/source.pdf", 0, "page"),
        sample_ocr_input("/tmp/source.pdf", 1, "page"),
    ];
    for input in &inputs {
        cache.store_successful(
            input,
            &xiuxian_wendao_attachments::pdf::ocr::PdfOcrShardResult::succeeded(
                input,
                format!("cached page {}", input.page_index),
                1.0,
            ),
        )?;
    }
    let scheduler = PdfOcrWorkerScheduler::with_limit_and_cache(2, cache);
    let endpoint_urls = vec!["http://127.0.0.1:9".to_string()];

    let response = scheduler
        .request_shards_with_endpoints(endpoint_urls.as_slice(), inputs.as_slice())
        .await?;

    assert_eq!(response.results.len(), 2);
    assert_eq!(response.results[0].text.as_deref(), Some("cached page 0"));
    assert_eq!(response.results[1].text.as_deref(), Some("cached page 1"));
    assert_eq!(response.resource_batch.num_rows(), 2);
    let snapshot = scheduler.snapshot();
    assert_eq!(snapshot.cache_hits, 2);
    assert_eq!(snapshot.cache_misses, 0);
    assert_eq!(snapshot.live_requests, 0);
    Ok(())
}

#[test]
fn pdf_ocr_scheduler_partitions_source_range_pages_from_direct_ocr2_regions() {
    let mut inputs = vec![
        sample_ocr_input("/tmp/source.pdf", 0, "page"),
        sample_ocr_input("/tmp/source.pdf", 1, "page"),
        sample_ocr_input("/tmp/source.pdf", 11, "region"),
        sample_ocr_input("/tmp/source.pdf", 12, "region"),
    ];
    for input in &mut inputs[2..] {
        input.ocr_profile = PDF_OCR_HOSTED_VLM_DIRECT_PROFILE.to_string();
        input.ocr_engine = "hosted-vlm-direct-ocr".to_string();
    }

    let groups = scheduler_shard_groups(inputs.as_slice());
    let positions = groups
        .iter()
        .map(|group| group.positions.clone())
        .collect::<Vec<_>>();

    assert_eq!(positions, vec![vec![0, 1], vec![2, 3]]);
}

#[test]
fn pdf_ocr_scheduler_trace_records_source_range_chunk_shape() {
    let inputs = vec![
        sample_ocr_input("/tmp/source.pdf", 3, "page"),
        sample_ocr_input("/tmp/source.pdf", 4, "page"),
    ];
    let results = vec![
        xiuxian_wendao_attachments::pdf::ocr::PdfOcrShardResult::succeeded(
            &inputs[0], "alpha", 1.0,
        ),
        xiuxian_wendao_attachments::pdf::ocr::PdfOcrShardResult::succeeded(
            &inputs[1],
            "beta gamma",
            1.0,
        ),
    ];

    let trace = scheduler_trace_for_chunk(
        OcrSchedulerLane::SourcePdfPageRange,
        inputs.as_slice(),
        results.as_slice(),
        Duration::from_millis(1234),
    );

    assert_eq!(trace.lane, "source-pdf-page-range");
    assert_eq!(trace.shard_count, 2);
    assert_eq!(trace.page_start, Some(3));
    assert_eq!(trace.page_end, Some(4));
    assert_eq!(trace.shard_type.as_deref(), Some("page"));
    assert_eq!(
        trace.ocr_profile.as_deref(),
        Some("docling-compatible-page-ocr-v1")
    );
    assert_eq!(trace.latency_ms, 1234.0);
    assert_eq!(trace.text_char_count, 15);
}

#[test]
fn pdf_ocr_scheduler_extracts_backend_text_locally_for_source_pages() {
    let mut inputs = vec![sample_ocr_input(
        autosearch_fixture().to_string_lossy().as_ref(),
        0,
        "page",
    )];
    inputs[0].ocr_profile = PDF_OCR_BACKEND_TEXT_PROFILE.to_string();
    inputs[0].ocr_engine = "docling-backend-text-ocr".to_string();

    let results = local_backend_text_results_for_tests(inputs.as_slice());

    let result = results[0]
        .as_ref()
        .expect("backend text page should be extracted locally");
    assert_eq!(result.ocr_profile, PDF_OCR_BACKEND_TEXT_PROFILE);
    assert_eq!(result.text_mime_type, "text/markdown");
    assert!(result.text.as_deref().unwrap_or_default().len() > 1_000);
}

#[test]
fn pdf_ocr_scheduler_can_extract_fast_text_locally_for_parent_pages() {
    let mut inputs = vec![sample_ocr_input(
        autosearch_fixture().to_string_lossy().as_ref(),
        5,
        "page",
    )];
    inputs[0].ocr_profile = PDF_OCR_FAST_TEXT_PROFILE.to_string();
    inputs[0].ocr_engine = "docling-fast-text-ocr".to_string();

    let backend_only = local_backend_text_results_for_tests(inputs.as_slice());
    assert!(
        backend_only[0].is_none(),
        "fast-text rows require the explicit local fast-text mode"
    );

    let results = local_backend_and_fast_text_results_for_tests(inputs.as_slice());

    let result = results[0]
        .as_ref()
        .expect("fast-text parent page should be extracted locally when enabled");
    assert_eq!(result.ocr_profile, PDF_OCR_FAST_TEXT_PROFILE);
    assert_eq!(result.text_mime_type, "text/markdown");
    assert!(result.text.as_deref().unwrap_or_default().len() > 1_000);
}

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

fn autosearch_fixture() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../tests/fixtures/document-extract/milestones/autosearch-2604.17337.pdf")
}
