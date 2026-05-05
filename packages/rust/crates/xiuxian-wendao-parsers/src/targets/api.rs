//! Public Markdown target occurrence extraction API.

use super::types::MarkdownTargetOccurrence;

/// Extract raw Markdown target occurrences in document order.
#[must_use]
pub fn extract_targets(markdown: &str) -> Vec<MarkdownTargetOccurrence> {
    super::scan::extract_targets_with_comrak(markdown)
}
