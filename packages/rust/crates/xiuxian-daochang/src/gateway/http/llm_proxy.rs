//! HTTP LLM proxy request handling.

// LLM Gateway Proxy Handler
//
// Provides a reverse proxy for OpenAI-compatible `/v1/chat/completions` requests.
// It dynamically routes requests to different providers (`OpenAI`, `MiniMax`, Anthropic, etc.)
// based on the model prefix (e.g., `minimax/MiniMax-M2.1`) or falls back to runtime TOML configuration.

use crate::config::{load_runtime_settings, load_xiuxian_config};
use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use reqwest::Client;
use serde_json::Value;
use std::sync::LazyLock;
use xiuxian_macros::{env_non_empty, string_first_non_empty};

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .build()
        .unwrap_or_else(|error| panic!("failed to build HTTP client for LLM proxy: {error}"))
});

fn resolve_azure_openai_endpoint() -> String {
    env_non_empty!("AZURE_OPENAI_ENDPOINT").unwrap_or_default()
}

pub(crate) fn resolve_target_base_url(
    base_url_override: Option<&str>,
    resolved_base_url: &str,
) -> String {
    string_first_non_empty!(base_url_override, Some(resolved_base_url))
}

pub(crate) fn resolve_target_api_key_env(
    api_key_env_override: Option<&str>,
    resolved_key_env: &str,
) -> String {
    string_first_non_empty!(api_key_env_override, Some(resolved_key_env))
}

fn is_env_var_name(raw: &str) -> bool {
    let mut chars = raw.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_uppercase()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_uppercase() || ch.is_ascii_digit())
}

pub(crate) fn read_api_key(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if let Some(env_name) = trimmed.strip_prefix("env:")
        && is_env_var_name(env_name)
    {
        return std::env::var(env_name).unwrap_or_default();
    }
    if trimmed.starts_with("${")
        && trimmed.ends_with('}')
        && trimmed.len() > 3
        && is_env_var_name(&trimmed[2..trimmed.len() - 1])
    {
        return std::env::var(&trimmed[2..trimmed.len() - 1]).unwrap_or_default();
    }
    if is_env_var_name(trimmed) {
        return std::env::var(trimmed).unwrap_or_default();
    }
    trimmed.to_string()
}

pub(crate) fn resolve_request_model(
    request_model: Option<&str>,
    inference_default_model: Option<&str>,
    xiuxian_default_model: Option<&str>,
) -> Option<String> {
    request_model
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .or_else(|| {
            inference_default_model
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
        })
        .or_else(|| {
            xiuxian_default_model
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
        })
}

/// Map provider alias to Base URL and API Key Env Var using xiuxian.toml or hardcoded fallbacks
fn resolve_provider_config(provider: &str) -> (String, String) {
    let xiuxian_conf = load_xiuxian_config();
    if let Some(cfg) = xiuxian_conf.llm.providers.get(provider)
        && let (Some(url), Some(api_key)) = (&cfg.base_url, &cfg.api_key)
    {
        return (url.clone(), api_key.clone());
    }

    match provider {
        "minimax" => (
            "https://api.minimax.io/v1".to_string(),
            "MINIMAX_API_KEY".to_string(),
        ),
        "anthropic" => (
            "https://api.anthropic.com/v1".to_string(),
            "ANTHROPIC_API_KEY".to_string(),
        ),
        "azure" => (
            resolve_azure_openai_endpoint(),
            "AZURE_OPENAI_API_KEY".to_string(),
        ),
        "deepseek" => (
            "https://api.deepseek.com/v1".to_string(),
            "DEEPSEEK_API_KEY".to_string(),
        ),
        _ => (
            "https://api.openai.com/v1".to_string(),
            "OPENAI_API_KEY".to_string(),
        ),
    }
}

/// Apply model aliases from xiuxian.toml
fn apply_model_alias(provider: &str, model: &str) -> String {
    let xiuxian_conf = load_xiuxian_config();
    let lower_model = model.to_lowercase();

    if let Some(cfg) = xiuxian_conf.llm.providers.get(provider)
        && let Some(alias) = cfg.model_aliases.get(&lower_model)
    {
        return alias.clone();
    }

    // Hardcoded fallback for minimax casing fix
    if provider == "minimax"
        && let Some(suffix) = lower_model.strip_prefix("minimax-")
    {
        let mut actual_model = format!("MiniMax-{suffix}");
        if lower_model == "minimax-m2.1-highspeed" {
            actual_model = "MiniMax-M2.1-lightning".to_string();
        }
        return actual_model;
    }

    model.to_string()
}

/// Handle `/v1/chat/completions` request.
/// Parses the JSON body to extract `model`.
/// Uses `provider/model_name` syntax if present (e.g., `minimax/MiniMax-M2.5`).
/// Otherwise, falls back to the configured provider in `xiuxian.toml`.
pub async fn handle_chat_completions(
    req: Request,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let settings = load_runtime_settings();
    let xiuxian_conf = load_xiuxian_config();

    let default_provider = settings
        .inference
        .provider
        .or(xiuxian_conf.llm.default_provider)
        .unwrap_or_else(|| "openai".to_string());

    // Read body as JSON to modify the model string
    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read body: {e}")))?;

    let mut json_body: Value = serde_json::from_slice(&body_bytes)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid JSON: {e}")))?;

    let mut provider = default_provider.clone();
    let mut base_url = settings.inference.base_url.clone();
    let mut api_key_env = settings.inference.api_key.clone();

    let request_model = json_body.get("model").and_then(Value::as_str);
    let resolved_model = resolve_request_model(
        request_model,
        settings.inference.model.as_deref(),
        xiuxian_conf.llm.default_model.as_deref(),
    )
    .ok_or((
        StatusCode::BAD_REQUEST,
        "model is required (request.model, settings.inference.model, or xiuxian.llm.default_model)"
            .to_string(),
    ))?;

    let mut actual_model = resolved_model.clone();
    if let Some((prefix, suffix)) = resolved_model.split_once('/') {
        provider = prefix.to_lowercase();
        actual_model = suffix.to_string();
        // Clear custom base_url if provider is selected from model prefix.
        base_url = None;
        api_key_env = None;
    }
    actual_model = apply_model_alias(&provider, &actual_model);

    if let Some(object) = json_body.as_object_mut() {
        object.insert("model".to_string(), Value::String(actual_model));
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            "request body must be a JSON object".to_string(),
        ));
    }

    let (resolved_base_url, resolved_key_env) = resolve_provider_config(&provider);
    let target_base_url = resolve_target_base_url(base_url.as_deref(), resolved_base_url.as_str());
    let target_api_key_env =
        resolve_target_api_key_env(api_key_env.as_deref(), resolved_key_env.as_str());

    let api_key = read_api_key(target_api_key_env.as_str());
    if api_key.is_empty() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "Missing API key for provider {provider}. Configure [llm.providers.{provider}].api_key or set {target_api_key_env}"
            ),
        ));
    }

    let target_url = format!("{}/chat/completions", target_base_url.trim_end_matches('/'));

    let mut proxy_req_builder = HTTP_CLIENT.post(&target_url).json(&json_body);

    // Some providers (like Anthropic) require specific headers.
    if provider == "anthropic" {
        proxy_req_builder = proxy_req_builder
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01");
    } else {
        proxy_req_builder = proxy_req_builder.header("Authorization", format!("Bearer {api_key}"));
    }

    let upstream_res = proxy_req_builder.send().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            format!("Upstream request failed: {e}"),
        )
    })?;

    let status = upstream_res.status();
    let headers = upstream_res.headers().clone();
    let body_bytes = upstream_res.bytes().await.map_err(|error| {
        (
            StatusCode::BAD_GATEWAY,
            format!("Failed to read upstream response body: {error}"),
        )
    })?;
    let body_stream = Body::from(body_bytes);

    let mut axum_res = body_stream.into_response();
    *axum_res.status_mut() = status;

    // Forward essential headers
    for (k, v) in headers {
        if let Some(key) = k
            && (key == axum::http::header::CONTENT_TYPE
                || key == axum::http::header::CONTENT_ENCODING
                || key.as_str().starts_with("x-"))
        {
            axum_res.headers_mut().insert(key, v);
        }
    }

    Ok(axum_res)
}
