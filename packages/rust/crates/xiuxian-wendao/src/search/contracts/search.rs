use serde::{Deserialize, Serialize};
use specta::Type;

use super::StudioNavigationTarget;

/// A single hit in a knowledge base search.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSearchHit {
    /// Global node identifier.
    pub id: String,
    /// Display label.
    pub label: String,
    /// File path.
    pub path: String,
    /// Navigation target.
    pub navigation_target: StudioNavigationTarget,
    /// Semantic score (0.0 - 1.0).
    pub score: f64,
    /// Snippet highlighting matching terms.
    pub snippet: String,
}

/// Structured backlink metadata surfaced on search hits.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchBacklinkItem {
    /// Stable backlink identifier.
    pub id: String,
    /// Optional display title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Optional source path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Optional relation kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

/// Unified search hit consumed by the frontend search surface.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    /// Stable stem or primary identifier.
    pub stem: String,
    /// Optional display title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Repository-relative or workspace-relative path.
    pub path: String,
    /// Optional logical hit kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_type: Option<String>,
    /// Search-visible tags.
    pub tags: Vec<String>,
    /// Normalized score.
    pub score: f64,
    /// Optional best section or signature summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub best_section: Option<String>,
    /// Optional match-reason string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_reason: Option<String>,
    /// Optional hierarchical URI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hierarchical_uri: Option<String>,
    /// Optional hierarchy segments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hierarchy: Option<Vec<String>>,
    /// Optional saliency score.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub saliency_score: Option<f64>,
    /// Optional audit status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_status: Option<String>,
    /// Optional verification state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_state: Option<String>,
    /// Optional backlink identifiers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implicit_backlinks: Option<Vec<String>>,
    /// Optional structured backlink items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implicit_backlink_items: Option<Vec<SearchBacklinkItem>>,
    /// Optional navigation target.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub navigation_target: Option<StudioNavigationTarget>,
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
    pub selected_mode: Option<String>,
    /// Optional resolved intent label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    /// Optional resolved intent confidence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent_confidence: Option<f64>,
    /// Optional backend search mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_mode: Option<String>,
    /// Whether the backend returned partial results because repo indexes are still warming or
    /// because a repo-wide search exhausted its bounded server-side budget.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub partial: bool,
    /// Optional aggregate indexing state for code search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indexing_state: Option<String>,
    /// Repo ids that are still queued or indexing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pending_repos: Vec<String>,
    /// Repo ids skipped because their repo index is unsupported or failed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skipped_repos: Vec<String>,
}

/// A hit derived from search intent hints (e.g., task-oriented).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct IntentSearchHit {
    /// Display label for the intent.
    pub label: String,
    /// Target semantic action.
    pub action: String,
    /// Score indicating intent alignment.
    pub score: f64,
}
