//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
//! `search::contracts::search` owns Wendao search contracts search behavior.

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
/// Stringly state boundary: this public record preserves serialized catalog tokens from external or stored Wendao data.
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
/// Stringly state boundary: this public record preserves serialized catalog tokens from external or stored Wendao data.
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
