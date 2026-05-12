//! `enhancer::markdown_config::types` owns Wendao enhancer markdown config types behavior.

use serde::{Deserialize, Serialize};

/// Extracted markdown configuration block bound to a tagged heading.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Stringly state boundary: this public record preserves serialized catalog tokens from external or stored Wendao data.
pub struct MarkdownConfigBlock {
    /// Exact identifier from HTML property tag.
    pub id: String,
    /// Configuration kind from HTML property tag.
    pub config_type: String,
    /// Optional logical template target.
    pub target: Option<String>,
    /// Heading title that owns this config block.
    pub heading: String,
    /// Fenced code language (for example `jinja2`).
    pub language: String,
    /// Raw code block content extracted from AST.
    pub content: String,
}

/// One normalized link target extracted under a tagged config heading.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Stringly state boundary: this public record preserves serialized catalog tokens from external or stored Wendao data.
pub struct MarkdownConfigLinkTarget {
    /// Normalized target path or semantic URI.
    pub target: String,
    /// Optional explicit reference category from section metadata.
    pub reference_type: Option<String>,
}
