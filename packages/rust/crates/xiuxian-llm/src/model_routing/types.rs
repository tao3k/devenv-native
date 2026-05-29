//! Model routing types — task/source kind tokens, routing mode, route intent and decision.

use std::fmt;

use reqwest::header::HeaderMap;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, Visitor},
};

use super::constants::{
    DEFAULT_WENDAO_MODEL_ROUTING_MODE, DEFAULT_WENDAO_VLLM_SR_BASE_URL, VLLM_SR_REQUEST_ID_HEADER,
    VLLM_SR_SELECTED_DECISION_HEADER, VLLM_SR_SELECTED_MODEL_HEADER,
    VLLM_SR_SELECTED_REASONING_HEADER, WENDAO_MODEL_ROUTING_MODE_ENV, WENDAO_VLLM_SR_BASE_URL_ENV,
};
use super::route_helpers::{
    build_vllm_sr_route_id, build_vllm_sr_route_trace, header_string, optional_string,
    required_string, response_body_selected_model,
};

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

impl Serialize for WendaoModelRoutingMode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(match self {
            Self::VllmSr => "vllm-sr",
            Self::Deterministic => "deterministic",
        })
    }
}

struct WendaoModelRoutingModeVisitor;

impl Visitor<'_> for WendaoModelRoutingModeVisitor {
    type Value = WendaoModelRoutingMode;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("`deterministic` or `vllm-sr`")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<WendaoModelRoutingMode, E> {
        WendaoModelRoutingMode::parse(value).map_err(de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for WendaoModelRoutingMode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(WendaoModelRoutingModeVisitor)
    }
}

impl WendaoModelRoutingMode {
    /// Return the routing mode as a stable string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::VllmSr => "vllm-sr",
            Self::Deterministic => "deterministic",
        }
    }

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
