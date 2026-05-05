//! Serializable DTOs for local docs page-index projections.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Deterministic page family kind for one projected or parsed document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub enum ProjectionPageKind {
    /// Reference-style document.
    Reference,
    /// Task-oriented how-to guide.
    HowTo,
    /// Step-by-step tutorial document.
    Tutorial,
    /// Explanatory or conceptual document.
    #[default]
    Explanation,
}

/// One page-index section summary in a parsed markdown document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct ProjectedPageIndexSection {
    /// Heading path identifier.
    pub heading_path: String,
    /// Section title.
    pub title: String,
    /// Heading level (1-6).
    pub level: usize,
    /// Start and end line numbers.
    pub line_range: (usize, usize),
    /// Key-value attributes for the section.
    pub attributes: Vec<(String, String)>,
}

/// Parsed page-index-ready document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct ProjectedPageIndexDocument {
    /// Scope identifier projected.
    pub repo_id: String,
    /// Stable page identifier.
    pub page_id: String,
    /// File path.
    pub path: String,
    /// Stable document identifier.
    pub doc_id: String,
    /// Page title.
    pub title: String,
    /// Parsed sections from the document.
    pub sections: Vec<ProjectedPageIndexSection>,
}

/// Snapshot of one page-index node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct ProjectedPageIndexNode {
    /// Node identifier.
    pub node_id: String,
    /// Node title.
    pub title: String,
    /// Heading level.
    pub level: usize,
    /// Structural path from root to this node.
    pub structural_path: Vec<String>,
    /// Start and end line numbers.
    pub line_range: (usize, usize),
    /// Approximate token count for this node.
    pub token_count: usize,
    /// Whether this node has been thinned during summarization.
    pub is_thinned: bool,
    /// Full text content of the node.
    pub text: String,
    /// Optional summary of the node content.
    pub summary: Option<String>,
    /// Link-like targets referenced inside this node section.
    #[serde(default)]
    pub links: Vec<ProjectedPageIndexLink>,
    /// Child nodes.
    pub children: Vec<ProjectedPageIndexNode>,
}

/// Compact link-like target summary attached to one page-index node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct ProjectedPageIndexLink {
    /// Parser-visible link occurrence kind.
    pub kind: String,
    /// Parser-visible target string.
    pub target: String,
    /// Parser-visible source syntax for the occurrence.
    #[serde(default)]
    pub surface: String,
}

/// Snapshot of one page-index tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct ProjectedPageIndexTree {
    /// Scope identifier projected.
    pub repo_id: String,
    /// Stable page identifier.
    pub page_id: String,
    /// Page kind.
    pub kind: ProjectionPageKind,
    /// File path.
    pub path: String,
    /// Stable document identifier.
    pub doc_id: String,
    /// Page title.
    pub title: String,
    /// Number of root nodes.
    pub root_count: usize,
    /// Root nodes of the tree.
    pub roots: Vec<ProjectedPageIndexNode>,
}

/// Deterministic TOC/page-index documents result set for one local scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct DocsPageIndexDocumentsResult {
    /// Scope identifier projected.
    pub repo_id: String,
    /// Parsed page-index-ready documents derived from target truth.
    pub documents: Vec<ProjectedPageIndexDocument>,
}

/// Deterministic text-free page-index trees result set for one local scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct DocsPageIndexTreesResult {
    /// Scope identifier projected.
    pub repo_id: String,
    /// Parsed text-free page-index trees derived from target truth.
    pub trees: Vec<ProjectedPageIndexTree>,
}
