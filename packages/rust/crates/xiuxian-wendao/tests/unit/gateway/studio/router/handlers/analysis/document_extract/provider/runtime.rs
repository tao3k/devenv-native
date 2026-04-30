use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn document_extract_job_remains_queued_until_conversion_permit_is_available()
-> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("manual.pdf");
    fs::write(source.as_path(), b"pdf fixture").map_err(|error| error.to_string())?;
    let registry = DocumentExtractJobRegistry::new(
        temp.path().join("jobs.duckdb"),
        temp.path().join("artifacts"),
    )?;
    let provider = StudioDocumentExtractFlightRouteProvider::from_registry(Ok(registry), 1);
    let held_permit = Arc::clone(&provider.runtime.conversion_permits)
        .acquire_owned()
        .await
        .map_err(|error| error.to_string())?;
    let queued =
        provider
            .registry()?
            .submit(source.as_path(), temp.path().join("out").as_path(), false)?;
    let job_id = queued.job_id.clone();
    let running_provider = provider.clone();

    let handle = tokio::spawn(async move { running_provider.run_job(job_id.as_str()).await });
    sleep(Duration::from_millis(50)).await;
    let status = provider
        .status(queued.job_id.as_str())?
        .ok_or_else(|| "job should still exist".to_string())?;

    assert_eq!(status.status, "queued");
    assert_eq!(status.attempt_count, 0);

    handle.abort();
    drop(held_permit);
    Ok(())
}

#[tokio::test]
async fn document_extract_runtime_snapshot_reports_capacity_and_registry_counts()
-> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("manual.pdf");
    fs::write(source.as_path(), b"pdf fixture").map_err(|error| error.to_string())?;
    let registry = DocumentExtractJobRegistry::new(
        temp.path().join("jobs.duckdb"),
        temp.path().join("artifacts"),
    )?;
    let provider = StudioDocumentExtractFlightRouteProvider::from_registry(Ok(registry), 2);
    let _held_permit = Arc::clone(&provider.runtime.conversion_permits)
        .acquire_owned()
        .await
        .map_err(|error| error.to_string())?;
    let queued =
        provider
            .registry()?
            .submit(source.as_path(), temp.path().join("out").as_path(), false)?;
    provider.schedule_job(queued.job_id.clone()).await;

    let snapshot = provider.runtime_snapshot().await?;

    assert_eq!(snapshot.max_running_conversions, 2);
    assert_eq!(snapshot.available_conversion_permits, 1);
    assert_eq!(snapshot.in_process_running_conversions, 1);
    assert_eq!(snapshot.in_process_scheduled_jobs, 1);
    assert_eq!(snapshot.registry.queued_jobs, 1);
    assert_eq!(snapshot.registry.total_jobs, 1);
    Ok(())
}

#[tokio::test]
async fn sync_document_extract_reuses_succeeded_content_artifact() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("scan.png");
    fs::write(source.as_path(), b"image fixture").map_err(|error| error.to_string())?;
    let registry = DocumentExtractJobRegistry::new(
        temp.path().join("jobs.duckdb"),
        temp.path().join("artifacts"),
    )?;
    let provider = StudioDocumentExtractFlightRouteProvider::from_registry(Ok(registry), 1);
    let first_output = temp.path().join("first-output");
    fs::create_dir_all(first_output.as_path()).map_err(|error| error.to_string())?;
    let first_markdown = first_output.join("scan.md");
    fs::write(first_markdown.as_path(), "# Scan\n").map_err(|error| error.to_string())?;
    let batch = test_document_resource_batch(
        source.to_string_lossy().as_ref(),
        first_markdown.to_string_lossy().as_ref(),
    )?;
    write_arrow_file(
        first_output.join("_resources.arrow").as_path(),
        std::slice::from_ref(&batch),
    )?;
    fs::write(first_output.join("_complete.marker"), b"").map_err(|error| error.to_string())?;

    provider
        .persist_sync_output_artifact(source.as_path(), first_output.as_path())
        .await?;

    let second_output = temp.path().join("second-output");
    let response = provider
        .sync_document_extract_batch(
            source.to_string_lossy().as_ref(),
            second_output.to_string_lossy().as_ref(),
            false,
            false,
        )
        .await?;

    assert_eq!(response.batches.len(), 1);
    assert!(second_output.join("_resources.arrow").exists());
    let mirrored = read_arrow_file(second_output.join("_resources.arrow").as_path())?;
    let resource_paths = mirrored[0]
        .column_by_name("resourcePath")
        .ok_or_else(|| "missing resourcePath column".to_string())?
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()
        .ok_or_else(|| "resourcePath column is not Utf8".to_string())?;
    assert_eq!(
        resource_paths.value(0),
        second_output.join("scan.md").to_string_lossy()
    );
    Ok(())
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[tokio::test]
async fn document_extract_runtime_snapshot_reports_pdf_ocr_worker_capacity() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let registry = DocumentExtractJobRegistry::new(
        temp.path().join("jobs.duckdb"),
        temp.path().join("artifacts"),
    )?;
    let provider =
        StudioDocumentExtractFlightRouteProvider::from_registry_with_pdf_ocr_worker_limit(
            Ok(registry),
            2,
            5,
        );
    let _held_permits = provider
        .runtime
        .pdf_ocr_scheduler
        .permits_for_tests()
        .acquire_many_owned(2)
        .await
        .map_err(|error: tokio::sync::AcquireError| error.to_string())?;

    let snapshot = provider.runtime_snapshot().await?;

    assert_eq!(snapshot.max_pdf_ocr_workers, 5);
    assert_eq!(snapshot.current_pdf_ocr_worker_budget, 3);
    assert_eq!(snapshot.available_pdf_ocr_worker_permits, 3);
    assert_eq!(snapshot.in_process_pdf_ocr_workers, 2);
    assert_eq!(snapshot.in_flight_pdf_ocr_shards, 0);
    assert_eq!(snapshot.pdf_ocr_cache_hits, 0);
    assert_eq!(snapshot.pdf_ocr_cache_misses, 0);
    assert_eq!(snapshot.pdf_ocr_live_requests, 0);
    Ok(())
}

#[test]
fn document_extract_provider_reuses_runtime_for_same_project_root() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("create tempdir: {error}"));
    let first = shared_document_extract_provider_runtime(temp.path());
    let second = shared_document_extract_provider_runtime(temp.path());

    assert!(Arc::ptr_eq(&first, &second));
}
