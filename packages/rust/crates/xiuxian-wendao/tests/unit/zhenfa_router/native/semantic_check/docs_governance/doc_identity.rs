use crate::parsers::docs_governance::derive_opaque_doc_id;
use crate::zhenfa_router::native::semantic_check::docs_governance::{
    DOC_IDENTITY_PROTOCOL_ISSUE_TYPE, collect_doc_governance_issues,
};

#[test]
fn detects_non_opaque_doc_identity_for_package_local_docs() {
    let content = "# Title\n\n:PROPERTIES:\n:ID: readable-id\n:TYPE: CORE\n:END:\n";
    let doc_path = "packages/rust/crates/demo/docs/01_core/101_test.md";
    let issues = collect_doc_governance_issues(doc_path, content);
    let expected = derive_opaque_doc_id(doc_path);

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].issue_type, DOC_IDENTITY_PROTOCOL_ISSUE_TYPE);
    assert_eq!(issues[0].suggestion.as_deref(), Some(expected.as_str()));
    assert_eq!(issues[0].location.as_ref().map(|loc| loc.line), Some(4));
}

#[test]
fn detects_missing_doc_identity_inside_top_properties_drawer() {
    let content = "# Title\n\n:PROPERTIES:\n:TYPE: CORE\n:END:\n";
    let doc_path = "packages/rust/crates/demo/docs/01_core/101_test.md";
    let issues = collect_doc_governance_issues(doc_path, content);
    let expected = format!(":ID: {}\n", derive_opaque_doc_id(doc_path));

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].issue_type, DOC_IDENTITY_PROTOCOL_ISSUE_TYPE);
    assert_eq!(issues[0].suggestion.as_deref(), Some(expected.as_str()));
    assert_eq!(issues[0].location.as_ref().map(|loc| loc.line), Some(4));
    assert_eq!(
        issues[0].location.as_ref().and_then(|loc| loc.byte_range),
        Some((22, 22))
    );
}

#[test]
fn ignores_docs_outside_package_local_crate_docs() {
    let content = "# Title\n\n:PROPERTIES:\n:ID: readable-id\n:TYPE: CORE\n:END:\n";
    let issues = collect_doc_governance_issues("docs/notes.md", content);
    assert!(issues.is_empty());
}
