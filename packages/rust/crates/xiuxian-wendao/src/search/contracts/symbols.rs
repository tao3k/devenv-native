use serde::{Deserialize, Serialize};
use specta::Type;

use super::StudioNavigationTarget;

/// A hit in a project-wide symbol index (e.g. Tantivy-backed).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SymbolSearchHit {
    /// Symbol name.
    pub name: String,
    /// Symbol kind (e.g. "fn", "struct").
    pub kind: String,
    /// Display path.
    pub path: String,
    /// 1-based line number for the symbol location.
    pub line: usize,
    /// Canonical `path:line` location string.
    pub location: String,
    /// Source language label inferred from the symbol path.
    pub language: String,
    /// Source identifier (e.g. "project", "external").
    pub source: String,
    /// Owning crate name.
    pub crate_name: String,
    /// Project grouping label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
    /// Root label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_label: Option<String>,
    /// Navigation target.
    pub navigation_target: StudioNavigationTarget,
    /// Semantic score.
    pub score: f64,
}

/// Response for Studio symbol search queries.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SymbolSearchResponse {
    /// Original query string.
    pub query: String,
    /// Matching symbol hits.
    pub hits: Vec<SymbolSearchHit>,
    /// Total number of hits returned.
    pub hit_count: usize,
    /// Selected symbol scope label.
    pub selected_scope: String,
    /// Whether the response is partial because the symbol index is still warming.
    pub partial: bool,
    /// Current symbol-index lifecycle state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indexing_state: Option<String>,
    /// Optional symbol-index error surfaced without blocking the request path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index_error: Option<String>,
}

/// A suggested autocomplete entry.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AutocompleteHit {
    /// Suggestion text.
    pub label: String,
    /// Category classification for icons.
    pub category: String,
    /// Optional snippet or description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// A single autocomplete suggestion.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AutocompleteSuggestion {
    /// Suggestion text emitted to the caller.
    pub text: String,
    /// Logical suggestion classification.
    pub suggestion_type: String,
}

/// Response for Studio autocomplete queries.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AutocompleteResponse {
    /// Original prefix used to generate suggestions.
    pub prefix: String,
    /// Ranked autocomplete suggestions.
    pub suggestions: Vec<AutocompleteSuggestion>,
}
