use crate::markdown_structure::parse_markdown_structure;

use super::types::MarkdownReference;

pub(super) fn extract_references_with_comrak(markdown: &str) -> Vec<MarkdownReference> {
    parse_markdown_structure(markdown).references().to_vec()
}
