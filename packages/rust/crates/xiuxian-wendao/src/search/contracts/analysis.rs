use serde::{Deserialize, Serialize};
use specta::Type;

use super::retrieval::RetrievalChunk;

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

/// Kind of an analysis edge.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AnalysisEdgeKind {
    /// Parent-child hierarchy.
    Parent,
    /// Semantic reference or mention.
    Mentions,
    /// Document membership.
    Contains,
    /// Next task in sequence.
    NextStep,
    /// Explicit document reference.
    References,
}

/// Metadata about an analysis edge evidence.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisEvidence {
    /// Evidence file path.
    pub path: String,
    /// 1-based start line.
    pub line_start: usize,
    /// 1-based end line.
    pub line_end: usize,
    /// Confidence score (0.0 - 1.0).
    pub confidence: f64,
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

/// A relationship edge in the document IR.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisEdge {
    /// Edge identifier.
    pub id: String,
    /// Source node identifier.
    pub source_id: String,
    /// Target node identifier.
    pub target_id: String,
    /// Relationship kind.
    pub kind: AnalysisEdgeKind,
    /// Display label.
    pub label: String,
    /// Evidence metadata.
    pub evidence: AnalysisEvidence,
}

/// Shared retrieval chunk used by markdown analysis surfaces.
pub type MarkdownRetrievalAtom = RetrievalChunk;

/// `DeepWiki`-style document link kind emitted by markdown analysis.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MarkdownAnalysisDocumentLinkKind {
    /// Ordinary structural body wikilink.
    Body,
    /// Explicit semantic relation from metadata.
    Relation,
    /// Docs-kernel `:RELATIONS: :LINKS:` edge.
    Index,
    /// Resolved parent document edge.
    Parent,
    /// Materialized backlink row from the reverse index.
    Backlink,
}

/// One `DeepWiki`-style document link row.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MarkdownAnalysisDocumentLink {
    /// Display label shown in the reader.
    pub label: String,
    /// Coarse source kind for the row.
    pub kind: MarkdownAnalysisDocumentLinkKind,
    /// Original literal token when it exists in source markdown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub literal: Option<String>,
    /// Explicit semantic relation type for metadata-owned links.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relation_type: Option<String>,
    /// Owning metadata scope for explicit relation rows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_owner: Option<String>,
    /// Canonical target/source document id when resolution succeeds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_id: Option<String>,
    /// Studio-display path fallback for reader navigation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Best-effort resolved title for the linked document.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Optional scoped target suffix such as `#Heading` or `#^block`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_address: Option<String>,
}

/// `DeepWiki`-style document identity payload emitted by markdown analysis.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MarkdownAnalysisDocumentMetadata {
    /// Canonical document id when the graph index can resolve the file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_id: Option<String>,
    /// Parser-owned document title.
    pub title: String,
    /// Parser-owned document tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Optional parser-owned semantic document type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_type: Option<String>,
    /// Best-effort updated timestamp from raw metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
    /// Explicit docs-kernel parent link, when declared.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<MarkdownAnalysisDocumentLink>,
    /// Unified outgoing relation/index rows.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outgoing_links: Vec<MarkdownAnalysisDocumentLink>,
    /// Canonical Obsidian/ZK-aligned explicit incoming note/document links.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub explicit_backlinks: Vec<MarkdownAnalysisDocumentLink>,
    /// Legacy compatibility alias for explicit incoming note/document links.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub backlinks: Vec<MarkdownAnalysisDocumentLink>,
}

/// Full response for Markdown analysis.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MarkdownAnalysisResponse {
    /// Analyzed file path.
    pub path: String,
    /// Content fingerprint.
    pub document_hash: String,
    /// Total number of nodes.
    pub node_count: usize,
    /// Total number of edges.
    pub edge_count: usize,
    /// IR nodes.
    pub nodes: Vec<AnalysisNode>,
    /// IR edges.
    pub edges: Vec<AnalysisEdge>,
    /// Mermaid diagram projections.
    pub projections: Vec<MermaidProjection>,
    /// Compact retrieval atoms for document / section surfaces.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retrieval_atoms: Vec<MarkdownRetrievalAtom>,
    /// Backend-owned `DeepWiki` document identity and link metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document_metadata: Option<MarkdownAnalysisDocumentMetadata>,
    /// Analysis diagnostics.
    pub diagnostics: Vec<String>,
}

/// Mermaid projection view kind.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum MermaidViewKind {
    /// Hierarchical document outline.
    Outline,
    /// Task dependency graph.
    Tasks,
    /// Semantic entity relations.
    Knowledge,
}

/// A single Mermaid diagram projection.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MermaidProjection {
    /// Type of projection.
    pub kind: MermaidViewKind,
    /// Generated Mermaid source.
    pub source: String,
    /// Number of nodes in projection.
    pub node_count: usize,
    /// Number of edges in projection.
    pub edge_count: usize,
}
