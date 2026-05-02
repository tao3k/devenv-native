//! HTTP gateway shared state and response types.

use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::ToolListCacheStatsSnapshot;
use crate::agent::Agent;
use crate::embedding::EmbeddingClient;

/// Request body for POST /message.
#[derive(Debug, Deserialize)]
pub struct MessageRequest {
    /// Conversation session identifier.
    pub session_id: String,
    /// User message to send to the agent.
    pub message: String,
}

/// Response body.
#[derive(Debug, Serialize)]
pub struct MessageResponse {
    /// Agent reply (text output).
    pub output: String,
    /// Session identifier (echo of request).
    pub session_id: String,
}

/// Shared state for the HTTP server: agent + per-turn timeout + optional concurrency limit.
#[derive(Clone)]
pub struct GatewayState {
    /// Shared agent runtime used by gateway request handlers.
    pub agent: Arc<Agent>,
    /// Maximum wall-clock timeout (seconds) for each turn.
    pub turn_timeout_secs: u64,
    /// When Some, limits concurrent agent turns; excess requests wait for a slot.
    pub concurrency_semaphore: Option<Arc<Semaphore>>,
    /// Configured max concurrency used for health reporting.
    pub max_concurrent_turns: Option<usize>,
    /// Embedding runtime used by embedding endpoints.
    pub embedding_runtime: Arc<GatewayEmbeddingRuntime>,
}

/// Shared embedding runtime state for gateway embedding handlers.
#[derive(Clone)]
pub struct GatewayEmbeddingRuntime {
    /// Embedding client used to execute vectorization requests.
    pub client: Arc<EmbeddingClient>,
    /// Optional default embedding model used when request omits `model`.
    pub default_model: Option<String>,
    /// Fallback embedding dimension used by deterministic hash encoder when upstream fails.
    pub fallback_embedding_dim: usize,
}

/// Request body for batch embedding endpoints.
#[derive(Debug, Deserialize)]
pub struct EmbedBatchRequest {
    /// Input texts to embed.
    pub texts: Vec<String>,
    /// Optional model override.
    #[serde(default)]
    pub model: Option<String>,
}

/// Response body containing vectors for each input text.
#[derive(Debug, Serialize)]
pub struct EmbedBatchResponse {
    /// Embedding vectors aligned with request order.
    pub vectors: Vec<Vec<f32>>,
}

/// Request body for single-text embedding endpoint.
#[derive(Debug, Deserialize)]
pub struct EmbedRequest {
    /// Input text to embed.
    pub text: String,
    /// Optional model override.
    #[serde(default)]
    pub model: Option<String>,
}

/// Response body for single-text embedding endpoint.
#[derive(Debug, Serialize)]
pub struct EmbedResponse {
    /// Embedding vector for the input text.
    pub vector: Vec<f32>,
}

/// OpenAI-compatible embeddings request payload.
#[derive(Debug, Deserialize)]
pub struct OpenAiEmbeddingsRequest {
    /// Input accepted as either string or array payload.
    pub input: serde_json::Value,
    /// Optional model override.
    #[serde(default)]
    pub model: Option<String>,
}

/// OpenAI-compatible embeddings response payload.
#[derive(Debug, Serialize)]
pub struct OpenAiEmbeddingsResponse {
    /// Object type identifier (e.g. `list`).
    pub object: &'static str,
    /// Effective model name used to produce embeddings.
    pub model: String,
    /// Embedding rows in request order.
    pub data: Vec<OpenAiEmbeddingData>,
    /// Token usage accounting for the request.
    pub usage: OpenAiEmbeddingUsage,
}

/// One embedding row in OpenAI-compatible response payload.
#[derive(Debug, Serialize)]
pub struct OpenAiEmbeddingData {
    /// Object type identifier (e.g. `embedding`).
    pub object: &'static str,
    /// Zero-based index matching input order.
    pub index: usize,
    /// Embedding vector values.
    pub embedding: Vec<f32>,
}

/// Token usage details in OpenAI-compatible embedding response.
#[derive(Debug, Serialize)]
pub struct OpenAiEmbeddingUsage {
    /// Prompt token estimate consumed by the request.
    pub prompt_tokens: usize,
    /// Total token estimate consumed by the request.
    pub total_tokens: usize,
}

/// External tool runtime section in gateway health response.
#[derive(Debug, Serialize)]
pub struct GatewayExternalToolHealthResponse {
    /// Whether external tool integration is enabled in this runtime.
    pub enabled: bool,
    /// Optional discover/tools cache stats snapshot.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools_list_cache: Option<ToolListCacheStatsSnapshot>,
}

/// Response body for gateway health endpoint.
#[derive(Debug, Serialize)]
pub struct GatewayHealthResponse {
    /// Overall gateway readiness status.
    pub status: &'static str,
    /// Effective per-turn timeout in seconds.
    pub turn_timeout_secs: u64,
    /// Optional max concurrency configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_concurrent_turns: Option<usize>,
    /// Optional current in-flight turns.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_flight_turns: Option<usize>,
    /// External tool runtime health sub-section.
    pub tools: GatewayExternalToolHealthResponse,
}

/// Standard JSON error payload tuple for gateway handlers.
pub type GatewayJsonError = (axum::http::StatusCode, String);
/// Standard JSON result alias for gateway handlers.
pub type GatewayJsonResult<T> = Result<Json<T>, GatewayJsonError>;
