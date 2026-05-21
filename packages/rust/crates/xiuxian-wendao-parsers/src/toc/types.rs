//! Markdown table-of-contents DTOs.

use crate::document::{DocumentType, MarkdownDocument};
use crate::sections::MarkdownSection;
use serde::{Deserialize, Serialize};

/// Parser-owned lightweight heading outline entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkdownOutlineHeading {
    /// Heading title text.
    pub title: String,
    /// ATX heading level.
    pub level: usize,
    /// Inclusive source line range covered by the heading section.
    pub line_range: (usize, usize),
}

/// Parser-owned lightweight document outline for heading-driven consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkdownOutlineDocument {
    /// Best-effort title from frontmatter or the first heading.
    pub title: String,
    /// Optional semantic document type from frontmatter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_type: Option<DocumentType>,
    /// Total number of lines in the stripped Markdown body.
    pub line_count: usize,
    /// Heading outline entries in source order.
    #[serde(default)]
    pub headings: Vec<MarkdownOutlineHeading>,
}

/// Parser-owned reusable table-of-contents aggregate shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "Document: serde::Serialize, Section: serde::Serialize",
    deserialize = "Document: serde::Deserialize<'de>, Section: serde::Deserialize<'de>"
))]
pub struct TocDocument<Document, Section> {
    /// Parser-owned format wrapper and stripped body.
    pub document: Document,
    /// Parser-owned section structure extracted from the document body.
    #[serde(default)]
    pub sections: Vec<Section>,
}

/// Parser-owned aggregate for one Markdown TOC/body structure.
pub type MarkdownTocDocument = TocDocument<MarkdownDocument, MarkdownSection>;
