use std::fs;
use std::path::{Path, PathBuf};

use duckdb::Connection;
use xiuxian_db_store::state::{ArtisanStateRootConfig, artisan_state_root_from_config};

use super::types::{DEFAULT_CONVERTER_PROFILE, DocumentExtractJobRegistry};

impl DocumentExtractJobRegistry {
    pub(crate) fn new(job_db: PathBuf, artifact_root: PathBuf) -> Result<Self, String> {
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

    pub(crate) fn default_for_project(project_root: &Path) -> Result<Self, String> {
        let state_root = artisan_state_root_from_config(ArtisanStateRootConfig {
            project_root: Some(project_root.to_path_buf()),
            state_root: None,
            home_dir: None,
        });
        let job_db = std::env::var_os("WENDAO_DOCUMENT_EXTRACT_JOB_DB").map_or_else(
            || state_root.join("wendao-document-extract/jobs.duckdb"),
            PathBuf::from,
        );
        let artifact_root = std::env::var_os("WENDAO_DOCUMENT_EXTRACT_ARTIFACT_ROOT").map_or_else(
            || state_root.join("wendao-document-extract/artifacts"),
            PathBuf::from,
        );
        Self::new(job_db, artifact_root)
    }

    pub(super) fn connection(&self) -> Result<Connection, String> {
        Connection::open(self.job_db.as_path()).map_err(|error| {
            format!(
                "open document extract DuckDB registry `{}`: {error}",
                self.job_db.display()
            )
        })
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
}
