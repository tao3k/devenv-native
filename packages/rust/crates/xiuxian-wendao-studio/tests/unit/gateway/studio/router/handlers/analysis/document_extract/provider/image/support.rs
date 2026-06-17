use std::sync::{Arc, Mutex};

use axum::http::{HeaderMap, HeaderValue};
use axum::{Json, Router, routing::post};
use serde_json::json;
use tokio::net::TcpListener;
use xiuxian_io::model_routing::{
    VLLM_SR_REQUEST_ID_HEADER, VLLM_SR_SELECTED_DECISION_HEADER, VLLM_SR_SELECTED_MODEL_HEADER,
};

pub(super) async fn spawn_vllm_sr_route_probe(
    observed_payload: Arc<Mutex<Option<serde_json::Value>>>,
) -> Result<(String, tokio::task::JoinHandle<()>), String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|error| format!("bind vLLM-SR route probe: {error}"))?;
    let address = listener
        .local_addr()
        .map_err(|error| format!("read vLLM-SR route probe address: {error}"))?;
    let app = Router::new().route(
        "/v1/chat/completions",
        post(move |Json(payload): Json<serde_json::Value>| {
            let observed_payload = Arc::clone(&observed_payload);
            async move {
                *observed_payload
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(payload);
                let mut headers = HeaderMap::new();
                headers.insert(
                    VLLM_SR_SELECTED_MODEL_HEADER,
                    HeaderValue::from_static("qwen/qwen3-vl-8b-instruct"),
                );
                headers.insert(
                    VLLM_SR_SELECTED_DECISION_HEADER,
                    HeaderValue::from_static("image-document-vlm"),
                );
                headers.insert(
                    VLLM_SR_REQUEST_ID_HEADER,
                    HeaderValue::from_static("route-image-1"),
                );
                (headers, Json(json!({"model": "qwen/qwen3-vl-8b-instruct"})))
            }
        }),
    );
    let handle = tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, app).await {
            panic!("vLLM-SR route probe failed: {error}");
        }
    });
    Ok((format!("http://{address}"), handle))
}
