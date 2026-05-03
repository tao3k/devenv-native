//! VFS endpoint handlers for Studio API.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path as AxumPath, Query, State},
    http::{HeaderValue, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};
use serde::Deserialize;

use crate::studio::router::{GatewayState, StudioApiError};
use crate::studio::types::{VfsContentResponse, VfsEntry, VfsScanResult};
use crate::studio::vfs;

/// Query parameters for VFS content retrieval.
#[derive(Debug, Deserialize)]
pub struct VfsCatQuery {
    /// The VFS path to retrieve.
    pub path: Option<String>,
}

/// Lists root VFS entries.
///
/// # Errors
///
/// This handler currently does not produce handler-local errors.
pub async fn root_entries(
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<Vec<VfsEntry>>, StudioApiError> {
    let entries = vfs::list_root_entries(&state.studio);
    Ok(Json(entries))
}

/// Scans all VFS roots.
///
/// # Errors
///
/// This handler currently does not produce handler-local errors.
pub async fn scan(
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<VfsScanResult>, StudioApiError> {
    let result = vfs::scan_roots(&state.studio);
    Ok(Json(result))
}

/// Gets a single VFS entry.
///
/// # Errors
///
/// Returns an error when the requested VFS entry does not exist or cannot be
/// resolved.
pub async fn entry(
    AxumPath(path): AxumPath<String>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<VfsEntry>, StudioApiError> {
    let entry = vfs::get_entry(&state.studio, path.as_str())?;
    Ok(Json(entry))
}

/// Reads file content.
///
/// # Errors
///
/// Returns an error when `path` is missing or when VFS content loading fails.
pub async fn cat(
    Query(query): Query<VfsCatQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<VfsContentResponse>, StudioApiError> {
    let path = query
        .path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| StudioApiError::bad_request("MISSING_PATH", "`path` is required"))?;
    let payload = vfs::read_content(&state.studio, path).await?;
    Ok(Json(payload))
}

/// Streams raw file bytes for multimodal preview surfaces.
///
/// # Errors
///
/// Returns an error when `path` is missing or when the VFS payload cannot be
/// resolved.
pub async fn raw(
    Query(query): Query<VfsCatQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Response, StudioApiError> {
    let path = query
        .path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| StudioApiError::bad_request("MISSING_PATH", "`path` is required"))?;
    let payload = vfs::read_raw_content(&state.studio, path).await?;
    let content_type = HeaderValue::from_str(payload.content_type.as_str()).map_err(|error| {
        StudioApiError::internal(
            "INVALID_CONTENT_TYPE",
            "Failed to render VFS raw content type header",
            Some(error.to_string()),
        )
    })?;

    Ok((
        StatusCode::OK,
        [(CONTENT_TYPE, content_type)],
        payload.content,
    )
        .into_response())
}
