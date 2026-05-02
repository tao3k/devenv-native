//! HTTP handlers for JSON-RPC dispatch and health checks.

use axum::{Extension, Json, body::Bytes};
use serde::Serialize;
use serde_json::{Value, json};
use std::time::Instant;

use crate::contracts::{
    INVALID_REQUEST_CODE, JsonRpcErrorObject, JsonRpcId, JsonRpcRequest, JsonRpcResponse,
    PARSE_ERROR_CODE,
};
use crate::router::MethodRegistry;

/// `/healthz` response payload.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct HealthResponse {
    /// Health state.
    pub status: &'static str,
    /// Service name.
    pub service: &'static str,
}

pub(super) async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "xiuxian-zhenfa",
    })
}

pub(super) async fn rpc_handler(
    Extension(method_registry): Extension<MethodRegistry>,
    body: Bytes,
) -> Json<JsonRpcResponse> {
    let request = match parse_request(&body) {
        Ok(request) => request,
        Err(response) => return Json(*response),
    };

    let id = request.id.clone();
    let started_at = Instant::now();
    match method_registry
        .call(&request.method, request.params, request.meta)
        .await
    {
        Ok(result) => Json(JsonRpcResponse::success(
            id,
            result,
            Some(execution_metrics(started_at)),
        )),
        Err(error) => Json(JsonRpcResponse::error(id, error)),
    }
}

fn parse_request(body: &[u8]) -> Result<JsonRpcRequest, Box<JsonRpcResponse>> {
    let raw: Value =
        serde_json::from_slice(body).map_err(|error| Box::new(parse_error_response(&error)))?;
    let id = extract_id_from_raw_value(&raw);
    let request: JsonRpcRequest = serde_json::from_value(raw).map_err(|error| {
        Box::new(JsonRpcResponse::error(
            id,
            JsonRpcErrorObject::new(
                INVALID_REQUEST_CODE,
                "invalid request envelope",
                Some(json!({ "details": error.to_string() })),
            ),
        ))
    })?;

    request
        .validate()
        .map_err(|error| Box::new(JsonRpcResponse::error(request.id.clone(), error)))?;
    Ok(request)
}

fn parse_error_response(error: &serde_json::Error) -> JsonRpcResponse {
    JsonRpcResponse::error(
        JsonRpcId::Null,
        JsonRpcErrorObject::new(
            PARSE_ERROR_CODE,
            "parse error",
            Some(json!({ "details": error.to_string() })),
        ),
    )
}

fn extract_id_from_raw_value(value: &Value) -> JsonRpcId {
    let Some(id_value) = value.as_object().and_then(|map| map.get("id")) else {
        return JsonRpcId::Null;
    };
    if let Some(id) = id_value.as_str() {
        return JsonRpcId::String(id.to_string());
    }
    if let Some(id) = id_value.as_i64() {
        return JsonRpcId::Number(id);
    }
    if let Some(id) = id_value.as_u64()
        && let Ok(parsed) = i64::try_from(id)
    {
        return JsonRpcId::Number(parsed);
    }
    JsonRpcId::Null
}

fn execution_metrics(started_at: Instant) -> Value {
    json!({ "execution_ms": started_at.elapsed().as_secs_f64() * 1000.0 })
}
