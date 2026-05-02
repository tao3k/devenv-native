//! Document DTOs shared by Markdown and Org parser surfaces.

use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::BTreeMap;

/// Parser-owned document format family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentFormat {
    /// `CommonMark` or Markdown-family document input.
    Markdown,
    /// Org-mode family document input.
    Org,
}

/// Parser-owned cross-format document metadata and normalized body content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentCore {
    /// Document format family that produced this parser-owned contract.
    pub format: DocumentFormat,
    /// Format-normalized document body with top-level metadata stripped.
    pub body: String,
    /// Best-effort document title from format-specific metadata or caller fallback.
    pub title: String,
    /// Best-effort tags from format-specific metadata.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Optional semantic document type from format-specific metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_type: Option<String>,
    /// Best-effort leading content snippet from the body.
    pub lead: String,
    /// Best-effort word count computed from the body.
    pub word_count: usize,
}

/// Parser-owned top-level document envelope shared across formats.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "RawMetadata: serde::Serialize",
    deserialize = "RawMetadata: serde::Deserialize<'de>"
))]
pub struct DocumentEnvelope<RawMetadata> {
    /// Raw format-specific metadata preserved from the input document.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_metadata: Option<RawMetadata>,
    /// Cross-format parser-owned document metadata and stripped body.
    pub core: DocumentCore,
}

/// Parser-owned Markdown document metadata extracted from raw content.
pub type MarkdownDocument = DocumentEnvelope<Value>;

/// Parser-owned Org document metadata extracted from top-level keywords and
/// the document property drawer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OrgDocumentMetadata {
    /// Top-level `#+KEY: value` metadata, grouped by upper-cased key.
    #[serde(default)]
    pub keywords: BTreeMap<String, Vec<String>>,
    /// Top-level Org property drawer attributes.
    #[serde(default)]
    pub properties: BTreeMap<String, String>,
}

/// Parser-owned Org document metadata extracted from raw content.
pub type OrgDocument = DocumentEnvelope<OrgDocumentMetadata>;
