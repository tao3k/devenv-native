//! Model routing contracts shared by Wendao Gateway and model-plane adapters.

#[path = "model_routing/attachment.rs"]
mod attachment;
#[path = "model_routing/chat.rs"]
mod chat;
#[path = "model_routing/config.rs"]
mod config;

use reqwest::header::{AUTHORIZATION, HeaderMap};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub use attachment::{
    DEFAULT_WENDAO_ATTACHMENT_ROUTE_PROVIDER,
    DEFAULT_WENDAO_AUDIO_TRANSCRIPT_ROUTE_BACKEND_PROFILE,
    DEFAULT_WENDAO_AUDIO_TRANSCRIPT_ROUTE_MODEL,
    DEFAULT_WENDAO_IMAGE_EXTRACT_ROUTE_BACKEND_PROFILE, DEFAULT_WENDAO_IMAGE_EXTRACT_ROUTE_MODEL,
    WENDAO_AUDIO_TRANSCRIPT_ROUTE_BACKEND_PROFILE_ENV, WENDAO_AUDIO_TRANSCRIPT_ROUTE_MODEL_ENV,
    WENDAO_AUDIO_TRANSCRIPT_ROUTE_PROVIDER_ENV, WENDAO_IMAGE_EXTRACT_ROUTE_BACKEND_PROFILE_ENV,
    WENDAO_IMAGE_EXTRACT_ROUTE_MODEL_ENV, WENDAO_IMAGE_EXTRACT_ROUTE_PROVIDER_ENV,
    WendaoAttachmentRouteConfig, WendaoAttachmentRouteInput,
    wendao_attachment_model_route_decision, wendao_attachment_route_intent,
    wendao_audio_transcript_route_config_with_lookup,
    wendao_audio_transcript_route_config_with_model_routing_config,
    wendao_image_extract_route_config_with_lookup,
    wendao_image_extract_route_config_with_model_routing_config,
};
pub use chat::{
    DEFAULT_WENDAO_CHAT_ROUTE_BACKEND_PROFILE, DEFAULT_WENDAO_CHAT_ROUTE_MODEL,
    WENDAO_CHAT_ROUTE_BACKEND_PROFILE_ENV, WENDAO_CHAT_ROUTE_MODEL_ENV,
    WENDAO_CHAT_ROUTE_PROVIDER_ENV, WendaoChatRouteConfig, WendaoChatRouteInput,
    wendao_chat_model_route_decision, wendao_chat_route_config_with_lookup,
    wendao_chat_route_config_with_model_routing_config, wendao_chat_route_intent,
};
pub use config::{
    WENDAO_MODEL_ROUTING_SYSTEM_DEFAULT_TOML, WendaoModelRoutingTomlConfig, WendaoRouteTomlConfig,
    wendao_model_routing_config_from_toml_str, wendao_model_routing_config_from_toml_value,
    wendao_model_routing_system_default_config,
};

/// Wendao model routing mode environment variable.
pub const WENDAO_MODEL_ROUTING_MODE_ENV: &str = "WENDAO_MODEL_ROUTING_MODE";
/// vLLM-SR base URL environment variable.
pub const WENDAO_VLLM_SR_BASE_URL_ENV: &str = "WENDAO_VLLM_SR_BASE_URL";
/// vLLM-SR config path environment variable.
pub const WENDAO_VLLM_SR_CONFIG_PATH_ENV: &str = "WENDAO_VLLM_SR_CONFIG_PATH";

/// Stable route id metadata header.
pub const WENDAO_ROUTE_ID_HEADER: &str = "x-wendao-route-id";
/// Stable route task-kind metadata header.
pub const WENDAO_ROUTE_TASK_KIND_HEADER: &str = "x-wendao-route-task-kind";
/// Stable route modality metadata header.
pub const WENDAO_ROUTE_MODALITY_HEADER: &str = "x-wendao-route-modality";
/// Stable selected-provider metadata header.
pub const WENDAO_ROUTE_SELECTED_PROVIDER_HEADER: &str = "x-wendao-route-selected-provider";
/// Stable selected-model metadata header.
pub const WENDAO_ROUTE_SELECTED_MODEL_HEADER: &str = "x-wendao-route-selected-model";
/// Stable selected backend-profile metadata header.
pub const WENDAO_ROUTE_SELECTED_BACKEND_PROFILE_HEADER: &str =
    "x-wendao-route-selected-backend-profile";
/// Stable precision-tier metadata header.
pub const WENDAO_ROUTE_PRECISION_TIER_HEADER: &str = "x-wendao-route-precision-tier";

/// Default vLLM-SR proxy endpoint used by Wendao.
pub const DEFAULT_WENDAO_VLLM_SR_BASE_URL: &str = "http://127.0.0.1:8888";
/// Default local model-routing mode for developer experience.
pub const DEFAULT_WENDAO_MODEL_ROUTING_MODE: &str = "deterministic";
/// vLLM-SR auto model token used by the OpenAI-compatible data plane.
pub const VLLM_SR_AUTO_MODEL: &str = "auto";
/// vLLM-SR selected decision response header.
pub const VLLM_SR_SELECTED_DECISION_HEADER: &str = "x-vsr-selected-decision";
/// vLLM-SR selected model response header.
pub const VLLM_SR_SELECTED_MODEL_HEADER: &str = "x-vsr-selected-model";
/// vLLM-SR selected confidence response header.
pub const VLLM_SR_SELECTED_CONFIDENCE_HEADER: &str = "x-vsr-selected-confidence";
/// vLLM-SR selected reasoning response header.
pub const VLLM_SR_SELECTED_REASONING_HEADER: &str = "x-vsr-selected-reasoning";
/// vLLM-SR selected modality response header.
pub const VLLM_SR_SELECTED_MODALITY_HEADER: &str = "x-vsr-selected-modality";
/// vLLM-SR request id response header.
pub const VLLM_SR_REQUEST_ID_HEADER: &str = "x-request-id";

/// Gateway task-kind token for a route intent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WendaoRouteTaskKind(String);

impl WendaoRouteTaskKind {
    /// Build a task-kind token.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Return the stable task-kind token.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for WendaoRouteTaskKind {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for WendaoRouteTaskKind {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Gateway source-kind token for a route intent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WendaoRouteSourceKind(String);

impl WendaoRouteSourceKind {
    /// Build a source-kind token.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Return the stable source-kind token.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for WendaoRouteSourceKind {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for WendaoRouteSourceKind {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Model-routing runtime mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WendaoModelRoutingMode {
    /// Use vLLM-SR as the required model routing plane.
    VllmSr,
    /// Use Gateway-owned deterministic local route policy.
    Deterministic,
}

impl WendaoModelRoutingMode {
    /// Parse a routing mode value.
    ///
    /// # Errors
    ///
    /// Returns an error when the value is not `vllm-sr` or `deterministic`.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "deterministic" => Ok(Self::Deterministic),
            "vllm-sr" | "vllm_sr" => Ok(Self::VllmSr),
            other => Err(format!(
                "unsupported {WENDAO_MODEL_ROUTING_MODE_ENV} value `{other}`"
            )),
        }
    }
}

/// Resolve model-routing mode using an injectable lookup.
///
/// # Errors
///
/// Returns an error when the configured mode is invalid.
pub fn wendao_model_routing_mode_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<WendaoModelRoutingMode, String> {
    WendaoModelRoutingMode::parse(
        lookup(WENDAO_MODEL_ROUTING_MODE_ENV)
            .as_deref()
            .unwrap_or(DEFAULT_WENDAO_MODEL_ROUTING_MODE),
    )
}

/// Resolve the vLLM-SR base URL using an injectable lookup.
#[must_use]
pub fn wendao_vllm_sr_base_url_with_lookup(lookup: &dyn Fn(&str) -> Option<String>) -> String {
    lookup(WENDAO_VLLM_SR_BASE_URL_ENV)
        .map(|value| value.trim().trim_end_matches('/').to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_WENDAO_VLLM_SR_BASE_URL.to_owned())
}

/// Gateway-owned route intent sent to the model routing plane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WendaoRouteIntent {
    /// Gateway task kind.
    pub task_kind: WendaoRouteTaskKind,
    /// Source modality.
    pub modality: String,
    /// Source kind.
    pub source_kind: WendaoRouteSourceKind,
    /// Precision tier required by the Gateway precision gate.
    pub precision_tier: String,
    /// Privacy tier for provider policy.
    pub privacy_tier: String,
    /// Latency budget in milliseconds.
    pub latency_budget_ms: u64,
    /// Evidence profile requested by Gateway.
    pub evidence_profile: String,
    /// Artifact references available to the selected backend.
    pub artifact_refs: Vec<String>,
}

/// Model decision returned by the model routing plane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WendaoModelDecision {
    /// Stable route identifier for traceability.
    pub route_id: String,
    /// Selected provider key.
    pub selected_provider: String,
    /// Selected provider model identifier.
    pub selected_model: String,
    /// Selected Wendao backend profile.
    pub selected_backend_profile: String,
    /// Optional reasoning policy selected by routing.
    pub reasoning_policy: Option<String>,
    /// Optional route trace summary.
    pub route_trace: Option<String>,
}

impl WendaoModelDecision {
    /// Parse a vLLM-SR decision response from JSON.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload is malformed or misses required
    /// selected provider/model/backend fields.
    pub fn from_vllm_sr_response_json(payload: &str) -> Result<Self, String> {
        let value: serde_json::Value = serde_json::from_str(payload)
            .map_err(|error| format!("parse vLLM-SR decision JSON: {error}"))?;
        let decision = value.get("decision").unwrap_or(&value);
        let route_id = required_string(decision, &["routeId", "route_id", "id"])?;
        let selected_provider = required_string(
            decision,
            &["selectedProvider", "selected_provider", "provider"],
        )?;
        let selected_model =
            required_string(decision, &["selectedModel", "selected_model", "model"])?;
        let selected_backend_profile = required_string(
            decision,
            &[
                "selectedBackendProfile",
                "selected_backend_profile",
                "backendProfile",
                "backend_profile",
            ],
        )?;
        Ok(Self {
            route_id,
            selected_provider,
            selected_model,
            selected_backend_profile,
            reasoning_policy: optional_string(decision, &["reasoningPolicy", "reasoning_policy"]),
            route_trace: optional_string(decision, &["routeTrace", "route_trace"]).or_else(|| {
                decision
                    .get("trace")
                    .and_then(|trace| serde_json::to_string(trace).ok())
            }),
        })
    }

    /// Parse a vLLM-SR data-plane response into a Wendao model decision.
    ///
    /// # Errors
    ///
    /// Returns an error when vLLM-SR did not return a selected model, or when
    /// the Gateway-supplied selected provider/backend profile is empty.
    pub fn from_vllm_sr_response_parts(
        headers: &HeaderMap,
        response_body: &str,
        selected_provider: &str,
        selected_backend_profile: &str,
    ) -> Result<Self, String> {
        let selected_provider = selected_provider.trim();
        if selected_provider.is_empty() {
            return Err("vLLM-SR route decision requires a selected provider hint".to_owned());
        }
        let selected_backend_profile = selected_backend_profile.trim();
        if selected_backend_profile.is_empty() {
            return Err(
                "vLLM-SR route decision requires a selected backend profile hint".to_owned(),
            );
        }
        let selected_model = header_string(headers, VLLM_SR_SELECTED_MODEL_HEADER)
            .or_else(|| response_body_selected_model(response_body))
            .ok_or_else(|| {
                format!("vLLM-SR response missing `{VLLM_SR_SELECTED_MODEL_HEADER}` header")
            })?;
        let selected_decision = header_string(headers, VLLM_SR_SELECTED_DECISION_HEADER);
        let route_id = header_string(headers, VLLM_SR_REQUEST_ID_HEADER).unwrap_or_else(|| {
            build_vllm_sr_route_id(selected_decision.as_deref(), &selected_model)
        });
        let route_trace = build_vllm_sr_route_trace(headers, selected_decision.as_deref());
        Ok(Self {
            route_id,
            selected_provider: selected_provider.to_owned(),
            selected_model,
            selected_backend_profile: selected_backend_profile.to_owned(),
            reasoning_policy: header_string(headers, VLLM_SR_SELECTED_REASONING_HEADER),
            route_trace,
        })
    }
}

/// OpenAI-compatible vLLM-SR route-decision probe client.
#[derive(Clone)]
pub struct VllmSrRouteDecisionClient {
    base_url: String,
    bearer_token: Option<String>,
    http: reqwest::Client,
}

impl VllmSrRouteDecisionClient {
    /// Build a route decision client.
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self::with_http_client(base_url, reqwest::Client::new())
    }

    /// Build a route decision client with an injected HTTP client.
    #[must_use]
    pub fn with_http_client(base_url: impl Into<String>, http: reqwest::Client) -> Self {
        let base_url = base_url.into();
        Self {
            base_url: normalize_base_url(base_url.as_str()),
            bearer_token: None,
            http,
        }
    }

    /// Attach an optional bearer token for vLLM-SR deployments that require it.
    #[must_use]
    pub fn with_bearer_token(mut self, bearer_token: impl Into<String>) -> Self {
        let bearer_token = bearer_token.into();
        self.bearer_token = (!bearer_token.trim().is_empty()).then_some(bearer_token);
        self
    }

    /// Return the normalized vLLM-SR base URL.
    #[must_use]
    pub fn base_url(&self) -> &str {
        self.base_url.as_str()
    }

    /// Obtain a route decision through vLLM-SR's OpenAI-compatible data plane.
    ///
    /// # Errors
    ///
    /// Returns an error when the HTTP request fails, vLLM-SR returns a
    /// non-success status, or the response does not include a selected model.
    pub async fn decide(
        &self,
        intent: &WendaoRouteIntent,
        selected_provider: &str,
        selected_backend_profile: &str,
    ) -> Result<WendaoModelDecision, String> {
        let endpoint = format!("{}/v1/chat/completions", self.base_url);
        let prompt = serde_json::to_string(intent)
            .map_err(|error| format!("serialize Wendao route intent: {error}"))?;
        let payload = json!({
            "model": VLLM_SR_AUTO_MODEL,
            "messages": [
                {
                    "role": "system",
                    "content": "Route this Wendao task. The response body is ignored; Wendao reads vLLM-SR routing headers."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0,
            "max_tokens": 1,
            "stream": false
        });

        let mut request = self.http.post(endpoint.as_str()).json(&payload);
        if let Some(token) = self.bearer_token.as_deref() {
            request = request.header(AUTHORIZATION, format!("Bearer {token}"));
        }
        let response = request
            .send()
            .await
            .map_err(|error| format!("call vLLM-SR route probe `{endpoint}`: {error}"))?;
        let status = response.status();
        let headers = response.headers().clone();
        let body = response
            .text()
            .await
            .map_err(|error| format!("read vLLM-SR route probe response body: {error}"))?;
        if !status.is_success() {
            return Err(format!(
                "vLLM-SR route probe `{endpoint}` returned {status}: {body}"
            ));
        }
        WendaoModelDecision::from_vllm_sr_response_parts(
            &headers,
            &body,
            selected_provider,
            selected_backend_profile,
        )
    }
}

/// Emit stable Flight metadata pairs for a route intent and decision.
#[must_use]
pub fn wendao_model_route_metadata(
    intent: &WendaoRouteIntent,
    decision: &WendaoModelDecision,
) -> Vec<(&'static str, String)> {
    vec![
        (WENDAO_ROUTE_ID_HEADER, decision.route_id.clone()),
        (
            WENDAO_ROUTE_TASK_KIND_HEADER,
            intent.task_kind.as_str().to_owned(),
        ),
        (WENDAO_ROUTE_MODALITY_HEADER, intent.modality.clone()),
        (
            WENDAO_ROUTE_SELECTED_PROVIDER_HEADER,
            decision.selected_provider.clone(),
        ),
        (
            WENDAO_ROUTE_SELECTED_MODEL_HEADER,
            decision.selected_model.clone(),
        ),
        (
            WENDAO_ROUTE_SELECTED_BACKEND_PROFILE_HEADER,
            decision.selected_backend_profile.clone(),
        ),
        (
            WENDAO_ROUTE_PRECISION_TIER_HEADER,
            intent.precision_tier.clone(),
        ),
    ]
}

fn required_string(value: &serde_json::Value, keys: &[&str]) -> Result<String, String> {
    optional_string(value, keys).ok_or_else(|| {
        format!(
            "vLLM-SR decision response missing required field `{}`",
            keys[0]
        )
    })
}

fn optional_string(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    })
}

fn normalize_base_url(value: &str) -> String {
    let trimmed = value.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        DEFAULT_WENDAO_VLLM_SR_BASE_URL.to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn header_string(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn response_body_selected_model(response_body: &str) -> Option<String> {
    let value = serde_json::from_str::<serde_json::Value>(response_body).ok()?;
    optional_string(&value, &["model", "selected_model", "selectedModel"])
}

fn build_vllm_sr_route_id(selected_decision: Option<&str>, selected_model: &str) -> String {
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

fn build_vllm_sr_route_trace(
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
