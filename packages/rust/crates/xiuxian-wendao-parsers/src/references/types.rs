//! Markdown reference DTOs.

use crate::reference_core::ReferenceCore;
use serde::{Deserialize, Serialize};

/// The concrete reference syntax found in Markdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkdownReferenceKind {
    /// A standard Markdown inline link such as `[label](target)`.
    Markdown,
    /// A wiki-style link such as `[[target]]` or `[[target|alias]]`.
    WikiLink,
}

/// One ordinary Markdown reference with structural target coordinates.
pub type MarkdownReference = ReferenceCore<MarkdownReferenceKind>;
