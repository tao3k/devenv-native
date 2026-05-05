//! Public Markdown reference extraction API.

use super::types::MarkdownReference;

/// Extract ordinary Markdown references in document order.
#[must_use]
pub fn extract_references(markdown: &str) -> Vec<MarkdownReference> {
    super::scan::extract_references_with_comrak(markdown)
}

/// Parse a standalone ordinary Markdown reference literal.
#[must_use]
pub fn parse_reference_literal(text: &str) -> Option<MarkdownReference> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut references = extract_references(trimmed);
    if references.len() != 1 {
        return None;
    }

    let parsed = references.pop()?;
    (parsed.original == trimmed).then_some(parsed)
}
