use serde::{Deserialize, Serialize};
use specta::Type;

use super::retrieval::{RetrievalChunk, RetrievalChunkSurface};

/// Kind of a code-AST node.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodeAstNodeKind {
    /// Module/namespace container.
    Module,
    /// Function/method declaration.
    Function,
    /// Type/struct/class declaration.
    Type,
    /// Constant declaration.
    Constant,
    /// External symbol imported from outside the file.
    ExternalSymbol,
    /// Other AST entities.
    Other,
}

/// Kind of a code-AST edge.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CodeAstEdgeKind {
    /// Ownership / nesting relation.
    Contains,
    /// Call relation.
    Calls,
    /// Usage relation.
    Uses,
    /// Import relation.
    Imports,
    /// Other relation.
    Other,
}

/// Kind of an AST projection view.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CodeAstProjectionKind {
    /// Containment projection.
    Contains,
    /// Call-graph projection.
    Calls,
    /// Usage projection.
    Uses,
}

/// Surface kind for a retrieval atom derived from the code-AST response.
pub type CodeAstRetrievalAtomScope = RetrievalChunkSurface;

/// A single AST node entry for diagram rendering.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CodeAstNode {
    /// Node identifier.
    pub id: String,
    /// Display label.
    pub label: String,
    /// Semantic node kind.
    pub kind: CodeAstNodeKind,
    /// Optional source path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Optional 1-based source start line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_start: Option<usize>,
    /// Optional 1-based source end line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_end: Option<usize>,
}

/// A single AST edge entry for diagram rendering.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CodeAstEdge {
    /// Edge identifier.
    pub id: String,
    /// Source node identifier.
    pub source_id: String,
    /// Target node identifier.
    pub target_id: String,
    /// Semantic edge kind.
    pub kind: CodeAstEdgeKind,
    /// Optional display label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Precomputed AST projection metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CodeAstProjection {
    /// Projection category.
    pub kind: CodeAstProjectionKind,
    /// Number of nodes included in projection.
    pub node_count: usize,
    /// Number of edges included in projection.
    pub edge_count: usize,
}

/// Shared retrieval chunk used by code-AST analysis surfaces.
pub type CodeAstRetrievalAtom = RetrievalChunk;

/// Response payload for code AST analysis.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CodeAstAnalysisResponse {
    /// Repository identifier.
    pub repo_id: String,
    /// Repository-relative source path.
    pub path: String,
    /// Source language.
    pub language: String,
    /// Total number of AST nodes.
    pub node_count: usize,
    /// Total number of AST edges.
    pub edge_count: usize,
    /// AST nodes.
    pub nodes: Vec<CodeAstNode>,
    /// AST edges.
    pub edges: Vec<CodeAstEdge>,
    /// Projection summaries.
    pub projections: Vec<CodeAstProjection>,
    /// Compact retrieval atoms for declaration- and symbol-backed surfaces.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retrieval_atoms: Vec<CodeAstRetrievalAtom>,
    /// Optional node identifier selected by line hint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus_node_id: Option<String>,
    /// Diagnostics emitted by parser/analyzer.
    pub diagnostics: Vec<String>,
}
