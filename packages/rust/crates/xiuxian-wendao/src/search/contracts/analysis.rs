//! `search::contracts::analysis` owns Wendao search contracts analysis behavior.

use serde::{Deserialize, Serialize};
use specta::Type;

/// Kind of an analysis node.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AnalysisNodeKind {
    /// Markdown section heading.
    Section,
    /// Task list item.
    Task,
    /// Observation/evidence block.
    Observation,
    /// Symbolic link or relation.
    Relation,
    /// Document-level node.
    Document,
    /// Code block node.
    CodeBlock,
    /// Markdown table node.
    Table,
    /// Display math node.
    Math,
    /// Semantic reference site.
    Reference,
    /// Property box node.
    Property,
    /// Symbolic entity node.
    Symbol,
}

/// A single node in the structural IR of a document.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisNode {
    /// Node identifier.
    pub id: String,
    /// Node kind.
    pub kind: AnalysisNodeKind,
    /// Display label.
    pub label: String,
    /// Nesting depth.
    pub depth: usize,
    /// 1-based start line.
    pub line_start: usize,
    /// 1-based end line.
    pub line_end: usize,
    /// Optional parent node identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}
