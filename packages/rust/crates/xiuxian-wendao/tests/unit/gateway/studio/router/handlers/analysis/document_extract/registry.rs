use super::*;

#[test]
fn document_extract_registry_deduplicates_same_content_job() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("manual.pdf");
    fs::write(source.as_path(), b"pdf fixture").map_err(|error| error.to_string())?;
    let registry = DocumentExtractJobRegistry::new(
        temp.path().join("jobs.duckdb"),
        temp.path().join("artifacts"),
    )?;
    let output = temp.path().join("out");

    let first = registry.submit(source.as_path(), output.as_path(), false)?;
    let second = registry.submit(source.as_path(), output.as_path(), false)?;

    assert_eq!(first.job_id, second.job_id);
    assert_eq!(first.status, "queued");
    assert_eq!(second.status, "queued");

    let running = registry
        .start_job(first.job_id.as_str())?
        .ok_or_else(|| "queued job should become running".to_string())?;
    assert_eq!(running.status, "running");
    assert_eq!(running.attempt_count, 1);
    assert!(
        registry.start_job(first.job_id.as_str())?.is_none(),
        "duplicate workers must not start the same job twice"
    );
    Ok(())
}

#[test]
fn document_extract_registry_recovers_stale_running_job_as_queued() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("manual.pdf");
    fs::write(source.as_path(), b"pdf fixture").map_err(|error| error.to_string())?;
    let job_db = temp.path().join("jobs.duckdb");
    let artifact_root = temp.path().join("artifacts");
    let registry = DocumentExtractJobRegistry::new(job_db.clone(), artifact_root.clone())?;
    let output = temp.path().join("out");
    let queued = registry.submit(source.as_path(), output.as_path(), false)?;
    let _ = registry.start_job(queued.job_id.as_str())?;

    let recovered = DocumentExtractJobRegistry::new(job_db, artifact_root)?;
    let status = recovered
        .status(queued.job_id.as_str())?
        .ok_or_else(|| "job should exist".to_string())?;

    assert_eq!(status.status, "queued");
    assert_eq!(status.attempt_count, 1);
    assert_eq!(status.started_at_ms, 0);
    Ok(())
}

#[test]
fn document_extract_registry_snapshot_counts_jobs_and_durations() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("manual.pdf");
    fs::write(source.as_path(), b"pdf fixture").map_err(|error| error.to_string())?;
    let registry = DocumentExtractJobRegistry::new(
        temp.path().join("jobs.duckdb"),
        temp.path().join("artifacts"),
    )?;
    let output = temp.path().join("out");
    let queued = registry.submit(source.as_path(), output.as_path(), false)?;
    let running = registry
        .start_job(queued.job_id.as_str())?
        .ok_or_else(|| "queued job should start".to_string())?;
    registry.mark_succeeded(running.job_id.as_str())?;

    let snapshot = registry.snapshot()?;

    assert_eq!(snapshot.total_jobs, 1);
    assert_eq!(snapshot.queued_jobs, 0);
    assert_eq!(snapshot.running_jobs, 0);
    assert_eq!(snapshot.succeeded_jobs, 1);
    assert_eq!(snapshot.failed_jobs, 0);
    assert_eq!(
        snapshot.last_finished_job_id.as_deref(),
        Some(running.job_id.as_str())
    );
    assert_eq!(snapshot.last_finished_status.as_deref(), Some("succeeded"));
    assert!(snapshot.last_conversion_duration_ms.is_some());
    assert!(snapshot.max_conversion_duration_ms.is_some());
    Ok(())
}

#[test]
fn document_extract_registry_finds_succeeded_output_dir_by_source() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("manual.pdf");
    fs::write(source.as_path(), b"pdf fixture").map_err(|error| error.to_string())?;
    let registry = DocumentExtractJobRegistry::new(
        temp.path().join("jobs.duckdb"),
        temp.path().join("artifacts"),
    )?;
    let output = temp.path().join("custom-output");
    let queued = registry.submit(source.as_path(), output.as_path(), false)?;
    let running = registry
        .start_job(queued.job_id.as_str())?
        .ok_or_else(|| "queued job should start".to_string())?;
    registry.mark_succeeded(running.job_id.as_str())?;

    let status = registry
        .latest_succeeded_status_for_source(source.as_path())?
        .ok_or_else(|| "succeeded source status should exist".to_string())?;

    assert_eq!(status.job_id, running.job_id);
    assert_eq!(status.output_dir, output.to_string_lossy());
    Ok(())
}
