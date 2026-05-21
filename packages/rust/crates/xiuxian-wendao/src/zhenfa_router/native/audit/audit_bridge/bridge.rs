//! `zhenfa_router::native::audit::audit_bridge::bridge` owns Wendao audit audit bridge bridge behavior.

use crate::zhenfa_router::native::semantic_check::SemanticIssue;
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

use super::BatchFix;

fn line_number(issue: &SemanticIssue) -> usize {
    issue.location.as_ref().map_or(0, |loc| loc.line)
}

fn build_fix(issue: &SemanticIssue) -> BatchFix {
    BatchFix::new(
        issue.issue_type.clone(),
        issue.doc.clone(),
        line_number(issue),
        String::new(),
        issue.suggestion.clone().unwrap_or_default(),
        1.0,
    )
}

fn build_create_file_fix(issue: &SemanticIssue) -> BatchFix {
    BatchFix::create_file(
        issue.doc.clone(),
        issue.suggestion.clone().unwrap_or_default(),
        1.0,
    )
}

/// Trait for bridging audit results to external tools.
pub trait AuditBridge: Send + std::fmt::Debug {
    /// Process audit issues and generate batch fixes.
    fn generate_fixes(&self, issues: &[SemanticIssue]) -> Vec<BatchFix>;
}

/// Default implementation of `AuditBridge` that generates fixes but doesn't apply them.
#[derive(Debug, Default)]
pub struct DefaultAuditBridge;

impl AuditBridge for DefaultAuditBridge {
    #[allow(clippy::too_many_lines)]
    fn generate_fixes(&self, issues: &[SemanticIssue]) -> Vec<BatchFix> {
        issues
            .iter()
            .filter_map(|issue| match issue.issue_type.as_str() {
                DOC_IDENTITY_PROTOCOL_ISSUE_TYPE
                | MISSING_PACKAGE_DOCS_INDEX_SECTION_LINK_ISSUE_TYPE
                | MISSING_PACKAGE_DOCS_INDEX_RELATION_LINK_ISSUE_TYPE
                | MISSING_PACKAGE_DOCS_INDEX_RELATIONS_BLOCK_ISSUE_TYPE
                | MISSING_PACKAGE_DOCS_INDEX_FOOTER_BLOCK_ISSUE_TYPE
                | INCOMPLETE_PACKAGE_DOCS_INDEX_FOOTER_BLOCK_ISSUE_TYPE
                | STALE_PACKAGE_DOCS_INDEX_FOOTER_STANDARDS_ISSUE_TYPE
                | STALE_PACKAGE_DOCS_INDEX_RELATION_LINK_ISSUE_TYPE => Some(build_fix(issue)),
                MISSING_PACKAGE_DOCS_INDEX_ISSUE_TYPE
                | MISSING_PACKAGE_DOCS_TREE_ISSUE_TYPE
                | MISSING_PACKAGE_DOCS_SECTION_LANDING_ISSUE_TYPE => {
                    Some(build_create_file_fix(issue))
                }
                _ => issue.fuzzy_suggestion.as_ref().map(|suggestion| {
                    BatchFix::from_fuzzy_suggestion(
                        issue.doc.clone(),
                        line_number(issue),
                        issue.suggestion.clone().unwrap_or_default(),
                        suggestion,
                    )
                }),
            })
            .collect()
    }
}

/// Generate batch fixes from semantic check issues.
#[must_use]
pub fn generate_batch_fixes(issues: &[SemanticIssue]) -> Vec<BatchFix> {
    let bridge = DefaultAuditBridge;
    bridge.generate_fixes(issues)
}
