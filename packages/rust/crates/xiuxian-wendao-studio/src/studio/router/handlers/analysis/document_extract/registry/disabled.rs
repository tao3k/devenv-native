use std::path::{Path, PathBuf};

use super::types::{
    DocumentExtractJobRegistry, DocumentExtractJobRegistrySnapshot, DocumentExtractJobStatus,
};

const DUCKDB_FEATURE_DISABLED: &str = "document extract job registry requires the `duckdb` feature";

impl DocumentExtractJobRegistry {
    pub(crate) fn new(_job_db: PathBuf, artifact_root: PathBuf) -> Result<Self, String> {
        Ok(Self { artifact_root })
    }

    pub(crate) fn default_for_project(project_root: &Path) -> Result<Self, String> {
        let data_root = std::env::var_os("PRJ_DATA_HOME")
            .map_or_else(|| project_root.join(".data"), PathBuf::from);
        Self::new(
            data_root.join("wendao.ai/document-extract/jobs.duckdb"),
            data_root.join("wendao.ai/document-extract/artifacts"),
        )
    }

    pub(crate) fn submit(
        &self,
        _source_path: &Path,
        _output_dir: &Path,
        _force: bool,
    ) -> Result<DocumentExtractJobStatus, String> {
        Err(disabled_error())
    }

    pub(crate) fn succeeded_status_for_source_content(
        &self,
        _source_path: &Path,
    ) -> Result<Option<DocumentExtractJobStatus>, String> {
        Ok(None)
    }

    pub(crate) fn artifact_dir_for_source_content(
        &self,
        source_path: &Path,
    ) -> Result<PathBuf, String> {
        Ok(self.artifact_root.join(
            source_path
                .file_name()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or("document"),
        ))
    }

    pub(crate) fn record_succeeded_output(
        &self,
        _source_path: &Path,
        _output_dir: &Path,
    ) -> Result<DocumentExtractJobStatus, String> {
        Err(disabled_error())
    }

    pub(crate) fn status(&self, _job_id: &str) -> Result<Option<DocumentExtractJobStatus>, String> {
        Ok(None)
    }

    pub(crate) fn latest_succeeded_status_for_source(
        &self,
        _source_path: &Path,
    ) -> Result<Option<DocumentExtractJobStatus>, String> {
        Ok(None)
    }

    pub(crate) fn snapshot(&self) -> Result<DocumentExtractJobRegistrySnapshot, String> {
        Ok(DocumentExtractJobRegistrySnapshot {
            total_jobs: 0,
            queued_jobs: 0,
            running_jobs: 0,
            succeeded_jobs: 0,
            failed_jobs: 0,
            last_finished_job_id: None,
            last_finished_status: None,
            last_conversion_duration_ms: None,
            max_conversion_duration_ms: None,
        })
    }

    pub(crate) fn start_job(
        &self,
        _job_id: &str,
    ) -> Result<Option<DocumentExtractJobStatus>, String> {
        Err(disabled_error())
    }

    pub(crate) fn mark_succeeded(&self, _job_id: &str) -> Result<(), String> {
        Err(disabled_error())
    }

    pub(crate) fn mark_failed(&self, _job_id: &str, _error_message: &str) -> Result<(), String> {
        Err(disabled_error())
    }
}

fn disabled_error() -> String {
    DUCKDB_FEATURE_DISABLED.to_string()
}
