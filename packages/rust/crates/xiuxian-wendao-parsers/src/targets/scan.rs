use super::types::MarkdownTargetOccurrence;
use crate::markdown_structure::parse_markdown_structure;

pub(super) fn extract_targets_with_comrak(markdown: &str) -> Vec<MarkdownTargetOccurrence> {
    parse_markdown_structure(markdown).targets().to_vec()
}
