use serde::{Deserialize, Serialize};
use specta::Type;

use super::StudioNavigationTarget;

/// A single hit in an AST definition search.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AstSearchHit {
    /// Captured definition name.
    pub name: String,
    /// Signature line or skeleton snippet.
    pub signature: String,
    /// Source file path relative to the project root.
    pub path: String,
    /// Source language name.
    pub language: String,
    /// Owning crate or package name.
    pub crate_name: String,
    /// Configured project name when the source path maps to a studio project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
    /// Configured root label when the source path maps to a project root path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_label: Option<String>,
    /// Optional AST node kind for richer Markdown search presentation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_kind: Option<String>,
    /// Optional owning Markdown section title/path for property-box derived hits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_title: Option<String>,
    /// Display-ready navigation target for opening this hit in studio.
    pub navigation_target: StudioNavigationTarget,
    /// 1-based start line.
    pub line_start: usize,
    /// 1-based end line.
    pub line_end: usize,
    /// Search relevance score.
    pub score: f64,
}

/// Result of a best-definition resolution.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DefinitionSearchHit {
    /// Symbol or definition name.
    pub name: String,
    /// Display signature for the definition.
    pub signature: String,
    /// Repository-relative path to the definition.
    pub path: String,
    /// Source language label for the definition.
    pub language: String,
    /// Owning crate or repository identifier.
    pub crate_name: String,
    /// Optional project grouping label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
    /// Optional root label derived from configured project scopes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_label: Option<String>,
    /// Optional AST node kind for the resolved symbol.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_kind: Option<String>,
    /// Optional owner title or containing symbol label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_title: Option<String>,
    /// Navigation target for opening the definition in Studio.
    pub navigation_target: StudioNavigationTarget,
    /// 1-based starting line for the definition span.
    pub line_start: usize,
    /// 1-based ending line for the definition span.
    pub line_end: usize,
    /// Resolution score assigned to this candidate.
    pub score: f64,
    /// Hints derived from :OBSERVE: property boxes.
    pub observation_hints: Vec<ObservationHint>,
}

/// A hint for observing code patterns near a definition.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ObservationHint {
    /// Language constraint (e.g., "rust").
    pub language: String,
    /// File path scope (e.g., "src/**").
    pub scope: String,
    /// Pattern to observe.
    pub pattern: String,
}

/// Response for studio AST definition search queries.
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
    pub indexing_state: Option<String>,
    /// Optional AST-index error surfaced without blocking the request path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index_error: Option<String>,
}

/// Response for native studio definition resolution.
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

/// A hit indicating where a symbol is referenced or used.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSearchHit {
    /// Symbol name being referenced.
    pub name: String,
    /// Referencing file path.
    pub path: String,
    /// Language of the referencing file.
    pub language: String,
    /// Crate name of the referencing file.
    pub crate_name: String,
    /// Project grouping label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
    /// Root label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_label: Option<String>,
    /// Navigation target for the reference site.
    pub navigation_target: StudioNavigationTarget,
    /// 1-based line number.
    pub line: usize,
    /// 1-based column number.
    pub column: usize,
    /// Snippet showing matching line.
    pub line_text: String,
    /// Scoring weight.
    pub score: f64,
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
    pub indexing_state: Option<String>,
    /// Optional reference-index error surfaced without blocking the request path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index_error: Option<String>,
}
