use super::{
    Arc, DocumentExtractJobRegistry, StudioDocumentExtractFlightRouteProvider, fs,
    shared_document_extract_provider_runtime,
};

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
