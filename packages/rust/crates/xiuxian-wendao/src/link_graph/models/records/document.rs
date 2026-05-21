//! `link_graph::models::records::document` owns Wendao models records document behavior.

use serde::{Deserialize, Serialize};

fn default_doc_saliency_base() -> f64 {
    crate::link_graph::saliency::DEFAULT_SALIENCY_BASE
}

fn default_doc_decay_rate() -> f64 {
    crate::link_graph::saliency::DEFAULT_DECAY_RATE
}

/// Indexed document row.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Stringly state boundary: this public record preserves serialized catalog tokens from external or stored Wendao data.
pub struct LinkGraphDocument {
    /// Canonical ID (relative path without extension, '/' separator).
    pub id: String,
    /// Lowercased canonical ID for case-insensitive matching.
    #[serde(skip_serializing, default)]
    pub id_lower: String,
    /// File stem (basename without extension).
    pub stem: String,
    /// Lowercased file stem for case-insensitive matching.
    #[serde(skip_serializing, default)]
    pub stem_lower: String,
    /// Relative path with extension.
    pub path: String,
    /// Lowercased relative path for case-insensitive matching.
    #[serde(skip_serializing, default)]
    pub path_lower: String,
    /// Best-effort title.
    pub title: String,
    /// Lowercased title for case-insensitive matching.
    #[serde(skip_serializing, default)]
    pub title_lower: String,
    /// Best-effort tags.
    pub tags: Vec<String>,
    /// Lowercased tags for case-insensitive matching.
    #[serde(skip_serializing, default)]
    pub tags_lower: Vec<String>,
    /// Best-effort leading content snippet.
    pub lead: String,
    /// Optional semantic document type from frontmatter (`type`/`kind`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_type: Option<String>,
    /// Best-effort word count of body.
    #[serde(default)]
    pub word_count: usize,
    /// Searchable markdown content without frontmatter.
    #[serde(skip_serializing, default)]
    pub search_text: String,
    /// Lowercased searchable content for case-insensitive matching.
    #[serde(skip_serializing, default)]
    pub search_text_lower: String,
    /// Initial saliency baseline extracted from frontmatter (`saliency_base`).
    #[serde(default = "default_doc_saliency_base")]
    pub saliency_base: f64,
    /// Initial saliency decay rate extracted from frontmatter (`decay_rate`).
    #[serde(default = "default_doc_decay_rate")]
    pub decay_rate: f64,
    /// Best-effort created timestamp in Unix seconds.
    pub created_ts: Option<i64>,
    /// Best-effort modified timestamp in Unix seconds.
    pub modified_ts: Option<i64>,
}
