//! `parsers::markdown::sections::types` owns Wendao markdown sections types behavior.

use std::collections::HashMap;
use xiuxian_wendao_parsers::sections::{
    LogbookEntry, MarkdownSection, SectionCore, SectionMetadata,
};

use crate::parsers::markdown::code_observation::CodeObservation;

/// Parsed section row for section-aware retrieval and `HippoRAG 2` `Passage Nodes`.
#[derive(Debug, Clone)]
pub struct ParsedSection {
    /// Leaf heading title for this section.
    pub heading_title: String,
    /// Slash-delimited heading ancestry for this section.
    pub heading_path: String,
    /// Lower-cased `heading_path` for case-insensitive matching.
    pub heading_path_lower: String,
    /// Markdown heading depth for this section.
    pub heading_level: usize,
    /// Inclusive 1-based start line within the markdown body.
    pub line_start: usize,
    /// Inclusive 1-based end line within the markdown body.
    pub line_end: usize,
    /// Byte offset from start of document where this section begins.
    pub byte_start: usize,
    /// Byte offset (exclusive) where this section ends.
    pub byte_end: usize,
    /// Content contained by this section.
    pub section_text: String,
    /// Lower-cased section text for case-insensitive matching.
    pub section_text_lower: String,
    /// List of entity IDs mentioned in this specific section.
    pub entities: Vec<String>,
    /// Property drawer attributes extracted from heading (e.g., :ID: arch-v1).
    pub attributes: HashMap<String, String>,
    /// Execution log entries from `:LOGBOOK:` drawer (Blueprint v2.4).
    pub logbook: Vec<LogbookEntry>,
    /// Code observations from `:OBSERVE:` property drawer (Blueprint v2.7).
    pub observations: Vec<CodeObservation>,
}

impl ParsedSection {
    pub(crate) fn from_parser_owned(
        section: MarkdownSection,
        entities: Vec<String>,
        observations: Vec<CodeObservation>,
    ) -> Self {
        let SectionCore {
            scope,
            section_text,
            section_text_lower,
            metadata,
        } = section;
        let SectionMetadata {
            attributes,
            logbook,
        } = metadata;
        Self {
            heading_title: scope.heading_title,
            heading_path: scope.heading_path,
            heading_path_lower: scope.heading_path_lower,
            heading_level: scope.heading_level,
            line_start: scope.line_start,
            line_end: scope.line_end,
            byte_start: scope.byte_start,
            byte_end: scope.byte_end,
            section_text,
            section_text_lower,
            entities,
            attributes,
            logbook,
            observations,
        }
    }
}
