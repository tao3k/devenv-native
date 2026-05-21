//! `zhenfa_router::native::semantic_check::docs_governance::collection::footer` owns Wendao docs governance collection footer behavior.

use crate::parsers::docs_governance::{collect_lines, parse_footer_block};
use crate::zhenfa_router::native::semantic_check::docs_governance::rendering::render_index_footer_with_values;
use crate::zhenfa_router::native::semantic_check::docs_governance::types::{
    INCOMPLETE_PACKAGE_DOCS_INDEX_FOOTER_BLOCK_ISSUE_TYPE,
    STALE_PACKAGE_DOCS_INDEX_FOOTER_STANDARDS_ISSUE_TYPE,
};
use crate::zhenfa_router::native::semantic_check::{IssueLocation, SemanticIssue};

/// Collect footer issues where the index `:FOOTER:` block is missing required values
/// or still advertises an outdated standards version.
#[must_use]
pub fn collect_stale_index_footer_standards(doc_path: &str, content: &str) -> Vec<SemanticIssue> {
    let lines = collect_lines(content);
    let mut issues = Vec::new();

    if let Some(footer) = parse_footer_block(&lines) {
        if footer.standards_value.is_none() || footer.last_sync_value.is_none() {
            issues.push(SemanticIssue {
                severity: "warning".to_string(),
                issue_type: INCOMPLETE_PACKAGE_DOCS_INDEX_FOOTER_BLOCK_ISSUE_TYPE.to_string(),
                doc: doc_path.to_string(),
                node_id: doc_path.to_string(),
                message: "Incomplete :FOOTER: block in documentation index (missing :STANDARDS: or :LAST_SYNC:)".to_string(),
                location: Some(IssueLocation {
                    line: footer.line,
                    heading_path: "Index Footer".to_string(),
                    byte_range: Some((footer.start_offset, footer.end_offset)),
                }),
                suggestion: Some(render_index_footer_with_values(
                    "v2.0",
                    footer.last_sync_value.unwrap_or("pending"),
                )),
                fuzzy_suggestion: None,
            });
        } else if footer.standards_value != Some("v2.0") {
            issues.push(SemanticIssue {
                severity: "warning".to_string(),
                issue_type: STALE_PACKAGE_DOCS_INDEX_FOOTER_STANDARDS_ISSUE_TYPE.to_string(),
                doc: doc_path.to_string(),
                node_id: doc_path.to_string(),
                message: format!(
                    "Stale documentation standards version in index: found '{:?}', expected 'v2.0'",
                    footer.standards_value
                ),
                location: Some(IssueLocation {
                    line: footer.line,
                    heading_path: "Index Footer".to_string(),
                    byte_range: Some((footer.start_offset, footer.end_offset)),
                }),
                suggestion: Some(render_index_footer_with_values(
                    "v2.0",
                    footer.last_sync_value.unwrap_or("pending"),
                )),
                fuzzy_suggestion: None,
            });
        }
    }

    issues
}
