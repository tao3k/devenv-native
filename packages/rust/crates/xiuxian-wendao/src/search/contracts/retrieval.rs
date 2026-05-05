use serde::{Deserialize, Serialize};
use specta::Type;

/// Surface kind for a shared retrieval chunk.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RetrievalChunkSurface {
    /// Markdown document identity card.
    Document,
    /// Markdown section card.
    Section,
    /// Markdown code / mermaid rich slot.
    CodeBlock,
    /// Markdown table rich slot.
    Table,
    /// Markdown display-math rich slot.
    Math,
    /// Markdown observation / blockquote rich slot.
    Observation,
    /// Code declaration surface.
    Declaration,
    /// Code logic-block surface.
    Block,
    /// Code symbol / anchor surface.
    Symbol,
}

/// Shared retrieval chunk contract across markdown and code analysis surfaces.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalChunk {
    /// Owning node identifier.
    pub owner_id: String,
    /// Stable retrieval chunk identifier.
    pub chunk_id: String,
    /// Semantic type for downstream retrieval / UI display.
    pub semantic_type: String,
    /// Stable semantic fingerprint.
    pub fingerprint: String,
    /// Approximate token estimate.
    pub token_estimate: usize,
    /// Optional display label for UI-facing retrieval rails.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_label: Option<String>,
    /// Optional excerpt for UI-facing retrieval rails.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    /// Optional 1-based start line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_start: Option<usize>,
    /// Optional 1-based end line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_end: Option<usize>,
    /// Optional surface kind for richer UI routing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface: Option<RetrievalChunkSurface>,
    /// Optional parser- or compiler-owned attributes for richer UI projection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<(String, String)>,
}
