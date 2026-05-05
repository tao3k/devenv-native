use std::sync::Arc;

use super::{
    DOCUMENT_EXTRACT_PDF_OCR_WORKERS_ENV, PdfOcrWorkerScheduler, endpoint_index_for_request,
    pdf_ocr_worker_limit_with_lookup, source_pdf_page_range_chunks,
    source_pdf_page_range_chunks_with_weights,
};
use crate::studio::router::handlers::analysis::document_extract::pdf_ocr_cache::PdfOcrShardCache;
use xiuxian_wendao_attachments::pdf::ocr::{PDF_OCR_SHARD_INPUT_SCHEMA_VERSION, PdfOcrShardInput};

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
fn source_pdf_page_range_chunks_split_balanced_contiguous_ranges() {
    let inputs = (0..21)
        .map(|page_index| sample_ocr_input("/tmp/source.pdf", page_index, "page"))
        .collect::<Vec<_>>();

    let chunks = source_pdf_page_range_chunks(inputs.as_slice(), 4);

    assert_eq!(chunks.len(), 4);
    assert_eq!(chunks[0].len(), 6);
    assert_eq!(chunks[1].len(), 5);
    assert_eq!(chunks[2].len(), 5);
    assert_eq!(chunks[3].len(), 5);
    assert_eq!(chunks[0][0].page_index, 0);
    assert_eq!(chunks[0][5].page_index, 5);
    assert_eq!(chunks[1][0].page_index, 6);
    assert_eq!(chunks[3][4].page_index, 20);
}

#[test]
fn source_pdf_page_range_chunks_keep_single_range_for_one_permit() {
    let inputs = (0..3)
        .map(|page_index| sample_ocr_input("/tmp/source.pdf", page_index, "page"))
        .collect::<Vec<_>>();

    let chunks = source_pdf_page_range_chunks(inputs.as_slice(), 1);

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].len(), 3);
    assert_eq!(chunks[0][0].page_index, 0);
    assert_eq!(chunks[0][2].page_index, 2);
}

#[test]
fn source_pdf_page_range_chunks_do_not_cross_cache_miss_gaps() {
    let inputs = [0, 1, 4, 5, 8]
        .into_iter()
        .map(|page_index| sample_ocr_input("/tmp/source.pdf", page_index, "page"))
        .collect::<Vec<_>>();

    let chunks = source_pdf_page_range_chunks(inputs.as_slice(), 2);
    let page_runs = chunks
        .iter()
        .map(|chunk| {
            chunk
                .iter()
                .map(|input| input.page_index)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(page_runs, vec![vec![0, 1], vec![4, 5], vec![8]]);
}

#[test]
fn source_pdf_page_range_chunks_split_long_runs_without_crossing_gaps() {
    let inputs = (0..9)
        .chain(20..29)
        .map(|page_index| sample_ocr_input("/tmp/source.pdf", page_index, "page"))
        .collect::<Vec<_>>();

    let chunks = source_pdf_page_range_chunks(inputs.as_slice(), 4);

    assert_eq!(chunks.len(), 4);
    for chunk in chunks {
        for window in chunk.windows(2) {
            assert_eq!(window[1].page_index, window[0].page_index + 1);
        }
    }
}

#[test]
fn source_pdf_page_range_chunks_with_weights_preserve_order_and_isolate_heavy_pages() {
    let inputs = (0..9)
        .map(|page_index| sample_ocr_input("/tmp/source.pdf", page_index, "page"))
        .collect::<Vec<_>>();
    let weights = [1, 1, 1, 1, 20, 1, 1, 1, 1];

    let chunks = source_pdf_page_range_chunks_with_weights(inputs.as_slice(), 3, &weights);
    let page_runs = chunks
        .iter()
        .map(|chunk| {
            chunk
                .iter()
                .map(|input| input.page_index)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(page_runs, vec![vec![0, 1, 2, 3], vec![4], vec![5, 6, 7, 8]]);
}

#[test]
fn source_pdf_page_range_chunks_with_weights_do_not_cross_cache_miss_gaps() {
    let inputs = [0, 1, 4, 5, 8]
        .into_iter()
        .map(|page_index| sample_ocr_input("/tmp/source.pdf", page_index, "page"))
        .collect::<Vec<_>>();
    let weights = [1, 30, 1, 1, 1];

    let chunks = source_pdf_page_range_chunks_with_weights(inputs.as_slice(), 2, &weights);
    let page_runs = chunks
        .iter()
        .map(|chunk| {
            chunk
                .iter()
                .map(|input| input.page_index)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(page_runs, vec![vec![0, 1], vec![4, 5], vec![8]]);
}

#[test]
fn endpoint_index_for_request_round_robins_endpoint_pool() -> Result<(), String> {
    assert_eq!(endpoint_index_for_request(0, 3)?, 0);
    assert_eq!(endpoint_index_for_request(1, 3)?, 1);
    assert_eq!(endpoint_index_for_request(2, 3)?, 2);
    assert_eq!(endpoint_index_for_request(3, 3)?, 0);
    assert!(endpoint_index_for_request(0, 0).is_err());
    Ok(())
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
