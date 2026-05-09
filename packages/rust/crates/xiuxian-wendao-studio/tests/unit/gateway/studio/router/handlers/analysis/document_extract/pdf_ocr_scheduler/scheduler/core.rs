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
    assert_eq!(trace.queue_wait_ms, None);
    assert_eq!(trace.dispatch_start_ms, None);
    assert_eq!(trace.dispatch_end_ms, None);
    assert_eq!(trace.latency_ms, 1234.0);
    assert_eq!(trace.text_char_count, 15);
}
