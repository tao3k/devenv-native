use std::path::Path;

use duckdb::{Connection, params};

use super::types::{
    DocumentExtractJobCounts, DocumentExtractJobStatus, LastFinishedDocumentExtractJob,
};

pub(super) fn fetch_status(
    conn: &Connection,
    job_id: &str,
) -> Result<Option<DocumentExtractJobStatus>, String> {
    let mut statement = conn
        .prepare(
            r"
            SELECT
                job_id, source_path, output_dir, artifact_dir, content_hash,
                status, attempt_count, created_at_ms, started_at_ms,
                finished_at_ms, error_message
            FROM document_extract_jobs
            WHERE job_id = ?
            ",
        )
        .map_err(|error| format!("prepare document extract status query: {error}"))?;
    let mut rows = statement
        .query(params![job_id])
        .map_err(|error| format!("query document extract status: {error}"))?;
    let Some(row) = rows
        .next()
        .map_err(|error| format!("read document extract status row: {error}"))?
    else {
        return Ok(None);
    };
    Ok(Some(DocumentExtractJobStatus {
        job_id: row
            .get(0)
            .map_err(|error| format!("read job_id: {error}"))?,
        source_path: row
            .get(1)
            .map_err(|error| format!("read source_path: {error}"))?,
        output_dir: row
            .get(2)
            .map_err(|error| format!("read output_dir: {error}"))?,
        artifact_dir: row
            .get(3)
            .map_err(|error| format!("read artifact_dir: {error}"))?,
        content_hash: row
            .get(4)
            .map_err(|error| format!("read content_hash: {error}"))?,
        status: row
            .get(5)
            .map_err(|error| format!("read status: {error}"))?,
        attempt_count: row
            .get(6)
            .map_err(|error| format!("read attempt_count: {error}"))?,
        created_at_ms: row
            .get(7)
            .map_err(|error| format!("read created_at_ms: {error}"))?,
        started_at_ms: row
            .get(8)
            .map_err(|error| format!("read started_at_ms: {error}"))?,
        finished_at_ms: row
            .get(9)
            .map_err(|error| format!("read finished_at_ms: {error}"))?,
        error_message: row
            .get(10)
            .map_err(|error| format!("read error_message: {error}"))?,
    }))
}

pub(super) fn fetch_latest_succeeded_status_for_source(
    conn: &Connection,
    source_path: &Path,
) -> Result<Option<DocumentExtractJobStatus>, String> {
    let mut statement = conn
        .prepare(
            r"
            SELECT job_id
            FROM document_extract_jobs
            WHERE source_path = ? AND status = 'succeeded'
            ORDER BY finished_at_ms DESC, created_at_ms DESC
            LIMIT 1
            ",
        )
        .map_err(|error| format!("prepare document extract source output-dir lookup: {error}"))?;
    let mut rows = statement
        .query(params![source_path.to_string_lossy().to_string()])
        .map_err(|error| format!("query document extract source output-dir lookup: {error}"))?;
    let Some(row) = rows
        .next()
        .map_err(|error| format!("read document extract source output-dir row: {error}"))?
    else {
        return Ok(None);
    };
    let job_id = row
        .get::<_, String>(0)
        .map_err(|error| format!("read source output-dir job_id: {error}"))?;
    drop(rows);
    drop(statement);
    fetch_status(conn, job_id.as_str())
}

pub(super) fn fetch_job_counts(conn: &Connection) -> Result<DocumentExtractJobCounts, String> {
    let mut statement = conn
        .prepare(
            r"
            SELECT
                COUNT(*),
                COALESCE(SUM(CASE WHEN status = 'queued' THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN status = 'running' THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN status = 'succeeded' THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END), 0),
                MAX(CASE
                    WHEN finished_at_ms > 0 AND started_at_ms > 0
                    THEN finished_at_ms - started_at_ms
                    ELSE NULL
                END)
            FROM document_extract_jobs
            ",
        )
        .map_err(|error| format!("prepare document extract job snapshot: {error}"))?;
    let mut rows = statement
        .query([])
        .map_err(|error| format!("query document extract job snapshot: {error}"))?;
    let Some(row) = rows
        .next()
        .map_err(|error| format!("read document extract job snapshot: {error}"))?
    else {
        return Ok(DocumentExtractJobCounts {
            total_jobs: 0,
            queued_jobs: 0,
            running_jobs: 0,
            succeeded_jobs: 0,
            failed_jobs: 0,
            max_conversion_duration_ms: None,
        });
    };

    Ok(DocumentExtractJobCounts {
        total_jobs: read_usize(row, 0, "total_jobs")?,
        queued_jobs: read_usize(row, 1, "queued_jobs")?,
        running_jobs: read_usize(row, 2, "running_jobs")?,
        succeeded_jobs: read_usize(row, 3, "succeeded_jobs")?,
        failed_jobs: read_usize(row, 4, "failed_jobs")?,
        max_conversion_duration_ms: row
            .get(5)
            .map_err(|error| format!("read max_conversion_duration_ms: {error}"))?,
    })
}

pub(super) fn fetch_last_finished_job(
    conn: &Connection,
) -> Result<LastFinishedDocumentExtractJob, String> {
    let mut statement = conn
        .prepare(
            r"
            SELECT job_id, status, finished_at_ms - started_at_ms
            FROM document_extract_jobs
            WHERE finished_at_ms > 0 AND started_at_ms > 0
            ORDER BY finished_at_ms DESC
            LIMIT 1
            ",
        )
        .map_err(|error| format!("prepare last finished document extract job query: {error}"))?;
    let mut rows = statement
        .query([])
        .map_err(|error| format!("query last finished document extract job: {error}"))?;
    let Some(row) = rows
        .next()
        .map_err(|error| format!("read last finished document extract job: {error}"))?
    else {
        return Ok(LastFinishedDocumentExtractJob {
            job_id: None,
            status: None,
            conversion_duration_ms: None,
        });
    };
    Ok(LastFinishedDocumentExtractJob {
        job_id: Some(
            row.get(0)
                .map_err(|error| format!("read last_finished_job_id: {error}"))?,
        ),
        status: Some(
            row.get(1)
                .map_err(|error| format!("read last_finished_status: {error}"))?,
        ),
        conversion_duration_ms: Some(
            row.get(2)
                .map_err(|error| format!("read last_conversion_duration_ms: {error}"))?,
        ),
    })
}

pub(super) fn lookup_content_hash(
    conn: &Connection,
    source_path: &Path,
    size_bytes: i64,
    mtime_ns: i64,
) -> Result<Option<String>, String> {
    let mut statement = conn
        .prepare(
            r"
            SELECT content_hash
            FROM document_extract_source_hashes
            WHERE source_path = ? AND size_bytes = ? AND mtime_ns = ?
            LIMIT 1
            ",
        )
        .map_err(|error| format!("prepare document extract content-hash lookup: {error}"))?;
    let mut rows = statement
        .query(params![
            source_path.to_string_lossy().to_string(),
            size_bytes,
            mtime_ns
        ])
        .map_err(|error| format!("query document extract content-hash lookup: {error}"))?;
    let Some(row) = rows
        .next()
        .map_err(|error| format!("read document extract content-hash row: {error}"))?
    else {
        return Ok(None);
    };
    row.get(0)
        .map(Some)
        .map_err(|error| format!("read document extract content hash: {error}"))
}

fn read_usize(row: &duckdb::Row<'_>, index: usize, label: &str) -> Result<usize, String> {
    let value = row
        .get::<_, i64>(index)
        .map_err(|error| format!("read {label}: {error}"))?;
    usize::try_from(value).map_err(|_| format!("{label} overflowed usize"))
}
