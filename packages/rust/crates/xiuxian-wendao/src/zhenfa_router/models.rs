//! `zhenfa_router::models` owns Wendao zhenfa router models behavior.

use serde::{Deserialize, Serialize};

use crate::LinkGraphSearchOptions;

/// Optional result shape for Wendao search output.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WendaoSearchResponseFormat {
    /// Human-readable markdown optimized for LLM context windows.
    #[default]
    Markdown,
    /// Canonical JSON string payload.
    Json,
}

/// Structured request payload for Wendao search routing.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WendaoSearchRequest {
    /// Query string with optional directives.
    pub query: String,
    /// Optional precomputed query embedding used by semantic ignition and
    /// Julia rerank when the caller already owns the vectorization step.
    #[serde(default)]
    pub query_vector: Option<Vec<f32>>,
    /// Maximum number of hits requested.
    #[serde(default)]
    pub limit: Option<usize>,
    /// Notebook root directory used for `LinkGraph` index construction.
    #[serde(default)]
    pub root_dir: Option<String>,
    /// Base query options merged with parsed query directives.
    #[serde(default)]
    pub options: Option<LinkGraphSearchOptions>,
    /// Whether provisional suggested links should be included.
    #[serde(default)]
    pub include_provisional: Option<bool>,
    /// Provisional suggestion cap when enabled.
    #[serde(default)]
    pub provisional_limit: Option<usize>,
    /// Result render shape returned to caller.
    #[serde(default)]
    pub response_format: WendaoSearchResponseFormat,
}

/// HTTP response envelope for direct Wendao search endpoint.
#[cfg(feature = "zhenfa-router")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WendaoSearchHttpResponse {
    /// Pre-rendered search result text.
    pub result: String,
}
