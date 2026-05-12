//! `link_graph::models::records::hits` owns Wendao models records hits behavior.

use serde::{Deserialize, Serialize};

/// Search hit.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Stringly state boundary: this public record preserves serialized catalog tokens from external or stored Wendao data.
pub struct LinkGraphHit {
    /// Stem identifier.
    pub stem: String,
    /// Optional title.
    pub title: String,
    /// Relative path.
    pub path: String,
    /// Optional semantic document type (`type`/`kind`) from source frontmatter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_type: Option<String>,
    /// Document tags propagated from frontmatter/metadata.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Relevance score (0-1).
    pub score: f64,
    /// Best-matching section/heading path when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub best_section: Option<String>,
    /// Human-readable match reason for debugging/observability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_reason: Option<String>,
}

/// Display-friendly search hit for external payload contracts.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Stringly state boundary: this public record preserves serialized catalog tokens from external or stored Wendao data.
pub struct LinkGraphDisplayHit {
    /// Stem identifier.
    pub stem: String,
    /// Optional title.
    pub title: String,
    /// Relative path.
    pub path: String,
    /// Optional semantic document type (`type`/`kind`) from source frontmatter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_type: Option<String>,
    /// Document tags propagated from frontmatter/metadata.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Relevance score (0-1).
    pub score: f64,
    /// Best-matching section/heading path (empty when unavailable).
    pub best_section: String,
    /// Human-readable match reason (empty when unavailable).
    pub match_reason: String,
}

impl From<&LinkGraphHit> for LinkGraphDisplayHit {
    fn from(value: &LinkGraphHit) -> Self {
        Self {
            stem: value.stem.clone(),
            title: value.title.clone(),
            path: value.path.clone(),
            doc_type: value.doc_type.clone(),
            tags: value.tags.clone(),
            score: value.score.max(0.0),
            best_section: value.best_section.clone().unwrap_or_default(),
            match_reason: value.match_reason.clone().unwrap_or_default(),
        }
    }
}
