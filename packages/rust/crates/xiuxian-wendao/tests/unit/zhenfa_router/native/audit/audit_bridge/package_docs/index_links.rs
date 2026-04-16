use super::*;

#[test]
fn test_generate_surgical_fixes_supports_missing_package_docs_index_section_link() {
    let doc_path = "/tmp/demo/docs/index.md".to_string();
    let file_content = "# Demo\n\n## 01_core\n\n".to_string();
    let mut file_contents = std::collections::HashMap::new();
    file_contents.insert(doc_path.clone(), file_content.clone());

    let issues = vec![SemanticIssue {
        severity: "warning".to_string(),
        issue_type: MISSING_PACKAGE_DOCS_INDEX_SECTION_LINK_ISSUE_TYPE.to_string(),
        doc: doc_path,
        node_id: "/tmp/demo/docs/index.md".to_string(),
        message: "Missing package docs index section link".to_string(),
        location: Some(IssueLocation {
            line: 3,
            heading_path: "01_core".to_string(),
            byte_range: Some((19, 19)),
        }),
        suggestion: Some("- [[01_core/101_demo_core_boundary]]\n".to_string()),
        fuzzy_suggestion: None,
    }];

    let fixes = generate_surgical_fixes(&issues, &file_contents);

    assert_eq!(fixes.len(), 1);
    assert!(fixes[0].is_surgical());
    assert!(!fixes[0].is_create_file());
    assert_eq!(
        fixes[0].replacement,
        "- [[01_core/101_demo_core_boundary]]\n"
    );
    assert_eq!(fixes[0].byte_range, Some(ByteRange::new(19, 19)));
}

#[test]
fn test_generate_surgical_fixes_supports_missing_package_docs_index_relation_link() {
    let doc_path = "/tmp/demo/docs/index.md".to_string();
    let file_content =
        ":RELATIONS:\n:LINKS: [[01_core/101_demo_core_boundary]]\n:END:\n".to_string();
    let mut file_contents = std::collections::HashMap::new();
    file_contents.insert(doc_path.clone(), file_content.clone());

    let relation_value = "[[01_core/101_demo_core_boundary]]";
    let value_start = file_content
        .find(relation_value)
        .unwrap_or_else(|| panic!("find links value"));
    let value_end = value_start + relation_value.len();

    let issues = vec![SemanticIssue {
        severity: "warning".to_string(),
        issue_type: MISSING_PACKAGE_DOCS_INDEX_RELATION_LINK_ISSUE_TYPE.to_string(),
        doc: doc_path,
        node_id: "/tmp/demo/docs/index.md".to_string(),
        message: "Missing package docs relation link".to_string(),
        location: Some(IssueLocation {
            line: 2,
            heading_path: "Index Relations".to_string(),
            byte_range: Some((value_start, value_end)),
        }),
        suggestion: Some(
            "[[01_core/101_demo_core_boundary]], [[01_core/102_demo_contracts]]".to_string(),
        ),
        fuzzy_suggestion: None,
    }];

    let fixes = generate_surgical_fixes(&issues, &file_contents);

    assert_eq!(fixes.len(), 1);
    assert!(fixes[0].is_surgical());
    assert!(!fixes[0].is_create_file());
    assert_eq!(
        fixes[0].replacement,
        "[[01_core/101_demo_core_boundary]], [[01_core/102_demo_contracts]]"
    );
    assert_eq!(
        fixes[0].byte_range,
        Some(ByteRange::new(value_start, value_end))
    );
}

#[test]
fn test_generate_surgical_fixes_supports_missing_package_docs_index_relations_block() {
    let doc_path = "/tmp/demo/docs/index.md".to_string();
    let file_content = "# Demo\n\n- [[01_core/101_demo_core_boundary]]\n\n---\n".to_string();
    let mut file_contents = std::collections::HashMap::new();
    file_contents.insert(doc_path.clone(), file_content.clone());

    let insert_offset = file_content
        .find("---")
        .unwrap_or_else(|| panic!("find footer separator"));
    let issues = vec![SemanticIssue {
        severity: "warning".to_string(),
        issue_type: MISSING_PACKAGE_DOCS_INDEX_RELATIONS_BLOCK_ISSUE_TYPE.to_string(),
        doc: doc_path,
        node_id: "/tmp/demo/docs/index.md".to_string(),
        message: "Missing package docs relations block".to_string(),
        location: Some(IssueLocation {
            line: 5,
            heading_path: "Index Relations".to_string(),
            byte_range: Some((insert_offset, insert_offset)),
        }),
        suggestion: Some(
            ":RELATIONS:\n:LINKS: [[01_core/101_demo_core_boundary]]\n:END:\n\n".to_string(),
        ),
        fuzzy_suggestion: None,
    }];

    let fixes = generate_surgical_fixes(&issues, &file_contents);

    assert_eq!(fixes.len(), 1);
    assert!(fixes[0].is_surgical());
    assert!(!fixes[0].is_create_file());
    assert_eq!(
        fixes[0].replacement,
        ":RELATIONS:\n:LINKS: [[01_core/101_demo_core_boundary]]\n:END:\n\n"
    );
    assert_eq!(
        fixes[0].byte_range,
        Some(ByteRange::new(insert_offset, insert_offset))
    );
}

#[test]
fn test_generate_surgical_fixes_supports_missing_package_docs_index_footer_block() {
    let doc_path = "/tmp/demo/docs/index.md".to_string();
    let file_content =
        ":RELATIONS:\n:LINKS: [[01_core/101_demo_core_boundary]]\n:END:\n".to_string();
    let mut file_contents = std::collections::HashMap::new();
    file_contents.insert(doc_path.clone(), file_content.clone());

    let insert_offset = file_content.len();
    let issues = vec![SemanticIssue {
        severity: "warning".to_string(),
        issue_type: MISSING_PACKAGE_DOCS_INDEX_FOOTER_BLOCK_ISSUE_TYPE.to_string(),
        doc: doc_path,
        node_id: "/tmp/demo/docs/index.md".to_string(),
        message: "Missing package docs footer block".to_string(),
        location: Some(IssueLocation {
            line: 3,
            heading_path: "Index Footer".to_string(),
            byte_range: Some((insert_offset, insert_offset)),
        }),
        suggestion: Some(
            "\n---\n\n:FOOTER:\n:STANDARDS: v2.0\n:LAST_SYNC: pending\n:END:\n".to_string(),
        ),
        fuzzy_suggestion: None,
    }];

    let fixes = generate_surgical_fixes(&issues, &file_contents);

    assert_eq!(fixes.len(), 1);
    assert!(fixes[0].is_surgical());
    assert!(!fixes[0].is_create_file());
    assert_eq!(
        fixes[0].replacement,
        "\n---\n\n:FOOTER:\n:STANDARDS: v2.0\n:LAST_SYNC: pending\n:END:\n"
    );
    assert_eq!(
        fixes[0].byte_range,
        Some(ByteRange::new(insert_offset, insert_offset))
    );
}
