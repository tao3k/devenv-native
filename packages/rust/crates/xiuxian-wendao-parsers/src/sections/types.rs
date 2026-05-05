//! Markdown section DTOs and normalized section core.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Execution log entry from a `:LOGBOOK:` drawer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogbookEntry {
    /// Timestamp of the log entry.
    pub timestamp: String,
    /// Log message content.
    pub message: String,
    /// 1-based line number within the document.
    pub line_number: usize,
}

/// Parser-owned section metadata payload shared across document formats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionMetadata {
    /// Property drawer attributes extracted from the heading section.
    pub attributes: HashMap<String, String>,
    /// Execution log entries extracted from the section.
    pub logbook: Vec<LogbookEntry>,
}

/// Parser-owned section identity and source-range contract shared across formats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionScope {
    /// Leaf heading title for this section.
    pub heading_title: String,
    /// Slash-delimited heading ancestry for this section.
    pub heading_path: String,
    /// Lower-cased `heading_path` for case-insensitive matching.
    pub heading_path_lower: String,
    /// Normalized heading depth for this section.
    pub heading_level: usize,
    /// Inclusive 1-based start line within the document body.
    pub line_start: usize,
    /// Inclusive 1-based end line within the document body.
    pub line_end: usize,
    /// Byte offset from start of document where this section begins.
    pub byte_start: usize,
    /// Byte offset (exclusive) where this section ends.
    pub byte_end: usize,
}

/// Parser-owned full section contract shared across document formats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionCore {
    /// Shared section identity and source-range contract.
    #[serde(flatten)]
    pub scope: SectionScope,
    /// Content contained by this section.
    pub section_text: String,
    /// Lower-cased section text for case-insensitive matching.
    pub section_text_lower: String,
    /// Parser-owned section metadata payload.
    #[serde(flatten)]
    pub metadata: SectionMetadata,
}

/// Markdown-local name for the shared parser-owned section core.
pub type MarkdownSection = SectionCore;

impl SectionCore {
    /// Shared section metadata payload.
    #[must_use]
    pub fn metadata(&self) -> &SectionMetadata {
        &self.metadata
    }

    /// Shared section identity and source-range contract.
    #[must_use]
    pub fn scope(&self) -> &SectionScope {
        &self.scope
    }

    /// Leaf heading title for this section.
    #[must_use]
    pub fn heading_title(&self) -> &str {
        &self.scope.heading_title
    }

    /// Slash-delimited heading ancestry for this section.
    #[must_use]
    pub fn heading_path(&self) -> &str {
        &self.scope.heading_path
    }

    /// Lower-cased section ancestry for case-insensitive matching.
    #[must_use]
    pub fn heading_path_lower(&self) -> &str {
        &self.scope.heading_path_lower
    }

    /// Heading depth for this section.
    #[must_use]
    pub fn heading_level(&self) -> usize {
        self.scope.heading_level
    }

    /// Inclusive 1-based start line within the document body.
    #[must_use]
    pub fn line_start(&self) -> usize {
        self.scope.line_start
    }

    /// Inclusive 1-based end line within the document body.
    #[must_use]
    pub fn line_end(&self) -> usize {
        self.scope.line_end
    }

    /// Byte offset from start of document where this section begins.
    #[must_use]
    pub fn byte_start(&self) -> usize {
        self.scope.byte_start
    }

    /// Byte offset (exclusive) where this section ends.
    #[must_use]
    pub fn byte_end(&self) -> usize {
        self.scope.byte_end
    }

    /// Property drawer attributes extracted from the heading section.
    #[must_use]
    pub fn attributes(&self) -> &HashMap<String, String> {
        &self.metadata.attributes
    }

    /// Execution log entries extracted from the section.
    #[must_use]
    pub fn logbook(&self) -> &[LogbookEntry] {
        &self.metadata.logbook
    }
}
