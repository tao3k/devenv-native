use super::{SemanticIssue, build_file_reports};

#[test]
fn test_build_file_reports() {
    let issues = vec![
        SemanticIssue {
            severity: "error".to_string(),
            issue_type: "dead_link".to_string(),
            doc: "doc1.md".to_string(),
            node_id: "node1".to_string(),
            message: "Test error".to_string(),
            location: None,
            suggestion: None,
            fuzzy_suggestion: None,
        },
        SemanticIssue {
            severity: "warning".to_string(),
            issue_type: "legacy_syntax".to_string(),
            doc: "doc1.md".to_string(),
            node_id: "node2".to_string(),
            message: "Test warning".to_string(),
            location: None,
            suggestion: None,
            fuzzy_suggestion: None,
        },
        SemanticIssue {
            severity: "error".to_string(),
            issue_type: "dead_link".to_string(),
            doc: "doc2.md".to_string(),
            node_id: "node3".to_string(),
            message: "Another error".to_string(),
            location: None,
            suggestion: None,
            fuzzy_suggestion: None,
        },
    ];

    let docs = vec!["doc1.md".to_string(), "doc2.md".to_string()];
    let reports = build_file_reports(&issues, &docs);

    assert_eq!(reports.len(), 2);

    assert_eq!(reports[0].path, "doc1.md");
    assert_eq!(reports[0].error_count, 1);
    assert_eq!(reports[0].warning_count, 1);
    assert_eq!(reports[0].health_score, 75);

    assert_eq!(reports[1].path, "doc2.md");
    assert_eq!(reports[1].error_count, 1);
    assert_eq!(reports[1].warning_count, 0);
    assert_eq!(reports[1].health_score, 80);
}

#[test]
fn test_build_file_reports_deduplicates_alias_doc_paths() {
    let cwd = std::env::current_dir().unwrap_or_else(|error| panic!("cwd: {error}"));
    let temp = tempfile::tempdir_in(&cwd).unwrap_or_else(|error| panic!("tempdir: {error}"));
    let doc_path = temp.path().join("docs/index.md");
    let parent = doc_path
        .parent()
        .unwrap_or_else(|| panic!("parent directory should exist"));
    std::fs::create_dir_all(parent).unwrap_or_else(|error| panic!("create dir: {error}"));
    std::fs::write(&doc_path, "# Demo\n").unwrap_or_else(|error| panic!("write doc: {error}"));

    let absolute_path = doc_path
        .canonicalize()
        .unwrap_or_else(|error| panic!("canonicalize: {error}"))
        .to_string_lossy()
        .to_string();
    let relative_path = doc_path
        .strip_prefix(&cwd)
        .unwrap_or_else(|error| panic!("strip prefix: {error}"))
        .to_string_lossy()
        .to_string();

    let issues = vec![SemanticIssue {
        severity: "warning".to_string(),
        issue_type: "doc_identity_protocol".to_string(),
        doc: absolute_path.clone(),
        node_id: absolute_path.clone(),
        message: "Alias path warning".to_string(),
        location: None,
        suggestion: None,
        fuzzy_suggestion: None,
    }];

    let docs = vec![relative_path.clone(), absolute_path];
    let reports = build_file_reports(&issues, &docs);

    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].path, relative_path);
    assert_eq!(reports[0].warning_count, 1);
    assert_eq!(reports[0].error_count, 0);
}

#[test]
fn test_health_score_bounds() {
    let issues: Vec<SemanticIssue> = (0..10)
        .map(|_| SemanticIssue {
            severity: "error".to_string(),
            issue_type: "dead_link".to_string(),
            doc: "doc.md".to_string(),
            node_id: "node".to_string(),
            message: "Error".to_string(),
            location: None,
            suggestion: None,
            fuzzy_suggestion: None,
        })
        .collect();

    let docs = vec!["doc.md".to_string()];
    let reports = build_file_reports(&issues, &docs);

    assert_eq!(reports[0].health_score, 0);
}
