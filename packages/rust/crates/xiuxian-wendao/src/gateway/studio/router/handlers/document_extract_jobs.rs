//! REST endpoints for Rust-owned async document extraction jobs.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::Json;
use axum::extract::Query;
use serde::Deserialize;

use super::analysis::StudioDocumentExtractFlightRouteProvider;
use super::analysis::document_extract::{
    DocumentExtractJobStatus as RegistryDocumentExtractJobStatus, DocumentExtractRuntimeSnapshot,
};
use crate::gateway::studio::router::{GatewayState, StudioApiError};
use crate::gateway::studio::types::{
    DocumentExtractJobStatus, DocumentExtractJobSubmitRequest, DocumentExtractJobsStatus,
};
use crate::gateway::studio::vfs;

/// Query parameters for document extraction job status.
#[derive(Debug, Deserialize)]
pub struct DocumentExtractJobQuery {
    /// Stable document extraction job id.
    pub job_id: String,
}

/// Submit an async document extraction job.
///
/// # Errors
///
/// Returns `BAD_REQUEST` when `sourcePath` is blank, or `INTERNAL_SERVER_ERROR`
/// when the Rust-owned registry or Python worker path fails.
pub async fn submit_document_extract_job(
    axum::extract::State(state): axum::extract::State<Arc<GatewayState>>,
    Json(request): Json<DocumentExtractJobSubmitRequest>,
) -> Result<Json<DocumentExtractJobStatus>, StudioApiError> {
    let source_path = resolve_document_source_path(&state, request.source_path.as_str())?;
    let provider = StudioDocumentExtractFlightRouteProvider::new(state.as_ref());
    let status = provider
        .submit_document_extract_job(
            source_path.to_string_lossy().as_ref(),
            request.output_dir.as_deref(),
            request.force,
            request.wait_ms,
        )
        .await
        .map_err(|error| {
            StudioApiError::internal(
                "DOCUMENT_EXTRACT_JOB_SUBMIT_FAILED",
                "Failed to submit document extraction job",
                Some(error),
            )
        })?;
    Ok(Json(status.into()))
}

/// Return runtime capacity and registry counters for async document extraction.
///
/// # Errors
///
/// Returns `INTERNAL_SERVER_ERROR` when the Rust-owned registry cannot be read.
pub async fn get_document_extract_jobs_status(
    axum::extract::State(state): axum::extract::State<Arc<GatewayState>>,
) -> Result<Json<DocumentExtractJobsStatus>, StudioApiError> {
    let provider = StudioDocumentExtractFlightRouteProvider::new(state.as_ref());
    let snapshot = provider.runtime_snapshot().await.map_err(|error| {
        StudioApiError::internal(
            "DOCUMENT_EXTRACT_JOBS_STATUS_FAILED",
            "Failed to read document extraction runtime status",
            Some(error),
        )
    })?;
    Ok(Json(snapshot.into()))
}

/// Return one async document extraction job status.
///
/// # Errors
///
/// Returns `BAD_REQUEST` when `job_id` is blank, `NOT_FOUND` when the job id is
/// unknown, or `INTERNAL_SERVER_ERROR` when the registry cannot be read.
pub async fn get_document_extract_job(
    Query(query): Query<DocumentExtractJobQuery>,
    axum::extract::State(state): axum::extract::State<Arc<GatewayState>>,
) -> Result<Json<DocumentExtractJobStatus>, StudioApiError> {
    let job_id = query.job_id.trim();
    if job_id.is_empty() {
        return Err(StudioApiError::bad_request(
            "MISSING_JOB_ID",
            "`job_id` query parameter is required",
        ));
    }

    let provider = StudioDocumentExtractFlightRouteProvider::new(state.as_ref());
    let status = provider.status(job_id).map_err(|error| {
        StudioApiError::internal(
            "DOCUMENT_EXTRACT_JOB_STATUS_FAILED",
            "Failed to read document extraction job status",
            Some(error),
        )
    })?;
    status.map_or_else(
        || {
            Err(StudioApiError::not_found(format!(
                "Document extraction job not found: {job_id}"
            )))
        },
        |status| Ok(Json(status.into())),
    )
}

fn resolve_document_source_path(
    state: &GatewayState,
    source_path: &str,
) -> Result<PathBuf, StudioApiError> {
    let trimmed = source_path.trim();
    if trimmed.is_empty() {
        return Err(StudioApiError::bad_request(
            "MISSING_SOURCE_PATH",
            "`sourcePath` is required",
        ));
    }
    let raw_path = Path::new(trimmed);
    if raw_path.exists() {
        return Ok(raw_path.to_path_buf());
    }
    vfs::resolve_vfs_file_path(&state.studio, trimmed)
        .map_err(|_| StudioApiError::not_found(format!("Document not found: {trimmed}")))
}

impl From<RegistryDocumentExtractJobStatus> for DocumentExtractJobStatus {
    fn from(status: RegistryDocumentExtractJobStatus) -> Self {
        Self {
            job_id: status.job_id,
            source_path: status.source_path,
            output_dir: status.output_dir,
            content_hash: status.content_hash,
            status: status.status,
            attempt_count: status.attempt_count,
            created_at_ms: status.created_at_ms,
            started_at_ms: status.started_at_ms,
            finished_at_ms: status.finished_at_ms,
            error_message: status.error_message,
        }
    }
}

impl From<DocumentExtractRuntimeSnapshot> for DocumentExtractJobsStatus {
    fn from(snapshot: DocumentExtractRuntimeSnapshot) -> Self {
        Self {
            max_running_conversions: snapshot.max_running_conversions,
            available_conversion_permits: snapshot.available_conversion_permits,
            in_process_running_conversions: snapshot.in_process_running_conversions,
            in_process_scheduled_jobs: snapshot.in_process_scheduled_jobs,
            total_jobs: snapshot.registry.total_jobs,
            queued_jobs: snapshot.registry.queued_jobs,
            running_jobs: snapshot.registry.running_jobs,
            succeeded_jobs: snapshot.registry.succeeded_jobs,
            failed_jobs: snapshot.registry.failed_jobs,
            last_finished_job_id: snapshot.registry.last_finished_job_id,
            last_finished_status: snapshot.registry.last_finished_status,
            last_conversion_duration_ms: snapshot.registry.last_conversion_duration_ms,
            max_conversion_duration_ms: snapshot.registry.max_conversion_duration_ms,
        }
    }
}
