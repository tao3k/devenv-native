use std::path::PathBuf;

pub(super) const DOCUMENT_EXTRACT_SCHEMA_VERSION: &str = "v2";
pub(super) const DEFAULT_CONVERTER_PROFILE: &str = "default";

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
pub(in super::super) struct DocumentExtractJobRegistry {
    pub(super) job_db: PathBuf,
    pub(super) artifact_root: PathBuf,
    pub(super) converter_profile: String,
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

pub(super) struct DocumentExtractJobCounts {
    pub(super) total_jobs: usize,
    pub(super) queued_jobs: usize,
    pub(super) running_jobs: usize,
    pub(super) succeeded_jobs: usize,
    pub(super) failed_jobs: usize,
    pub(super) max_conversion_duration_ms: Option<i64>,
}

pub(super) struct LastFinishedDocumentExtractJob {
    pub(super) job_id: Option<String>,
    pub(super) status: Option<String>,
    pub(super) conversion_duration_ms: Option<i64>,
}
