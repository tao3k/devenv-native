use std::sync::Arc;

use crate::studio::arrow_types::{
    LanceDataType, LanceField, LanceRecordBatch, LanceSchema, LanceStringArray, LanceUInt64Array,
};
use async_trait::async_trait;
use tonic::Status;
use xiuxian_wendao_server::transport::{
    VfsContentFlightRouteProvider, VfsContentFlightRouteResponse,
};

use crate::studio::types::VfsContentResponse;
use crate::studio::{StudioApiError, StudioState};

use super::read_content;

/// Studio-backed Flight provider for the semantic `/vfs/content` route.
#[derive(Clone)]
pub(crate) struct StudioVfsContentFlightRouteProvider {
    studio: Arc<StudioState>,
}

impl StudioVfsContentFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(studio: Arc<StudioState>) -> Self {
        Self { studio }
    }
}

impl std::fmt::Debug for StudioVfsContentFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioVfsContentFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl VfsContentFlightRouteProvider for StudioVfsContentFlightRouteProvider {
    async fn read_vfs_content_batch(
        &self,
        path: &str,
    ) -> Result<VfsContentFlightRouteResponse, Status> {
        load_vfs_content_flight_response(self.studio.as_ref(), path)
            .await
            .map_err(studio_api_error_to_tonic_status)
    }
}

pub(crate) async fn load_vfs_content_flight_response(
    studio: &StudioState,
    path: &str,
) -> Result<VfsContentFlightRouteResponse, StudioApiError> {
    let response = build_vfs_content_response(studio, path).await?;
    let batch = vfs_content_response_batch(&response).map_err(|error| {
        StudioApiError::internal(
            "VFS_CONTENT_FLIGHT_BATCH_FAILED",
            "Failed to materialize VFS content through the Flight-backed provider",
            Some(error),
        )
    })?;
    let app_metadata = vfs_content_response_flight_app_metadata(&response).map_err(|error| {
        StudioApiError::internal(
            "VFS_CONTENT_FLIGHT_METADATA_FAILED",
            "Failed to encode VFS content Flight app metadata",
            Some(error),
        )
    })?;
    Ok(VfsContentFlightRouteResponse::new(batch).with_app_metadata(app_metadata))
}

pub(crate) async fn build_vfs_content_response(
    studio: &StudioState,
    path: &str,
) -> Result<VfsContentResponse, StudioApiError> {
    let path = path.trim();
    if path.is_empty() {
        return Err(StudioApiError::bad_request(
            "MISSING_PATH",
            "`path` is required",
        ));
    }
    read_content(studio, path).await
}

pub(crate) fn vfs_content_response_batch(
    response: &VfsContentResponse,
) -> Result<LanceRecordBatch, String> {
    LanceRecordBatch::try_new(
        Arc::new(LanceSchema::new(vec![
            LanceField::new("path", LanceDataType::Utf8, false),
            LanceField::new("contentType", LanceDataType::Utf8, false),
            LanceField::new("content", LanceDataType::Utf8, false),
            LanceField::new("modified", LanceDataType::UInt64, false),
        ])),
        vec![
            Arc::new(LanceStringArray::from(vec![response.path.as_str()])),
            Arc::new(LanceStringArray::from(vec![response.content_type.as_str()])),
            Arc::new(LanceStringArray::from(vec![response.content.as_str()])),
            Arc::new(LanceUInt64Array::from(vec![response.modified])),
        ],
    )
    .map_err(|error| format!("failed to build VFS content Flight batch: {error}"))
}

pub(crate) fn vfs_content_response_flight_app_metadata(
    response: &VfsContentResponse,
) -> Result<Vec<u8>, String> {
    serde_json::to_vec(&serde_json::json!({
        "path": response.path,
        "contentType": response.content_type,
        "modified": response.modified,
    }))
    .map_err(|error| format!("failed to encode VFS content Flight app metadata: {error}"))
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
#[path = "../../../tests/unit/gateway/studio/vfs/flight_content.rs"]
mod tests;
