//! Studio-owned search hit contracts.

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::contracts::{
    StudioContractDocType, StudioContractId, StudioContractKind, StudioContractNodeKind,
    StudioContractPath, StudioContractState, StudioContractStatus, StudioContractTag,
    StudioNavigationTarget,
};

/// A hit representing an attachment or external resource.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentSearchHit {
    /// Attachment filename.
    pub name: String,
    /// Relative path.
    pub path: StudioContractPath,
    /// Stable source document identifier.
    pub source_id: StudioContractId,
    /// Source document stem.
    pub source_stem: String,
    /// Source document title.
    pub source_title: String,
    /// Source document path.
    pub source_path: StudioContractPath,
    /// Stable attachment identifier.
    pub attachment_id: StudioContractId,
    /// Relative attachment path.
    pub attachment_path: StudioContractPath,
    /// Attachment display name.
    pub attachment_name: String,
    /// Lowercased attachment extension without leading dot.
    pub attachment_ext: String,
    /// Attachment kind label.
    pub kind: StudioContractKind,
    /// Navigation target.
    pub navigation_target: StudioNavigationTarget,
    /// Relevance score.
    pub score: f64,
    /// Optional OCR or vision snippet for the attachment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vision_snippet: Option<String>,
}

/// A local source-symbol search hit used by definition and intent recovery.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SourceSymbolHit {
    /// Captured symbol or heading name.
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
    /// Optional source node kind for richer Markdown search presentation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_kind: Option<StudioContractNodeKind>,
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
    pub node_kind: Option<StudioContractNodeKind>,
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
    /// Hints derived from `:OBSERVE:` property boxes.
    pub observation_hints: Vec<ObservationHint>,
}

/// A hint for observing code patterns near a definition.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ObservationHint {
    /// Language constraint, such as `rust`.
    pub language: String,
    /// File path scope, such as `src/**`.
    pub scope: String,
    /// Pattern to observe.
    pub pattern: String,
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

/// A single hit in a knowledge base search.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSearchHit {
    /// Global node identifier.
    pub id: StudioContractId,
    /// Display label.
    pub label: String,
    /// File path.
    pub path: String,
    /// Navigation target.
    pub navigation_target: StudioNavigationTarget,
    /// Semantic score.
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
    pub path: Option<StudioContractPath>,
    /// Optional relation kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<StudioContractKind>,
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
    pub path: StudioContractPath,
    /// Optional logical hit kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_type: Option<StudioContractDocType>,
    /// Search-visible tags.
    pub tags: Vec<StudioContractTag>,
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
    pub audit_status: Option<StudioContractStatus>,
    /// Optional verification state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_state: Option<StudioContractState>,
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

/// A hit derived from search intent hints.
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
