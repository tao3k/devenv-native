use std::collections::HashMap;

use super::{
    ByteRange, INCOMPLETE_PACKAGE_DOCS_INDEX_FOOTER_BLOCK_ISSUE_TYPE, IssueLocation,
    STALE_PACKAGE_DOCS_INDEX_FOOTER_STANDARDS_ISSUE_TYPE,
    STALE_PACKAGE_DOCS_INDEX_RELATION_LINK_ISSUE_TYPE, SemanticIssue, generate_surgical_fixes,
};

#[test]
fn test_generate_surgical_fixes_supports_incomplete_package_docs_index_footer_block() {
    let doc_path = "/tmp/demo/docs/index.md".to_string();
    let file_content = ":FOOTER:\n:STANDARDS: v2.0\n:END:\n".to_string();
    let mut file_contents = HashMap::new();
    file_contents.insert(doc_path.clone(), file_content.clone());

    let issues = vec![SemanticIssue {
        severity: "warning".to_string(),
        issue_type: INCOMPLETE_PACKAGE_DOCS_INDEX_FOOTER_BLOCK_ISSUE_TYPE.to_string(),
        doc: doc_path,
        node_id: "/tmp/demo/docs/index.md".to_string(),
        message: "Incomplete package docs footer block".to_string(),
        location: Some(IssueLocation {
            line: 1,
            heading_path: "Index Footer".to_string(),
            byte_range: Some((0, file_content.len())),
        }),
        suggestion: Some(":FOOTER:\n:STANDARDS: v2.0\n:LAST_SYNC: pending\n:END:\n".to_string()),
        fuzzy_suggestion: None,
    }];

    let fixes = generate_surgical_fixes(&issues, &file_contents);

    assert_eq!(fixes.len(), 1);
    assert!(fixes[0].is_surgical());
    assert!(!fixes[0].is_create_file());
    assert_eq!(
        fixes[0].replacement,
        ":FOOTER:\n:STANDARDS: v2.0\n:LAST_SYNC: pending\n:END:\n"
    );
    assert_eq!(
        fixes[0].byte_range,
        Some(ByteRange::new(0, file_content.len()))
    );
}

#[test]
fn test_generate_surgical_fixes_supports_stale_package_docs_index_footer_standards() {
    let doc_path = "/tmp/demo/docs/index.md".to_string();
    let file_content = ":FOOTER:\n:STANDARDS: v1.0\n:LAST_SYNC: 2026-03-20\n:END:\n".to_string();
    let mut file_contents = HashMap::new();
    file_contents.insert(doc_path.clone(), file_content.clone());

    let issues = vec![SemanticIssue {
        severity: "warning".to_string(),
        issue_type: STALE_PACKAGE_DOCS_INDEX_FOOTER_STANDARDS_ISSUE_TYPE.to_string(),
        doc: doc_path,
        node_id: "/tmp/demo/docs/index.md".to_string(),
        message: "Stale package docs footer standards".to_string(),
        location: Some(IssueLocation {
            line: 1,
            heading_path: "Index Footer".to_string(),
            byte_range: Some((0, file_content.len())),
        }),
        suggestion: Some(":FOOTER:\n:STANDARDS: v2.0\n:LAST_SYNC: 2026-03-20\n:END:\n".to_string()),
        fuzzy_suggestion: None,
    }];

    let fixes = generate_surgical_fixes(&issues, &file_contents);

    assert_eq!(fixes.len(), 1);
    assert!(fixes[0].is_surgical());
    assert!(!fixes[0].is_create_file());
    assert_eq!(
        fixes[0].replacement,
        ":FOOTER:\n:STANDARDS: v2.0\n:LAST_SYNC: 2026-03-20\n:END:\n"
    );
    assert_eq!(
        fixes[0].byte_range,
        Some(ByteRange::new(0, file_content.len()))
    );
}

#[test]
fn test_generate_surgical_fixes_resolves_absolute_doc_path_against_relative_file_content_key() {
    let relative_doc_path = "packages/rust/crates/demo/docs/index.md".to_string();
    let absolute_doc_path = "/tmp/workspace/packages/rust/crates/demo/docs/index.md".to_string();
    let file_content = ":FOOTER:\n:STANDARDS: v1.0\n:LAST_SYNC: 2026-03-20\n:END:\n".to_string();
    let mut file_contents = HashMap::new();
    file_contents.insert(relative_doc_path, file_content.clone());

    let issues = vec![SemanticIssue {
        severity: "warning".to_string(),
        issue_type: STALE_PACKAGE_DOCS_INDEX_FOOTER_STANDARDS_ISSUE_TYPE.to_string(),
        doc: absolute_doc_path,
        node_id: "/tmp/workspace/packages/rust/crates/demo/docs/index.md".to_string(),
        message: "Stale package docs footer standards".to_string(),
        location: Some(IssueLocation {
            line: 1,
            heading_path: "Index Footer".to_string(),
            byte_range: Some((0, file_content.len())),
        }),
        suggestion: Some(":FOOTER:\n:STANDARDS: v2.0\n:LAST_SYNC: 2026-03-20\n:END:\n".to_string()),
        fuzzy_suggestion: None,
    }];

    let fixes = generate_surgical_fixes(&issues, &file_contents);

    assert_eq!(fixes.len(), 1);
    assert!(fixes[0].is_surgical());
    assert_eq!(
        fixes[0].replacement,
        ":FOOTER:\n:STANDARDS: v2.0\n:LAST_SYNC: 2026-03-20\n:END:\n"
    );
}

#[test]
fn test_generate_surgical_fixes_supports_stale_package_docs_index_relation_link() {
    let doc_path = "/tmp/demo/docs/index.md".to_string();
    let file_content =
        ":RELATIONS:\n:LINKS: [[01_core/101_demo_core_boundary]], [[01_core/999_stale]]\n:END:\n"
            .to_string();
    let mut file_contents = HashMap::new();
    file_contents.insert(doc_path.clone(), file_content.clone());

    let relation_value = "[[01_core/101_demo_core_boundary]], [[01_core/999_stale]]";
    let value_start = file_content
        .find(relation_value)
        .unwrap_or_else(|| panic!("find links value"));
    let value_end = value_start + relation_value.len();

    let issues = vec![SemanticIssue {
        severity: "warning".to_string(),
        issue_type: STALE_PACKAGE_DOCS_INDEX_RELATION_LINK_ISSUE_TYPE.to_string(),
        doc: doc_path,
        node_id: "/tmp/demo/docs/index.md".to_string(),
        message: "Stale package docs relation link".to_string(),
        location: Some(IssueLocation {
            line: 2,
            heading_path: "Index Relations".to_string(),
            byte_range: Some((value_start, value_end)),
        }),
        suggestion: Some("[[01_core/101_demo_core_boundary]]".to_string()),
        fuzzy_suggestion: None,
    }];

    let fixes = generate_surgical_fixes(&issues, &file_contents);

    assert_eq!(fixes.len(), 1);
    assert!(fixes[0].is_surgical());
    assert!(!fixes[0].is_create_file());
    assert_eq!(fixes[0].replacement, "[[01_core/101_demo_core_boundary]]");
    assert_eq!(
        fixes[0].byte_range,
        Some(ByteRange::new(value_start, value_end))
    );
}
