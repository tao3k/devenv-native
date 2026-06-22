use super::{
    Arc, DocumentExtractJobRegistry, Duration, StudioDocumentExtractFlightRouteProvider, fs, sleep,
};

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
