use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Semaphore;
use xiuxian_macros::env_non_empty;

use crate::config::{RuntimeSettings, load_runtime_settings};
use crate::llm::backend::{
    LlmBackendMode, extract_api_base_from_inference_url, parse_backend_mode,
};
use crate::llm::client::LlmClient;
#[cfg(feature = "agent-provider-litellm")]
use crate::llm::compat::litellm::LiteLlmRuntime;
use crate::llm::providers::{ProviderSettings, resolve_provider_settings};

impl LlmClient {
    pub fn new(inference_url: String, model: String, api_key: Option<String>) -> Self {
        let runtime_settings = load_runtime_settings();
        let env_backend = env_non_empty!("XIUXIAN_DAOCHANG_LLM_BACKEND");
        let (backend_mode, backend_source) = resolve_backend_mode_for_inference_url(
            &runtime_settings,
            &inference_url,
            env_backend.as_deref(),
        );
        let provider_settings = resolve_provider_settings(&runtime_settings, model);
        let ProviderSettings {
            mode: litellm_provider_mode,
            wire_api: litellm_wire_api,
            source: litellm_provider_source,
            api_key: provider_api_key,
            api_key_env: litellm_api_key_env,
            minimax_api_base,
            model,
            timeout_secs: inference_timeout_secs,
            max_tokens: inference_max_tokens,
            max_in_flight: inference_max_in_flight,
        } = provider_settings;
        let api_key = provider_api_key.or(api_key);
        let in_flight_gate = inference_max_in_flight.map(|limit| Arc::new(Semaphore::new(limit)));
        let inference_api_base = extract_api_base_from_inference_url(&inference_url);
        tracing::info!(
            llm_backend = backend_mode.as_str(),
            llm_backend_source = backend_source,
            litellm_provider = litellm_provider_mode.as_str(),
            litellm_wire_api = litellm_wire_api.as_str(),
            litellm_provider_source = litellm_provider_source,
            litellm_api_key_env = %litellm_api_key_env,
            minimax_api_base = %minimax_api_base,
            inference_timeout_secs = inference_timeout_secs,
            inference_max_tokens = inference_max_tokens,
            inference_max_in_flight = inference_max_in_flight,
            model = %model,
            inference_api_base = %inference_api_base,
            "llm backend selected"
        );
        Self {
            client: build_http_client(),
            inference_url,
            #[cfg(feature = "agent-provider-litellm")]
            inference_api_base,
            model,
            api_key,
            backend_mode,
            litellm_provider_mode,
            litellm_wire_api,
            #[cfg(feature = "agent-provider-litellm")]
            litellm_api_key_env,
            #[cfg(feature = "agent-provider-litellm")]
            minimax_api_base,
            inference_timeout_secs,
            inference_max_tokens,
            inference_max_in_flight,
            in_flight_gate,
            #[cfg(feature = "agent-provider-litellm")]
            litellm_runtime: LiteLlmRuntime::new(),
        }
    }
}

fn resolve_backend_mode_for_inference_url(
    runtime_settings: &RuntimeSettings,
    inference_url: &str,
    env_backend_raw: Option<&str>,
) -> (LlmBackendMode, &'static str) {
    if let Some(raw) = env_backend_raw
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return (parse_backend_mode(Some(raw)), "env");
    }

    if should_prefer_http_backend_for_inference_url(runtime_settings, inference_url) {
        return (LlmBackendMode::OpenAiCompatibleHttp, "inference_url");
    }

    let settings_backend = runtime_settings
        .agent
        .llm_backend
        .as_deref()
        .map(str::trim)
        .map(ToString::to_string)
        .filter(|raw| !raw.is_empty());
    if let Some(raw) = settings_backend.as_deref() {
        (parse_backend_mode(Some(raw)), "settings")
    } else {
        (parse_backend_mode(None), "default")
    }
}

fn should_prefer_http_backend_for_inference_url(
    runtime_settings: &RuntimeSettings,
    inference_url: &str,
) -> bool {
    let trimmed = inference_url.trim();
    if trimmed.is_empty() {
        return false;
    }

    let Some(parsed) = reqwest::Url::parse(trimmed).ok() else {
        return false;
    };
    let Some(host) = parsed.host_str() else {
        return false;
    };
    let is_loopback = host.eq_ignore_ascii_case("localhost")
        || host == "127.0.0.1"
        || host == "::1"
        || parsed.domain().is_none()
            && host
                .parse::<std::net::IpAddr>()
                .is_ok_and(|ip| ip.is_loopback());
    if !is_loopback {
        return false;
    }

    let Some(configured_base) = runtime_settings
        .inference
        .base_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return true;
    };

    let configured = configured_base.trim_end_matches('/');
    let explicit = trimmed.trim_end_matches('/');
    configured != explicit
}

pub(crate) fn test_resolve_backend_mode_for_inference_url(
    runtime_settings: &RuntimeSettings,
    inference_url: &str,
    env_backend_raw: Option<&str>,
) -> (LlmBackendMode, &'static str) {
    resolve_backend_mode_for_inference_url(runtime_settings, inference_url, env_backend_raw)
}

fn build_http_client() -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(64)
        .tcp_nodelay(true);
    if !system_proxy_enabled() {
        builder = builder.no_proxy();
    }
    match builder.build() {
        Ok(client) => client,
        Err(error) => {
            tracing::warn!(
                error = %error,
                "failed to build tuned llm http client; falling back to default client"
            );
            reqwest::Client::new()
        }
    }
}

fn system_proxy_enabled() -> bool {
    env_non_empty!("XIUXIAN_DAOCHANG_HTTP_ENABLE_SYSTEM_PROXY")
        .map(|raw| raw.trim().to_ascii_lowercase())
        .is_some_and(|raw| matches!(raw.as_str(), "1" | "true" | "yes" | "on"))
}
