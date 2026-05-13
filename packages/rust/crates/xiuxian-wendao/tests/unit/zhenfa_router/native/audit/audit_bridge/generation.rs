use super::{
    AuditBridge, DefaultAuditBridge, FuzzySuggestionData, IssueLocation, SemanticIssue,
    generate_batch_fixes,
};

#[test]
fn test_default_audit_bridge_generate_fixes() {
    let bridge = DefaultAuditBridge;

    let issues = vec![
        SemanticIssue {
            severity: "error".to_string(),
            issue_type: "invalid_observation_pattern".to_string(),
            doc: "docs/api.md".to_string(),
            node_id: "node1".to_string(),
            message: "Invalid pattern".to_string(),
            location: Some(IssueLocation {
                line: 42,
                heading_path: "API".to_string(),
                byte_range: None,
            }),
            suggestion: Some("Fix it".to_string()),
            fuzzy_suggestion: Some(FuzzySuggestionData {
                original_pattern: "fn process_data".to_string(),
                suggested_pattern: "fn process_records($$$)".to_string(),
                confidence: 0.85,
                source_location: Some("src/lib.rs:42".to_string()),
                replacement_drawer: r#":OBSERVE: lang:rust "fn process_records($$$)""#.to_string(),
            }),
        },
        SemanticIssue {
            severity: "error".to_string(),
            issue_type: "dead_link".to_string(),
            doc: "docs/other.md".to_string(),
            node_id: "node2".to_string(),
            message: "Dead link".to_string(),
            location: None,
            suggestion: None,
            fuzzy_suggestion: None,
        },
    ];

    let fixes = bridge.generate_fixes(&issues);

    assert_eq!(fixes.len(), 1);
    assert_eq!(fixes[0].doc_path, "docs/api.md");
}

#[test]
fn test_generate_batch_fixes_function() {
    let issues = vec![SemanticIssue {
        severity: "error".to_string(),
        issue_type: "invalid_observation_pattern".to_string(),
        doc: "docs/api.md".to_string(),
        node_id: "node1".to_string(),
        message: "Invalid pattern".to_string(),
        location: None,
        suggestion: Some("Fix it".to_string()),
        fuzzy_suggestion: Some(FuzzySuggestionData {
            original_pattern: "fn process_data".to_string(),
            suggested_pattern: "fn process_records($$$)".to_string(),
            confidence: 0.85,
            source_location: Some("src/lib.rs:42".to_string()),
            replacement_drawer: r#":OBSERVE: lang:rust "fn process_records($$$)""#.to_string(),
        }),
    }];

    let fixes = generate_batch_fixes(&issues);
    assert_eq!(fixes.len(), 1);
}
