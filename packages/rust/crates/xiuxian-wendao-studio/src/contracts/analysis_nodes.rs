//! Studio-owned document analysis node contracts.

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

#[cfg(feature = "local-runtime")]
impl From<xiuxian_wendao::search::contracts::AnalysisNodeKind> for AnalysisNodeKind {
    fn from(value: xiuxian_wendao::search::contracts::AnalysisNodeKind) -> Self {
        match value {
            xiuxian_wendao::search::contracts::AnalysisNodeKind::Section => Self::Section,
            xiuxian_wendao::search::contracts::AnalysisNodeKind::Task => Self::Task,
            xiuxian_wendao::search::contracts::AnalysisNodeKind::Observation => Self::Observation,
            xiuxian_wendao::search::contracts::AnalysisNodeKind::Relation => Self::Relation,
            xiuxian_wendao::search::contracts::AnalysisNodeKind::Document => Self::Document,
            xiuxian_wendao::search::contracts::AnalysisNodeKind::CodeBlock => Self::CodeBlock,
            xiuxian_wendao::search::contracts::AnalysisNodeKind::Table => Self::Table,
            xiuxian_wendao::search::contracts::AnalysisNodeKind::Math => Self::Math,
            xiuxian_wendao::search::contracts::AnalysisNodeKind::Reference => Self::Reference,
            xiuxian_wendao::search::contracts::AnalysisNodeKind::Property => Self::Property,
            xiuxian_wendao::search::contracts::AnalysisNodeKind::Symbol => Self::Symbol,
        }
    }
}

#[cfg(feature = "local-runtime")]
impl From<xiuxian_wendao::search::contracts::AnalysisNode> for AnalysisNode {
    fn from(value: xiuxian_wendao::search::contracts::AnalysisNode) -> Self {
        Self {
            id: value.id,
            kind: value.kind.into(),
            label: value.label,
            depth: value.depth,
            line_start: value.line_start,
            line_end: value.line_end,
            parent_id: value.parent_id,
        }
    }
}
