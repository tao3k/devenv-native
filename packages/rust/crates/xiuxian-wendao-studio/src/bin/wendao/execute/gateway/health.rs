//! Gateway health endpoints.

use std::path::{Path, PathBuf};

use anyhow::Result;
use axum::{
    Json,
    http::{HeaderValue, StatusCode, header::HeaderName},
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::bin_support::wendao::execute::gateway::state::{
    GATEWAY_PIDFILE_ENV, GATEWAY_PROCESS_ID_HEADER,
};

/// Health check endpoint.
pub(crate) async fn health() -> Response {
    gateway_health_response(gateway_pidfile_from_env().as_deref())
}

#[cfg(feature = "zhenfa-router")]
const GATEWAY_FLIGHT_PLANE_STATUS: &str = "mounted";
#[cfg(not(feature = "zhenfa-router"))]
const GATEWAY_FLIGHT_PLANE_STATUS: &str = "disabled";

pub(crate) fn gateway_pidfile_from_env() -> Option<PathBuf> {
    std::env::var_os(GATEWAY_PIDFILE_ENV)
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
}

pub(crate) fn gateway_health_response(pidfile: Option<&Path>) -> Response {
    let process_id = std::process::id();
    let mut response = match pidfile {
        Some(pidfile) => match gateway_pidfile_process_id(pidfile) {
            Ok(expected_process_id) if expected_process_id == process_id => Json(json!({
                "status": "ok",
                "ready": true,
                "service": "wendao-gateway",
                "processId": process_id,
                "planes": {
                    "http": "ready",
                    "flight": GATEWAY_FLIGHT_PLANE_STATUS,
                },
            }))
            .into_response(),
            Ok(expected_process_id) => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "degraded",
                    "ready": false,
                    "service": "wendao-gateway",
                    "error": "gateway is not ready",
                    "pidfile": pidfile.display().to_string(),
                    "expectedPid": expected_process_id,
                    "processId": process_id,
                    "planes": {
                        "http": "ready",
                        "flight": GATEWAY_FLIGHT_PLANE_STATUS,
                    },
                })),
            )
                .into_response(),
            Err(details) => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "degraded",
                    "ready": false,
                    "service": "wendao-gateway",
                    "error": "gateway is not ready",
                    "pidfile": pidfile.display().to_string(),
                    "details": details,
                    "processId": process_id,
                    "planes": {
                        "http": "ready",
                        "flight": GATEWAY_FLIGHT_PLANE_STATUS,
                    },
                })),
            )
                .into_response(),
        },
        None => Json(json!({
            "status": "ok",
            "ready": true,
            "service": "wendao-gateway",
            "processId": process_id,
            "planes": {
                "http": "ready",
                "flight": GATEWAY_FLIGHT_PLANE_STATUS,
            },
        }))
        .into_response(),
    };

    response.headers_mut().insert(
        HeaderName::from_static(GATEWAY_PROCESS_ID_HEADER),
        HeaderValue::from_str(&process_id.to_string()).unwrap_or_else(|error| {
            panic!("gateway process id should be a valid header value: {error}")
        }),
    );

    response
}

pub(crate) fn gateway_pidfile_process_id(pidfile: &Path) -> Result<u32, String> {
    let contents = std::fs::read_to_string(pidfile)
        .map_err(|error| format!("failed to read pidfile {}: {error}", pidfile.display()))?;
    contents.trim().parse::<u32>().map_err(|error| {
        format!(
            "failed to parse pidfile {} as a process id: {error}",
            pidfile.display()
        )
    })
}
