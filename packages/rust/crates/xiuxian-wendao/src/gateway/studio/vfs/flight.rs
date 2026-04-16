use std::sync::Arc;

use async_trait::async_trait;
use tonic::Status;
use xiuxian_vector_store::{
    LanceDataType, LanceField, LanceInt32Array, LanceRecordBatch, LanceSchema, LanceStringArray,
};
use xiuxian_wendao_runtime::transport::{
    VfsResolveFlightRouteProvider, VfsResolveFlightRouteResponse,
};

use crate::gateway::studio::{StudioApiError, StudioState};
use crate::gateway::studio::types::StudioNavigationTarget;

use super::resolve_navigation_target;

/// Studio-backed Flight provider for the semantic `/vfs/resolve` route.
#[derive(Clone)]
pub(crate) struct StudioVfsResolveFlightRouteProvider {
    studio: Arc<StudioState>,
}

impl StudioVfsResolveFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(studio: Arc<StudioState>) -> Self {
        Self { studio }
    }
}

impl std::fmt::Debug for StudioVfsResolveFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioVfsResolveFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl VfsResolveFlightRouteProvider for StudioVfsResolveFlightRouteProvider {
    async fn resolve_vfs_navigation_batch(
        &self,
        path: &str,
    ) -> Result<VfsResolveFlightRouteResponse, Status> {
        load_vfs_resolve_flight_response(self.studio.as_ref(), path)
            .map_err(studio_api_error_to_tonic_status)
    }
}

pub(crate) fn build_vfs_resolve_response(
    studio: &StudioState,
    path: &str,
) -> Result<StudioNavigationTarget, StudioApiError> {
    let path = path.trim();
    if path.is_empty() {
        return Err(StudioApiError::bad_request(
            "MISSING_PATH",
            "`path` is required",
        ));
    }
    Ok(resolve_navigation_target(studio, path))
}

pub(crate) fn load_vfs_resolve_flight_response(
    studio: &StudioState,
    path: &str,
) -> Result<VfsResolveFlightRouteResponse, StudioApiError> {
    let response = build_vfs_resolve_response(studio, path)?;
    let batch = vfs_navigation_target_batch(&response).map_err(|error| {
        StudioApiError::internal(
            "VFS_RESOLVE_FLIGHT_BATCH_FAILED",
            "Failed to materialize VFS navigation target through the Flight-backed provider",
            Some(error),
        )
    })?;
    let app_metadata =
        vfs_resolve_response_flight_app_metadata(path, &response).map_err(|error| {
            StudioApiError::internal(
                "VFS_RESOLVE_FLIGHT_METADATA_FAILED",
                "Failed to encode VFS resolve Flight app metadata",
                Some(error),
            )
        })?;
    Ok(VfsResolveFlightRouteResponse::new(batch).with_app_metadata(app_metadata))
}

pub(crate) fn vfs_navigation_target_batch(
    target: &StudioNavigationTarget,
) -> Result<LanceRecordBatch, String> {
    LanceRecordBatch::try_new(
        Arc::new(LanceSchema::new(vec![
            LanceField::new("path", LanceDataType::Utf8, false),
            LanceField::new("category", LanceDataType::Utf8, false),
            LanceField::new("projectName", LanceDataType::Utf8, true),
            LanceField::new("rootLabel", LanceDataType::Utf8, true),
            LanceField::new("line", LanceDataType::Int32, true),
            LanceField::new("lineEnd", LanceDataType::Int32, true),
            LanceField::new("column", LanceDataType::Int32, true),
        ])),
        vec![
            Arc::new(LanceStringArray::from(vec![target.path.as_str()])),
            Arc::new(LanceStringArray::from(vec![target.category.as_str()])),
            Arc::new(LanceStringArray::from(vec![target.project_name.as_deref()])),
            Arc::new(LanceStringArray::from(vec![target.root_label.as_deref()])),
            Arc::new(LanceInt32Array::from(vec![
                target.line.map(line_to_i32).transpose()?,
            ])),
            Arc::new(LanceInt32Array::from(vec![
                target.line_end.map(line_to_i32).transpose()?,
            ])),
            Arc::new(LanceInt32Array::from(vec![
                target.column.map(line_to_i32).transpose()?,
            ])),
        ],
    )
    .map_err(|error| format!("failed to build VFS resolve Flight batch: {error}"))
}

pub(crate) fn vfs_resolve_response_flight_app_metadata(
    requested_path: &str,
    target: &StudioNavigationTarget,
) -> Result<Vec<u8>, String> {
    serde_json::to_vec(&serde_json::json!({
        "path": requested_path.trim(),
        "navigationTarget": target,
    }))
    .map_err(|error| format!("failed to encode VFS resolve Flight app metadata: {error}"))
}

fn line_to_i32(value: usize) -> Result<i32, String> {
    i32::try_from(value)
        .map_err(|error| format!("failed to represent VFS navigation position: {error}"))
}

fn studio_api_error_to_tonic_status(error: StudioApiError) -> Status {
    match error.status() {
        axum::http::StatusCode::BAD_REQUEST => Status::invalid_argument(error.error.message),
        axum::http::StatusCode::NOT_FOUND => Status::not_found(error.error.message),
        axum::http::StatusCode::CONFLICT => Status::failed_precondition(error.error.message),
        _ => Status::internal(error.error.message),
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/gateway/studio/vfs/flight.rs"]
mod tests;
