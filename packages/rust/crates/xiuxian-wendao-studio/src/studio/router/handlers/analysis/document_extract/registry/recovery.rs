use duckdb::params;

use super::queries::fetch_status;
use super::types::DocumentExtractJobRegistry;
use super::utils::{artifact_ready, now_ms};

impl DocumentExtractJobRegistry {
    pub(super) fn recover_stale_running_jobs(&self) -> Result<(), String> {
        let conn = self.connection()?;
        let mut statement = conn
            .prepare("SELECT job_id FROM document_extract_jobs WHERE status = 'running'")
            .map_err(|error| format!("prepare stale document extract recovery: {error}"))?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|error| format!("query stale document extract jobs: {error}"))?;
        let job_ids = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("read stale document extract job id: {error}"))?;
        drop(statement);

        for job_id in job_ids {
            let Some(status) = fetch_status(&conn, job_id.as_str())? else {
                continue;
            };
            recover_stale_job(&conn, job_id.as_str(), &status)?;
        }
        Ok(())
    }
}

fn recover_stale_job(
    conn: &duckdb::Connection,
    job_id: &str,
    status: &super::types::DocumentExtractJobStatus,
) -> Result<(), String> {
    if artifact_ready(status) {
        conn.execute(
            "UPDATE document_extract_jobs SET status = 'succeeded', finished_at_ms = ? WHERE job_id = ?",
            params![now_ms(), job_id],
        )
        .map_err(|error| format!("recover succeeded document extract job: {error}"))?;
    } else if status.attempt_count >= 2 {
        conn.execute(
            "UPDATE document_extract_jobs SET status = 'failed', finished_at_ms = ?, error_message = 'stale running job exceeded retry limit' WHERE job_id = ?",
            params![now_ms(), job_id],
        )
        .map_err(|error| format!("recover failed document extract job: {error}"))?;
    } else {
        conn.execute(
            "UPDATE document_extract_jobs SET status = 'queued', started_at_ms = 0, finished_at_ms = 0, error_message = '' WHERE job_id = ?",
            params![job_id],
        )
        .map_err(|error| format!("recover queued document extract job: {error}"))?;
    }
    Ok(())
}
