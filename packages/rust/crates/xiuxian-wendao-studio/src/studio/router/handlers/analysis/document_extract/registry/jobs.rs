use std::path::Path;

use duckdb::params;

use super::artifacts::{artifact_ready, now_ms};
use super::queries::{
    fetch_job_counts, fetch_last_finished_job, fetch_latest_succeeded_status_for_source,
    fetch_status,
};
use super::types::{
    DocumentExtractJobRegistry, DocumentExtractJobRegistrySnapshot, DocumentExtractJobStatus,
};

impl DocumentExtractJobRegistry {
    pub(crate) fn submit(
        &self,
        source_path: &Path,
        output_dir: &Path,
        force: bool,
    ) -> Result<DocumentExtractJobStatus, String> {
        let content_hash = self.content_hash_for(source_path)?;
        let job_id = self.job_id_for(source_path, content_hash.as_str());
        let artifact_dir = self.artifact_root.join(job_id.as_str());
        let conn = self.connection()?;
        if let Some(existing) = fetch_status(&conn, job_id.as_str())? {
            if matches!(existing.status.as_str(), "queued" | "running") {
                return Ok(existing.with_output_dir(output_dir));
            }
            if !force && existing.status == "succeeded" && artifact_ready(&existing) {
                return Ok(existing.with_output_dir(output_dir));
            }
            if !force && existing.status == "failed" {
                return Ok(existing.with_output_dir(output_dir));
            }
        }

        conn.execute(
            "DELETE FROM document_extract_jobs WHERE job_id = ?",
            params![job_id],
        )
        .map_err(|error| format!("replace document extract job row: {error}"))?;
        conn.execute(
            r"
            INSERT INTO document_extract_jobs (
                job_id, source_path, output_dir, artifact_dir, content_hash,
                source_suffix, converter_profile, status, attempt_count,
                created_at_ms, started_at_ms, finished_at_ms, error_message
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, 'queued', 0, ?, 0, 0, '')
            ",
            params![
                job_id,
                source_path.to_string_lossy().to_string(),
                output_dir.to_string_lossy().to_string(),
                artifact_dir.to_string_lossy().to_string(),
                content_hash,
                source_path
                    .extension()
                    .and_then(std::ffi::OsStr::to_str)
                    .map_or_else(String::new, |suffix| format!(".{suffix}")),
                self.converter_profile,
                now_ms(),
            ],
        )
        .map_err(|error| format!("insert document extract job row: {error}"))?;

        fetch_status(&conn, job_id.as_str())?
            .ok_or_else(|| format!("document extract job was not persisted: {job_id}"))
    }

    pub(crate) fn succeeded_status_for_source_content(
        &self,
        source_path: &Path,
    ) -> Result<Option<DocumentExtractJobStatus>, String> {
        let content_hash = self.content_hash_for(source_path)?;
        let job_id = self.job_id_for(source_path, content_hash.as_str());
        let conn = self.connection()?;
        let Some(status) = fetch_status(&conn, job_id.as_str())? else {
            return Ok(None);
        };
        if status.status == "succeeded" && artifact_ready(&status) {
            return Ok(Some(status));
        }
        Ok(None)
    }

    pub(crate) fn artifact_dir_for_source_content(
        &self,
        source_path: &Path,
    ) -> Result<std::path::PathBuf, String> {
        let content_hash = self.content_hash_for(source_path)?;
        let job_id = self.job_id_for(source_path, content_hash.as_str());
        Ok(self.artifact_root.join(job_id))
    }

    pub(crate) fn record_succeeded_output(
        &self,
        source_path: &Path,
        output_dir: &Path,
    ) -> Result<DocumentExtractJobStatus, String> {
        let content_hash = self.content_hash_for(source_path)?;
        let job_id = self.job_id_for(source_path, content_hash.as_str());
        let artifact_dir = self.artifact_root.join(job_id.as_str());
        let source_suffix = source_path
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .map_or_else(String::new, |suffix| format!(".{suffix}"));
        let timestamp_ms = now_ms();
        let conn = self.connection()?;
        conn.execute(
            "DELETE FROM document_extract_jobs WHERE job_id = ?",
            params![job_id.as_str()],
        )
        .map_err(|error| format!("replace sync document extract job row: {error}"))?;
        conn.execute(
            r"
            INSERT INTO document_extract_jobs (
                job_id, source_path, output_dir, artifact_dir, content_hash,
                source_suffix, converter_profile, status, attempt_count,
                created_at_ms, started_at_ms, finished_at_ms, error_message
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, 'succeeded', 1, ?, ?, ?, '')
            ",
            params![
                job_id,
                source_path.to_string_lossy().to_string(),
                output_dir.to_string_lossy().to_string(),
                artifact_dir.to_string_lossy().to_string(),
                content_hash,
                source_suffix,
                self.converter_profile,
                timestamp_ms,
                timestamp_ms,
                timestamp_ms,
            ],
        )
        .map_err(|error| format!("record sync document extract artifact: {error}"))?;

        fetch_status(&conn, job_id.as_str())?
            .ok_or_else(|| format!("sync document extract job was not persisted: {job_id}"))
    }

    pub(crate) fn status(&self, job_id: &str) -> Result<Option<DocumentExtractJobStatus>, String> {
        let conn = self.connection()?;
        fetch_status(&conn, job_id)
    }

    pub(crate) fn latest_succeeded_status_for_source(
        &self,
        source_path: &Path,
    ) -> Result<Option<DocumentExtractJobStatus>, String> {
        let conn = self.connection()?;
        fetch_latest_succeeded_status_for_source(&conn, source_path)
    }

    pub(crate) fn snapshot(&self) -> Result<DocumentExtractJobRegistrySnapshot, String> {
        let conn = self.connection()?;
        let counts = fetch_job_counts(&conn)?;
        let last_finished = fetch_last_finished_job(&conn)?;
        Ok(DocumentExtractJobRegistrySnapshot {
            total_jobs: counts.total_jobs,
            queued_jobs: counts.queued_jobs,
            running_jobs: counts.running_jobs,
            succeeded_jobs: counts.succeeded_jobs,
            failed_jobs: counts.failed_jobs,
            last_finished_job_id: last_finished.job_id,
            last_finished_status: last_finished.status,
            last_conversion_duration_ms: last_finished.conversion_duration_ms,
            max_conversion_duration_ms: counts.max_conversion_duration_ms,
        })
    }

    pub(crate) fn start_job(
        &self,
        job_id: &str,
    ) -> Result<Option<DocumentExtractJobStatus>, String> {
        let conn = self.connection()?;
        let updated = conn
            .execute(
                r"
                UPDATE document_extract_jobs
                SET status = 'running',
                    attempt_count = attempt_count + 1,
                    started_at_ms = ?,
                    finished_at_ms = 0,
                    error_message = ''
                WHERE job_id = ? AND status = 'queued' AND attempt_count < 2
                ",
                params![now_ms(), job_id],
            )
            .map_err(|error| format!("mark document extract job running: {error}"))?;
        if updated == 0 {
            return Ok(None);
        }
        fetch_status(&conn, job_id)
    }

    pub(crate) fn mark_succeeded(&self, job_id: &str) -> Result<(), String> {
        let conn = self.connection()?;
        conn.execute(
            r"
            UPDATE document_extract_jobs
            SET status = 'succeeded', finished_at_ms = ?, error_message = ''
            WHERE job_id = ?
            ",
            params![now_ms(), job_id],
        )
        .map_err(|error| format!("mark document extract job succeeded: {error}"))?;
        Ok(())
    }

    pub(crate) fn mark_failed(&self, job_id: &str, error_message: &str) -> Result<(), String> {
        let conn = self.connection()?;
        conn.execute(
            r"
            UPDATE document_extract_jobs
            SET status = 'failed', finished_at_ms = ?, error_message = ?
            WHERE job_id = ?
            ",
            params![now_ms(), error_message, job_id],
        )
        .map_err(|error| format!("mark document extract job failed: {error}"))?;
        Ok(())
    }
}
