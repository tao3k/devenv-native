mod basic;
mod display;
mod edges;
mod generation;
mod package_docs;

pub(super) use crate::zhenfa_router::native::audit::audit_bridge::{
    AuditBridge, BatchFix, BatchFixMode, ByteRange, DefaultAuditBridge, FixResult, compute_hash,
    generate_batch_fixes, generate_surgical_fixes,
};
pub(super) use crate::zhenfa_router::native::semantic_check::docs_governance::{
    INCOMPLETE_PACKAGE_DOCS_INDEX_FOOTER_BLOCK_ISSUE_TYPE,
    MISSING_PACKAGE_DOCS_INDEX_FOOTER_BLOCK_ISSUE_TYPE, MISSING_PACKAGE_DOCS_INDEX_ISSUE_TYPE,
    MISSING_PACKAGE_DOCS_INDEX_RELATION_LINK_ISSUE_TYPE,
    MISSING_PACKAGE_DOCS_INDEX_RELATIONS_BLOCK_ISSUE_TYPE,
    MISSING_PACKAGE_DOCS_INDEX_SECTION_LINK_ISSUE_TYPE,
    MISSING_PACKAGE_DOCS_SECTION_LANDING_ISSUE_TYPE, MISSING_PACKAGE_DOCS_TREE_ISSUE_TYPE,
    STALE_PACKAGE_DOCS_INDEX_FOOTER_STANDARDS_ISSUE_TYPE,
    STALE_PACKAGE_DOCS_INDEX_RELATION_LINK_ISSUE_TYPE,
};
pub(super) use crate::zhenfa_router::native::semantic_check::{
    FuzzySuggestionData, IssueLocation, SemanticIssue,
};

pub(super) fn test_file_content() -> String {
    "line 1\n:OBSERVE: lang:rust \"fn process_data\"\nline 3".to_string()
}

pub(super) fn observe_line_range() -> ByteRange {
    let content = test_file_content();
    let observe_content = r#":OBSERVE: lang:rust "fn process_data""#;
    let Some(start) = content.find(observe_content) else {
        panic!("OBSERVE content should exist");
    };
    let end = start + observe_content.len();
    ByteRange::new(start, end)
}
