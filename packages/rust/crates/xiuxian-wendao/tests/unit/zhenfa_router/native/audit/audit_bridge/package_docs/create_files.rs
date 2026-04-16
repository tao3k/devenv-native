use super::*;

#[test]
fn test_generate_surgical_fixes_supports_missing_package_docs_index() {
    let issues = vec![SemanticIssue {
        severity: "error".to_string(),
        issue_type: MISSING_PACKAGE_DOCS_INDEX_ISSUE_TYPE.to_string(),
        doc: "/tmp/demo/docs/index.md".to_string(),
        node_id: "/tmp/demo/docs/index.md".to_string(),
        message: "Missing package docs index".to_string(),
        location: Some(IssueLocation {
            line: 1,
            heading_path: "Docs Index".to_string(),
            byte_range: None,
        }),
        suggestion: Some("# Demo: Map of Content\n".to_string()),
        fuzzy_suggestion: None,
    }];

    let fixes = generate_surgical_fixes(&issues, &std::collections::HashMap::new());

    assert_eq!(fixes.len(), 1);
    assert!(fixes[0].is_create_file());
    assert_eq!(fixes[0].replacement, "# Demo: Map of Content\n");
}

#[test]
fn test_generate_surgical_fixes_supports_missing_package_docs_tree() {
    let issues = vec![SemanticIssue {
        severity: "warning".to_string(),
        issue_type: MISSING_PACKAGE_DOCS_TREE_ISSUE_TYPE.to_string(),
        doc: "/tmp/demo/docs/index.md".to_string(),
        node_id: "/tmp/demo/docs/index.md".to_string(),
        message: "Missing package docs tree".to_string(),
        location: Some(IssueLocation {
            line: 1,
            heading_path: "Docs Bootstrap".to_string(),
            byte_range: None,
        }),
        suggestion: Some("# Demo: Map of Content\n".to_string()),
        fuzzy_suggestion: None,
    }];

    let fixes = generate_surgical_fixes(&issues, &std::collections::HashMap::new());

    assert_eq!(fixes.len(), 1);
    assert!(fixes[0].is_create_file());
    assert_eq!(fixes[0].replacement, "# Demo: Map of Content\n");
}

#[test]
fn test_generate_surgical_fixes_supports_missing_package_docs_section_landing() {
    let issues = vec![SemanticIssue {
        severity: "warning".to_string(),
        issue_type: MISSING_PACKAGE_DOCS_SECTION_LANDING_ISSUE_TYPE.to_string(),
        doc: "/tmp/demo/docs/03_features/201_demo_feature_ledger.md".to_string(),
        node_id: "/tmp/demo/docs/03_features/201_demo_feature_ledger.md".to_string(),
        message: "Missing package docs section landing".to_string(),
        location: Some(IssueLocation {
            line: 1,
            heading_path: "Feature Ledger".to_string(),
            byte_range: None,
        }),
        suggestion: Some("# Feature Ledger\n".to_string()),
        fuzzy_suggestion: None,
    }];

    let fixes = generate_surgical_fixes(&issues, &std::collections::HashMap::new());

    assert_eq!(fixes.len(), 1);
    assert!(fixes[0].is_create_file());
    assert_eq!(fixes[0].replacement, "# Feature Ledger\n");
}
