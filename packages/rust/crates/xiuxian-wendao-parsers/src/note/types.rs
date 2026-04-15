use crate::document::MarkdownDocument;
use crate::references::MarkdownReference;
use crate::sections::MarkdownSection;
use crate::targets::MarkdownTargetOccurrence;
use serde::{Deserialize, Serialize};

/// Parser-owned reusable note-body aggregation shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(bound(
    serialize = "Reference: serde::Serialize, Target: serde::Serialize, Section: serde::Serialize",
    deserialize = "Reference: serde::Deserialize<'de>, Target: serde::Deserialize<'de>, Section: serde::Deserialize<'de>"
))]
pub struct NoteCore<Reference, Target, Section> {
    /// Ordinary format-owned references in document order.
    #[serde(default)]
    pub references: Vec<Reference>,
    /// Raw note-level target occurrences in document order.
    #[serde(default)]
    pub targets: Vec<Target>,
    /// Parser-owned section structure extracted from the document body.
    #[serde(default)]
    pub sections: Vec<Section>,
}

/// Markdown-specific note-body aggregation over parser-owned Markdown item contracts.
pub type MarkdownNoteCore = NoteCore<MarkdownReference, MarkdownTargetOccurrence, MarkdownSection>;

/// Parser-owned top-level note aggregate shared across formats.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "Document: serde::Serialize, Reference: serde::Serialize, Target: serde::Serialize, Section: serde::Serialize",
    deserialize = "Document: serde::Deserialize<'de>, Reference: serde::Deserialize<'de>, Target: serde::Deserialize<'de>, Section: serde::Deserialize<'de>"
))]
pub struct NoteAggregate<Document, Reference, Target, Section> {
    /// Parser-owned format wrapper and stripped body.
    pub document: Document,
    /// Cross-format note-body aggregation shape populated with format-owned item types.
    #[serde(default)]
    #[serde(flatten)]
    pub core: NoteCore<Reference, Target, Section>,
}

/// Parser-owned aggregate for one Markdown note body.
pub type MarkdownNote =
    NoteAggregate<MarkdownDocument, MarkdownReference, MarkdownTargetOccurrence, MarkdownSection>;

/// Parser-owned single-pass markdown parse artifacts for direct consumers.
#[derive(Debug, Clone, PartialEq)]
pub struct MarkdownNoteParseArtifacts {
    /// Parser-owned note aggregate assembled from one markdown body.
    pub note: MarkdownNote,
    /// Parser-owned symbol fingerprint derived from the same structural pass.
    pub symbol_fingerprint: String,
}
