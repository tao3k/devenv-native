//! `zhenfa_router::native::audit::audit_bridge::surgical` owns Wendao audit audit bridge surgical behavior.

use std::collections::HashMap;
use std::hash::BuildHasher;

use crate::zhenfa_router::native::semantic_check::docs_governance::{
    DOC_IDENTITY_PROTOCOL_ISSUE_TYPE, INCOMPLETE_PACKAGE_DOCS_INDEX_FOOTER_BLOCK_ISSUE_TYPE,
    MISSING_PACKAGE_DOCS_INDEX_FOOTER_BLOCK_ISSUE_TYPE, MISSING_PACKAGE_DOCS_INDEX_ISSUE_TYPE,
    MISSING_PACKAGE_DOCS_INDEX_RELATION_LINK_ISSUE_TYPE,
    MISSING_PACKAGE_DOCS_INDEX_RELATIONS_BLOCK_ISSUE_TYPE,
    MISSING_PACKAGE_DOCS_INDEX_SECTION_LINK_ISSUE_TYPE,
    MISSING_PACKAGE_DOCS_SECTION_LANDING_ISSUE_TYPE, MISSING_PACKAGE_DOCS_TREE_ISSUE_TYPE,
    STALE_PACKAGE_DOCS_INDEX_FOOTER_STANDARDS_ISSUE_TYPE,
    STALE_PACKAGE_DOCS_INDEX_RELATION_LINK_ISSUE_TYPE,
};
use crate::zhenfa_router::native::semantic_check::{IssueLocation, SemanticIssue};

use super::{BatchFix, ByteRange};
use super::{compute_hash, resolve_file_content};

fn build_range_fix(
    issue: &SemanticIssue,
    location: &IssueLocation,
    file_content: &str,
) -> Option<BatchFix> {
    let (start, end) = location.byte_range?;
    let byte_range = ByteRange::new(start, end);
    let original_content = byte_range
        .extract(file_content)
        .unwrap_or_default()
        .to_string();

    Some(
        BatchFix::new(
            issue.issue_type.clone(),
            issue.doc.clone(),
            location.line,
            original_content,
            issue.suggestion.clone().unwrap_or_default(),
            1.0,
        )
        .with_surgical(byte_range, compute_hash(file_content)),
    )
}

/// Generate surgical batch fixes with byte precision.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn generate_surgical_fixes<S: BuildHasher>(
    issues: &[SemanticIssue],
    file_contents: &HashMap<String, String, S>,
) -> Vec<BatchFix> {
    issues
        .iter()
        .filter_map(|issue| {
            if matches!(
                issue.issue_type.as_str(),
                MISSING_PACKAGE_DOCS_INDEX_ISSUE_TYPE
                    | MISSING_PACKAGE_DOCS_TREE_ISSUE_TYPE
                    | MISSING_PACKAGE_DOCS_SECTION_LANDING_ISSUE_TYPE
            ) {
                return Some(BatchFix::create_file(
                    issue.doc.clone(),
                    issue.suggestion.clone().unwrap_or_default(),
                    1.0,
                ));
            }

            let location = issue.location.as_ref()?;
            let file_content = resolve_file_content(file_contents, &issue.doc)?;

            match issue.issue_type.as_str() {
                DOC_IDENTITY_PROTOCOL_ISSUE_TYPE
                | MISSING_PACKAGE_DOCS_INDEX_SECTION_LINK_ISSUE_TYPE
                | MISSING_PACKAGE_DOCS_INDEX_RELATION_LINK_ISSUE_TYPE
                | MISSING_PACKAGE_DOCS_INDEX_RELATIONS_BLOCK_ISSUE_TYPE
                | MISSING_PACKAGE_DOCS_INDEX_FOOTER_BLOCK_ISSUE_TYPE
                | INCOMPLETE_PACKAGE_DOCS_INDEX_FOOTER_BLOCK_ISSUE_TYPE
                | STALE_PACKAGE_DOCS_INDEX_FOOTER_STANDARDS_ISSUE_TYPE
                | STALE_PACKAGE_DOCS_INDEX_RELATION_LINK_ISSUE_TYPE => {
                    build_range_fix(issue, location, file_content)
                }
                _ => {
                    let suggestion = issue.fuzzy_suggestion.as_ref()?;
                    Some(BatchFix::from_fuzzy_suggestion_surgical(
                        issue.doc.clone(),
                        location,
                        issue.suggestion.clone().unwrap_or_default(),
                        file_content,
                        suggestion,
                    ))
                }
            }
        })
        .collect()
}
