//! Studio-owned retrieval atom contracts.

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

#[cfg(feature = "local-runtime")]
impl From<xiuxian_wendao::search::contracts::RetrievalChunkSurface> for RetrievalChunkSurface {
    fn from(value: xiuxian_wendao::search::contracts::RetrievalChunkSurface) -> Self {
        match value {
            xiuxian_wendao::search::contracts::RetrievalChunkSurface::Document => Self::Document,
            xiuxian_wendao::search::contracts::RetrievalChunkSurface::Section => Self::Section,
            xiuxian_wendao::search::contracts::RetrievalChunkSurface::CodeBlock => Self::CodeBlock,
            xiuxian_wendao::search::contracts::RetrievalChunkSurface::Table => Self::Table,
            xiuxian_wendao::search::contracts::RetrievalChunkSurface::Math => Self::Math,
            xiuxian_wendao::search::contracts::RetrievalChunkSurface::Observation => {
                Self::Observation
            }
            xiuxian_wendao::search::contracts::RetrievalChunkSurface::Declaration => {
                Self::Declaration
            }
            xiuxian_wendao::search::contracts::RetrievalChunkSurface::Block => Self::Block,
            xiuxian_wendao::search::contracts::RetrievalChunkSurface::Symbol => Self::Symbol,
        }
    }
}

#[cfg(feature = "local-runtime")]
impl From<xiuxian_wendao::search::contracts::RetrievalChunk> for RetrievalChunk {
    fn from(value: xiuxian_wendao::search::contracts::RetrievalChunk) -> Self {
        Self {
            owner_id: value.owner_id,
            chunk_id: value.chunk_id,
            semantic_type: value.semantic_type,
            fingerprint: value.fingerprint,
            token_estimate: value.token_estimate,
            display_label: value.display_label,
            excerpt: value.excerpt,
            line_start: value.line_start,
            line_end: value.line_end,
            surface: value.surface.map(Into::into),
            attributes: value.attributes,
        }
    }
}
