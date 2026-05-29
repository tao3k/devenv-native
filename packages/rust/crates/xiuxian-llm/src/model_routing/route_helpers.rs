//! Route-level helpers for vLLM-SR decision parsing and trace construction.

use reqwest::header::HeaderMap;
use serde_json::json;

use super::constants::{
    DEFAULT_WENDAO_VLLM_SR_BASE_URL, VLLM_SR_SELECTED_CONFIDENCE_HEADER,
    VLLM_SR_SELECTED_MODALITY_HEADER, VLLM_SR_SELECTED_REASONING_HEADER,
};

pub(in crate::model_routing) fn required_string(
    value: &serde_json::Value,
    keys: &[&str],
) -> Result<String, String> {
    optional_string(value, keys).ok_or_else(|| {
        format!(
            "vLLM-SR decision response missing required field `{}`",
            keys[0]
        )
    })
}

pub(in crate::model_routing) fn optional_string(
    value: &serde_json::Value,
    keys: &[&str],
) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    })
}

pub(in crate::model_routing) fn normalize_base_url(value: &str) -> String {
    let trimmed = value.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        DEFAULT_WENDAO_VLLM_SR_BASE_URL.to_owned()
    } else {
        trimmed.to_owned()
    }
}

pub(in crate::model_routing) fn header_string(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

pub(in crate::model_routing) fn response_body_selected_model(
    response_body: &str,
) -> Option<String> {
    let value = serde_json::from_str::<serde_json::Value>(response_body).ok()?;
    optional_string(&value, &["model", "selected_model", "selectedModel"])
}

pub(in crate::model_routing) fn build_vllm_sr_route_id(
    selected_decision: Option<&str>,
    selected_model: &str,
) -> String {
    format!(
        "vllm-sr:{}:{}",
        sanitize_route_id_part(selected_decision.unwrap_or("unknown-decision")),
        sanitize_route_id_part(selected_model)
    )
}

fn sanitize_route_id_part(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    out.trim_matches('-').to_owned()
}

pub(in crate::model_routing) fn build_vllm_sr_route_trace(
    headers: &HeaderMap,
    selected_decision: Option<&str>,
) -> Option<String> {
    let trace = json!({
        "selectedDecision": selected_decision,
        "confidence": header_string(headers, VLLM_SR_SELECTED_CONFIDENCE_HEADER),
        "reasoning": header_string(headers, VLLM_SR_SELECTED_REASONING_HEADER),
        "modality": header_string(headers, VLLM_SR_SELECTED_MODALITY_HEADER),
    });
    serde_json::to_string(&trace).ok()
}
