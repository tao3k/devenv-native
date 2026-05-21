//! Studio-owned search response wrapper contracts.

use serde::{Deserialize, Serialize};
use specta::Type;

use super::{
    AstSearchHit, AttachmentSearchHit, DefinitionSearchHit, ReferenceSearchHit, SearchHit,
};
use crate::contracts::{StudioContractMode, StudioContractState, StudioNavigationTarget};

/// Response for Studio attachment search queries.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentSearchResponse {
    /// Original query string.
    pub query: String,
    /// Matching attachment hits.
    pub hits: Vec<AttachmentSearchHit>,
    /// Total number of hits returned.
    pub hit_count: usize,
    /// Selected attachment scope label.
    pub selected_scope: String,
    /// Whether the response is partial because the attachment index is still warming.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub partial: bool,
    /// Current attachment-index lifecycle state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indexing_state: Option<StudioContractState>,
    /// Optional attachment-index error surfaced without blocking the request path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index_error: Option<String>,
}

/// Response for Studio AST definition search queries.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AstSearchResponse {
    /// Original query string.
    pub query: String,
    /// Matching AST hits.
    pub hits: Vec<AstSearchHit>,
    /// Total number of hits returned.
    pub hit_count: usize,
    /// Selected AST scope.
    pub selected_scope: String,
    /// Whether the response is partial because the AST index is still warming.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub partial: bool,
    /// Current AST-index lifecycle state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indexing_state: Option<StudioContractState>,
    /// Optional AST-index error surfaced without blocking the request path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index_error: Option<String>,
}

/// Response for native Studio definition resolution.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DefinitionResolveResponse {
    /// Original query string.
    pub query: String,
    /// Optional source path used to bias resolution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    /// Optional source line used by the caller.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_line: Option<usize>,
    /// Number of candidate definitions considered for this resolution.
    pub candidate_count: usize,
    /// The selected scope used to resolve the definition.
    pub selected_scope: String,
    /// Display-ready navigation target for the resolved definition.
    pub navigation_target: StudioNavigationTarget,
    /// The resolved definition hit.
    pub definition: DefinitionSearchHit,
    /// Display-ready navigation target for the resolved definition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_target: Option<StudioNavigationTarget>,
    /// The actual hit that was resolved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_hit: Option<DefinitionSearchHit>,
}

/// Response for Studio reference search queries.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSearchResponse {
    /// Original query string.
    pub query: String,
    /// Matching reference hits.
    pub hits: Vec<ReferenceSearchHit>,
    /// Total number of hits returned.
    pub hit_count: usize,
    /// Selected reference scope label.
    pub selected_scope: String,
    /// Whether the response is partial because the reference index is still warming.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub partial: bool,
    /// Current reference-index lifecycle state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indexing_state: Option<StudioContractState>,
    /// Optional reference-index error surfaced without blocking the request path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index_error: Option<String>,
}

/// Unified search response consumed by the frontend search shell.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    /// Original query string.
    pub query: String,
    /// Matching hits.
    pub hits: Vec<SearchHit>,
    /// Total number of hits returned.
    pub hit_count: usize,
    /// Optional graph confidence score.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_confidence_score: Option<f64>,
    /// Optional selected mode label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_mode: Option<StudioContractMode>,
    /// Optional resolved intent label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    /// Optional resolved intent confidence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent_confidence: Option<f64>,
    /// Optional backend search mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_mode: Option<StudioContractMode>,
    /// Whether the backend returned partial results because repo indexes are still warming or
    /// because a repo-wide search exhausted its bounded server-side budget.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub partial: bool,
    /// Optional aggregate indexing state for code search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indexing_state: Option<StudioContractState>,
    /// Repo ids that are still queued or indexing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pending_repos: Vec<String>,
    /// Repo ids skipped because their repo index is unsupported or failed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skipped_repos: Vec<String>,
}
