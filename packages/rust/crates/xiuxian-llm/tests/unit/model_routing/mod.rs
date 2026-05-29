use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use xiuxian_llm::model_routing::{
    VLLM_SR_SELECTED_CONFIDENCE_HEADER, VLLM_SR_SELECTED_DECISION_HEADER,
    VLLM_SR_SELECTED_MODALITY_HEADER, VLLM_SR_SELECTED_MODEL_HEADER,
    VLLM_SR_SELECTED_REASONING_HEADER,
};

mod config;
mod route_decision;
mod vllm_sr_probe;

pub(super) fn vllm_sr_chat_probe(
    State(requests): State<Arc<Mutex<Vec<Value>>>>,
    body: &Bytes,
) -> Response {
    let payload = serde_json::from_slice::<Value>(body)
        .unwrap_or_else(|error| Value::String(format!("invalid_json:{error}")));
    requests
        .lock()
        .unwrap_or_else(|_| panic!("request capture lock should not be poisoned"))
        .push(payload);

    let mut response = (StatusCode::OK, r#"{"model":"fallback-model"}"#).into_response();
    response.headers_mut().insert(
        VLLM_SR_SELECTED_DECISION_HEADER,
        HeaderValue::from_static("audio-decision"),
    );
    response.headers_mut().insert(
        VLLM_SR_SELECTED_MODEL_HEADER,
        HeaderValue::from_static("qwen/qwen3-asr"),
    );
    response.headers_mut().insert(
        VLLM_SR_SELECTED_CONFIDENCE_HEADER,
        HeaderValue::from_static("0.99"),
    );
    response.headers_mut().insert(
        VLLM_SR_SELECTED_REASONING_HEADER,
        HeaderValue::from_static("off"),
    );
    response.headers_mut().insert(
        VLLM_SR_SELECTED_MODALITY_HEADER,
        HeaderValue::from_static("AR"),
    );
    response
}
