use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::{Connection, params};
use sha2::{Digest, Sha256};

use super::arrow_cache::DOCUMENT_RESOURCE_ARROW_CACHE_NAME;

const DOCUMENT_EXTRACT_SCHEMA_VERSION: &str = "v2";
const DEFAULT_CONVERTER_PROFILE: &str = "default";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DocumentExtractJobStatus {
    pub(crate) job_id: String,
    pub(crate) source_path: String,
    pub(crate) output_dir: String,
    pub(crate) artifact_dir: String,
    pub(crate) content_hash: String,
    pub(crate) status: String,
    pub(crate) attempt_count: i32,
    pub(crate) created_at_ms: i64,
    pub(crate) started_at_ms: i64,
    pub(crate) finished_at_ms: i64,
    pub(crate) error_message: String,
}

#[derive(Debug)]
pub(super) struct DocumentExtractJobRegistry {
    job_db: PathBuf,
    artifact_root: PathBuf,
    converter_profile: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DocumentExtractJobRegistrySnapshot {
    pub(crate) total_jobs: usize,
    pub(crate) queued_jobs: usize,
    pub(crate) running_jobs: usize,
    pub(crate) succeeded_jobs: usize,
    pub(crate) failed_jobs: usize,
    pub(crate) last_finished_job_id: Option<String>,
    pub(crate) last_finished_status: Option<String>,
    pub(crate) last_conversion_duration_ms: Option<i64>,
    pub(crate) max_conversion_duration_ms: Option<i64>,
}

struct DocumentExtractJobCounts {
    total_jobs: usize,
    queued_jobs: usize,
    running_jobs: usize,
    succeeded_jobs: usize,
    failed_jobs: usize,
    max_conversion_duration_ms: Option<i64>,
}

struct LastFinishedDocumentExtractJob {
    job_id: Option<String>,
    status: Option<String>,
    conversion_duration_ms: Option<i64>,
}

impl DocumentExtractJobRegistry {
    pub(super) fn new(job_db: PathBuf, artifact_root: PathBuf) -> Result<Self, String> {
        if let Some(parent) = job_db.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "create document extract job registry directory `{}`: {error}",
                    parent.display()
                )
            })?;
        }
        fs::create_dir_all(artifact_root.as_path()).map_err(|error| {
            format!(
                "create document extract artifact root `{}`: {error}",
                artifact_root.display()
            )
        })?;
        let registry = Self {
            job_db,
            artifact_root,
            converter_profile: DEFAULT_CONVERTER_PROFILE.to_string(),
        };
        registry.init_schema()?;
        registry.recover_stale_running_jobs()?;
        Ok(registry)
    }

    pub(super) fn default_for_project(project_root: &Path) -> Result<Self, String> {
        let cache_root = std::env::var_os("PRJ_CACHE_HOME")
            .map_or_else(|| project_root.join(".cache"), PathBuf::from);
        let job_db = std::env::var_os("WENDAO_DOCUMENT_EXTRACT_JOB_DB").map_or_else(
            || cache_root.join("wendao-document-extract/jobs.duckdb"),
            PathBuf::from,
        );
        let artifact_root = std::env::var_os("WENDAO_DOCUMENT_EXTRACT_ARTIFACT_ROOT").map_or_else(
            || cache_root.join("wendao-document-extract/artifacts"),
            PathBuf::from,
        );
        Self::new(job_db, artifact_root)
    }

    pub(super) fn submit(
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

    pub(super) fn status(&self, job_id: &str) -> Result<Option<DocumentExtractJobStatus>, String> {
        let conn = self.connection()?;
        fetch_status(&conn, job_id)
    }

    pub(super) fn latest_succeeded_status_for_source(
        &self,
        source_path: &Path,
    ) -> Result<Option<DocumentExtractJobStatus>, String> {
        let conn = self.connection()?;
        fetch_latest_succeeded_status_for_source(&conn, source_path)
    }

    pub(super) fn snapshot(&self) -> Result<DocumentExtractJobRegistrySnapshot, String> {
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

    pub(super) fn start_job(
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

    pub(super) fn mark_succeeded(&self, job_id: &str) -> Result<(), String> {
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

    pub(super) fn mark_failed(&self, job_id: &str, error_message: &str) -> Result<(), String> {
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

    fn init_schema(&self) -> Result<(), String> {
        let conn = self.connection()?;
        conn.execute_batch(
            r"
            CREATE TABLE IF NOT EXISTS document_extract_source_hashes (
                source_path VARCHAR NOT NULL,
                size_bytes BIGINT NOT NULL,
                mtime_ns BIGINT NOT NULL,
                content_hash VARCHAR NOT NULL
            );
            CREATE TABLE IF NOT EXISTS document_extract_jobs (
                job_id VARCHAR NOT NULL,
                source_path VARCHAR NOT NULL,
                output_dir VARCHAR NOT NULL,
                artifact_dir VARCHAR NOT NULL,
                content_hash VARCHAR NOT NULL,
                source_suffix VARCHAR NOT NULL,
                converter_profile VARCHAR NOT NULL,
                status VARCHAR NOT NULL,
                attempt_count INTEGER NOT NULL,
                created_at_ms BIGINT NOT NULL,
                started_at_ms BIGINT NOT NULL,
                finished_at_ms BIGINT NOT NULL,
                error_message VARCHAR NOT NULL
            );
            CREATE UNIQUE INDEX IF NOT EXISTS idx_document_extract_jobs_job_id
            ON document_extract_jobs(job_id);
            ",
        )
        .map_err(|error| format!("initialize document extract job registry: {error}"))?;
        Ok(())
    }

    fn recover_stale_running_jobs(&self) -> Result<(), String> {
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
            if artifact_ready(&status) {
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
        }
        Ok(())
    }

    fn content_hash_for(&self, source_path: &Path) -> Result<String, String> {
        let metadata = source_path.metadata().map_err(|error| {
            format!(
                "read document extract source metadata `{}`: {error}",
                source_path.display()
            )
        })?;
        let size_bytes = i64::try_from(metadata.len()).unwrap_or(i64::MAX);
        let mtime_ns = metadata_modified_ns(&metadata)?;
        let conn = self.connection()?;
        if let Some(hash) = lookup_content_hash(&conn, source_path, size_bytes, mtime_ns)? {
            return Ok(hash);
        }

        let hash = streaming_sha256(source_path)?;
        conn.execute(
            r"
            INSERT INTO document_extract_source_hashes
            (source_path, size_bytes, mtime_ns, content_hash)
            VALUES (?, ?, ?, ?)
            ",
            params![
                source_path.to_string_lossy().to_string(),
                size_bytes,
                mtime_ns,
                hash
            ],
        )
        .map_err(|error| format!("cache document extract content hash: {error}"))?;
        Ok(hash)
    }

    fn job_id_for(&self, source_path: &Path, content_hash: &str) -> String {
        let suffix = source_path
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .map_or_else(String::new, |suffix| {
                format!(".{}", suffix.to_ascii_lowercase())
            });
        let key = format!(
            "{content_hash}|{suffix}|{DOCUMENT_EXTRACT_SCHEMA_VERSION}|{}",
            self.converter_profile
        );
        hex_sha256(key.as_bytes())
    }

    fn connection(&self) -> Result<Connection, String> {
        Connection::open(self.job_db.as_path()).map_err(|error| {
            format!(
                "open document extract DuckDB registry `{}`: {error}",
                self.job_db.display()
            )
        })
    }
}

impl DocumentExtractJobStatus {
    fn with_output_dir(&self, output_dir: &Path) -> Self {
        Self {
            output_dir: output_dir.to_string_lossy().to_string(),
            ..self.clone()
        }
    }
}

pub(crate) fn default_output_dir(source_path: &Path) -> PathBuf {
    let Some(extension) = source_path.extension().and_then(std::ffi::OsStr::to_str) else {
        return source_path.with_extension("extracted");
    };
    source_path.with_extension(format!("{extension}.extracted"))
}

pub(super) fn artifact_ready(status: &DocumentExtractJobStatus) -> bool {
    Path::new(status.artifact_dir.as_str())
        .join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME)
        .exists()
        && Path::new(status.artifact_dir.as_str())
            .join("_complete.marker")
            .exists()
}

fn fetch_status(
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

fn fetch_latest_succeeded_status_for_source(
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

fn fetch_job_counts(conn: &Connection) -> Result<DocumentExtractJobCounts, String> {
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

fn fetch_last_finished_job(conn: &Connection) -> Result<LastFinishedDocumentExtractJob, String> {
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

fn read_usize(row: &duckdb::Row<'_>, index: usize, label: &str) -> Result<usize, String> {
    let value = row
        .get::<_, i64>(index)
        .map_err(|error| format!("read {label}: {error}"))?;
    usize::try_from(value).map_err(|_| format!("{label} overflowed usize"))
}

fn lookup_content_hash(
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

fn metadata_modified_ns(metadata: &fs::Metadata) -> Result<i64, String> {
    let duration = metadata
        .modified()
        .map_err(|error| format!("read document extract source mtime: {error}"))?
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("document extract source mtime is before epoch: {error}"))?;
    i64::try_from(duration.as_nanos()).map_err(|_| "source mtime_ns overflowed i64".to_string())
}

fn streaming_sha256(source_path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(source_path).map_err(|error| {
        format!(
            "open document extract source for hashing `{}`: {error}",
            source_path.display()
        )
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| {
            format!(
                "read document extract source for hashing `{}`: {error}",
                source_path.display()
            )
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn hex_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn now_ms() -> i64 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    i64::try_from(duration.as_millis()).unwrap_or(i64::MAX)
}

#[cfg(test)]
#[path = "../../../../../../../tests/unit/gateway/studio/router/handlers/analysis/document_extract/registry.rs"]
mod tests;
