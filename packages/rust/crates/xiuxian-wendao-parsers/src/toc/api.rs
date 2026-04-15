use super::types::{MarkdownTocDocument, TocDocument};
use crate::document::parse_markdown_document_from_parts;
use crate::frontmatter::split_frontmatter;
use crate::markdown_structure::parse_markdown_structure;
use crate::sections::extract_sections_with_structure;

/// Parse one parser-owned Markdown TOC surface from raw content.
#[must_use]
pub fn parse_markdown_toc(content: &str, fallback_title: &str) -> MarkdownTocDocument {
    let (frontmatter, body) = split_frontmatter(content);
    let structure = parse_markdown_structure(body);
    let document = parse_markdown_document_from_parts(
        frontmatter,
        body,
        fallback_title,
        structure.first_heading_title(),
        structure.lead_snippet(),
        false,
    );
    let sections = extract_sections_with_structure(body, &structure);
    TocDocument { document, sections }
}
