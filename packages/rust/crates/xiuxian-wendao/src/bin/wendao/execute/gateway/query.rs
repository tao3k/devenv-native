//! Gateway shared-query compatibility route.

use std::sync::Arc;
use std::time::Duration;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{
        IntoResponse, Response,
        sse::{Event, Sse},
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio_stream::iter;
use uuid::Uuid;
use xiuxian_wendao::search::queries::{
    SearchQueryService,
    rest::{RestQueryPayload, RestQueryRequest, query_rest_payload},
};

use crate::execute::gateway::shared::AppState;

/// Compatibility HTTP route used by bounded external query clients.
pub(crate) const GATEWAY_QUERY_AXUM_PATH: &str = "/query";
/// OpenAI-style public query response route over the shared Wendao query system.
pub(crate) const GATEWAY_RESPONSES_AXUM_PATH: &str = "/v1/responses";

#[derive(Debug, Deserialize)]
pub(crate) struct GatewayResponseRequest {
    #[serde(default)]
    input: Option<String>,
    #[serde(default)]
    query: Option<String>,
    #[serde(default)]
    document: Option<String>,
    #[serde(default = "default_response_query_language")]
    query_language: GatewayResponseQueryLanguage,
    #[serde(default)]
    stream: bool,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum GatewayResponseQueryLanguage {
    Sql,
    Graphql,
}

fn default_response_query_language() -> GatewayResponseQueryLanguage {
    GatewayResponseQueryLanguage::Sql
}

#[derive(Debug, Serialize)]
struct GatewayResponse {
    id: String,
    object: &'static str,
    status: &'static str,
    output: Vec<GatewayResponseOutput>,
}

#[derive(Debug, Serialize)]
struct GatewayResponseOutput {
    #[serde(rename = "type")]
    output_type: &'static str,
    json: Value,
}

#[derive(Debug, Serialize)]
struct GatewayResponseCompleted {
    id: String,
    object: &'static str,
    status: &'static str,
}

/// Execute one shared REST query request through the gateway.
pub(crate) async fn query(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RestQueryRequest>,
) -> Result<Json<RestQueryPayload>, (StatusCode, Json<serde_json::Value>)> {
    let service = SearchQueryService::new(state.studio.search_plane_service());
    query_rest_payload(&service, &request)
        .await
        .map(Json)
        .map_err(|details| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "failed to execute shared query request",
                    "code": "QUERY_EXECUTION_FAILED",
                    "details": details,
                })),
            )
        })
}

/// Execute one public response request through the gateway.
pub(crate) async fn responses(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<GatewayResponseRequest>,
) -> Response {
    let stream = request.stream || accepts_event_stream(&headers);
    let service = SearchQueryService::new(state.studio.search_plane_service());
    let rest_request = match response_rest_query_request(&request) {
        Ok(request) => request,
        Err(details) => return response_bad_request(&details),
    };
    match query_rest_payload(&service, &rest_request).await {
        Ok(payload) if stream => response_sse(payload),
        Ok(payload) => Json(response_json(payload)).into_response(),
        Err(details) => response_bad_request(&details),
    }
}

fn accepts_event_stream(headers: &HeaderMap) -> bool {
    headers
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            value
                .split(',')
                .any(|media_type| media_type.trim().starts_with("text/event-stream"))
        })
}

fn response_rest_query_request(
    request: &GatewayResponseRequest,
) -> Result<RestQueryRequest, String> {
    match request.query_language {
        GatewayResponseQueryLanguage::Graphql => {
            let Some(document) =
                non_empty_response_field(request.document.as_deref().or(request.input.as_deref()))
            else {
                return Err("`document` or `input` must be a non-empty string".to_string());
            };
            Ok(RestQueryRequest::Graphql { document })
        }
        GatewayResponseQueryLanguage::Sql => {
            let Some(query) =
                non_empty_response_field(request.query.as_deref().or(request.input.as_deref()))
            else {
                return Err("`query` or `input` must be a non-empty string".to_string());
            };
            Ok(RestQueryRequest::Sql { query })
        }
    }
}

fn non_empty_response_field(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn response_json(payload: RestQueryPayload) -> GatewayResponse {
    let payload = serde_json::to_value(payload).unwrap_or_else(|error| {
        json!({
            "error": "failed to serialize response payload",
            "details": error.to_string(),
        })
    });
    GatewayResponse {
        id: format!("resp_{}", Uuid::new_v4().simple()),
        object: "response",
        status: "completed",
        output: vec![GatewayResponseOutput {
            output_type: "output_json",
            json: payload,
        }],
    }
}

fn response_sse(payload: RestQueryPayload) -> Response {
    let response = response_json(payload);
    let completed = GatewayResponseCompleted {
        id: response.id.clone(),
        object: "response",
        status: "completed",
    };
    let events = vec![
        Ok::<Event, std::convert::Infallible>(
            Event::default()
                .event("response.output_json.delta")
                .json_data(&response)
                .unwrap_or_else(|_| Event::default().event("response.output_json.delta")),
        ),
        Ok(Event::default()
            .event("response.completed")
            .json_data(&completed)
            .unwrap_or_else(|_| Event::default().event("response.completed"))),
    ];
    Sse::new(iter(events))
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keep-alive"),
        )
        .into_response()
}

fn response_bad_request(details: &str) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "error": "failed to execute public response request",
            "code": "RESPONSE_EXECUTION_FAILED",
            "details": details,
        })),
    )
        .into_response()
}
