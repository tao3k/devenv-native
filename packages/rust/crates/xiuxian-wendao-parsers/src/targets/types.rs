//! Markdown target occurrence DTOs.

use serde::{Deserialize, Serialize};

/// Parser-owned ordered target-occurrence contract shared across formats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "Kind: serde::Serialize",
    deserialize = "Kind: serde::Deserialize<'de>"
))]
pub struct TargetOccurrenceCore<Kind> {
    /// Which surface syntax produced this occurrence.
    pub kind: Kind,
    /// The parser-visible target string captured for this occurrence.
    pub target: String,
    /// Exact parser-visible source syntax for this occurrence.
    #[serde(default)]
    pub surface: String,
    /// Byte range within the parsed document body.
    pub byte_range: (usize, usize),
    /// Inclusive 1-based line range within the parsed document body.
    pub line_range: (usize, usize),
}

/// The concrete target occurrence syntax found in Markdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkdownTargetOccurrenceKind {
    /// A standard Markdown inline link such as `[label](target)`.
    MarkdownLink,
    /// A standard Markdown image such as `![label](target)`.
    MarkdownImage,
    /// A wiki-style link such as `[[target]]` or `[[target|alias]]`.
    WikiLink,
    /// A wiki-style embed such as `![[target]]`.
    WikiEmbed,
}

/// Markdown-local name for the shared parser-owned target-occurrence core.
pub type MarkdownTargetOccurrence = TargetOccurrenceCore<MarkdownTargetOccurrenceKind>;

impl<Kind> TargetOccurrenceCore<Kind> {
    #[must_use]
    pub(crate) fn new(
        kind: Kind,
        target: String,
        surface: String,
        byte_range: (usize, usize),
        line_range: (usize, usize),
    ) -> Self {
        Self {
            kind,
            target,
            surface,
            byte_range,
            line_range,
        }
    }
}
