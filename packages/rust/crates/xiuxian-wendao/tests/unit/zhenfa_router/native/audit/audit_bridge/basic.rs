use std::collections::HashMap;

use super::{
    BatchFix, ByteRange, FixResult, FuzzySuggestionData, IssueLocation, SemanticIssue,
    compute_hash, generate_surgical_fixes, observe_line_range, test_file_content,
};

#[test]
fn test_batch_fix_from_fuzzy_suggestion() {
    let suggestion = FuzzySuggestionData {
        original_pattern: "fn process_data($$$)".to_string(),
        suggested_pattern: "fn process_records($$$)".to_string(),
        confidence: 0.85,
        source_location: Some("src/lib.rs:42".to_string()),
        replacement_drawer: r#":OBSERVE: lang:rust "fn process_records($$$)""#.to_string(),
    };

    let fix = BatchFix::from_fuzzy_suggestion(
        "docs/api.md".to_string(),
        42,
        r#":OBSERVE: lang:rust "fn process_data($$$)""#.to_string(),
        &suggestion,
    );

    assert_eq!(fix.issue_type, "invalid_observation_pattern");
    assert_eq!(fix.doc_path, "docs/api.md");
    assert_eq!(fix.line_number, 42);
    assert!(!fix.is_surgical());
}

#[test]
fn test_batch_fix_surgical() {
    let content = test_file_content();
    let base_hash = compute_hash(&content);
    let byte_range = observe_line_range();

    let fix = BatchFix::surgical(
        "test.md".to_string(),
        2,
        byte_range,
        base_hash,
        r#":OBSERVE: lang:rust "fn process_data""#.to_string(),
        r#":OBSERVE: lang:rust "fn process_records""#.to_string(),
        0.9,
    );

    assert!(fix.is_surgical());
    assert!(fix.byte_range.is_some());
    assert!(fix.base_hash.is_some());
}

#[test]
fn test_surgical_fix_apply_success() {
    let content = test_file_content();
    let base_hash = compute_hash(&content);
    let byte_range = observe_line_range();

    let fix = BatchFix::surgical(
        "test.md".to_string(),
        2,
        byte_range,
        base_hash,
        r#":OBSERVE: lang:rust "fn process_data""#.to_string(),
        r#":OBSERVE: lang:rust "fn process_records""#.to_string(),
        0.9,
    );

    let mut content = test_file_content();
    let result = fix.apply_surgical(&mut content);

    assert_eq!(result, FixResult::Success);
    assert!(content.contains("process_records"));
}

#[test]
fn test_surgical_fix_content_at_range_mismatch() {
    let content = test_file_content();
    let base_hash = compute_hash(&content);
    let byte_range = ByteRange::new(0, 3);

    let fix = BatchFix::surgical(
        "test.md".to_string(),
        1,
        byte_range,
        base_hash,
        "old".to_string(),
        "new".to_string(),
        0.9,
    );

    let mut content = test_file_content();
    let result = fix.apply_surgical(&mut content);

    assert!(matches!(result, FixResult::ContentMismatch { .. }));
}

#[test]
fn test_surgical_fix_out_of_bounds() {
    let content = test_file_content();
    let base_hash = compute_hash(&content);
    let byte_range = ByteRange::new(100, 200);

    let fix = BatchFix::surgical(
        "test.md".to_string(),
        2,
        byte_range,
        base_hash,
        "something".to_string(),
        "replacement".to_string(),
        0.9,
    );

    let mut content = test_file_content();
    let result = fix.apply_surgical(&mut content);

    assert!(matches!(result, FixResult::OutOfBounds { .. }));
}

#[test]
fn test_surgical_fix_content_mismatch() {
    let content = test_file_content();
    let base_hash = compute_hash(&content);
    let byte_range = observe_line_range();

    let fix = BatchFix::surgical(
        "test.md".to_string(),
        2,
        byte_range,
        base_hash,
        "wrong original content".to_string(),
        "replacement".to_string(),
        0.9,
    );

    let mut content = test_file_content();
    let result = fix.apply_surgical(&mut content);

    assert!(matches!(result, FixResult::ContentMismatch { .. }));
}

#[test]
fn test_legacy_fallback() {
    let fix = BatchFix::new(
        "test".to_string(),
        "test.md".to_string(),
        1,
        "old content".to_string(),
        "new content".to_string(),
        0.9,
    );

    let mut content = "line 1\nold content\nline 3".to_string();
    let result = fix.apply_surgical(&mut content);

    assert_eq!(result, FixResult::Success);
    assert_eq!(content, "line 1\nnew content\nline 3");
}

#[test]
fn test_preview() {
    let content = test_file_content();
    let base_hash = compute_hash(&content);
    let byte_range = observe_line_range();

    let fix = BatchFix::surgical(
        "test.md".to_string(),
        2,
        byte_range,
        base_hash,
        r#":OBSERVE: lang:rust "fn process_data""#.to_string(),
        r#":OBSERVE: lang:rust "fn process_records""#.to_string(),
        0.9,
    );

    let original = test_file_content();
    let preview = match fix.preview(&original) {
        Ok(preview) => preview,
        Err(error) => panic!("preview should succeed: {error:?}"),
    };

    assert!(preview.contains("process_records"));
    assert!(!original.contains("process_records"));
}

#[test]
fn test_compute_hash_deterministic() {
    let content = "test content";
    let hash1 = compute_hash(content);
    let hash2 = compute_hash(content);

    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 64);
}

#[test]
fn test_with_surgical() {
    let content = test_file_content();
    let base_hash = compute_hash(&content);

    let fix = BatchFix::new(
        "test".to_string(),
        "test.md".to_string(),
        2,
        "old".to_string(),
        "new".to_string(),
        0.9,
    )
    .with_surgical(ByteRange::new(7, 43), base_hash);

    assert!(fix.is_surgical());
}

#[test]
fn test_generate_surgical_fixes() {
    let doc_path = "docs/api.md".to_string();
    let file_content = "line 1\n:OBSERVE: lang:rust \"fn process_data\"\nline 3".to_string();

    let mut file_contents = HashMap::new();
    file_contents.insert(doc_path.clone(), file_content.clone());

    let issues = vec![SemanticIssue {
        severity: "error".to_string(),
        issue_type: "invalid_observation_pattern".to_string(),
        doc: doc_path.clone(),
        node_id: "node1".to_string(),
        message: "Invalid pattern".to_string(),
        location: Some(IssueLocation {
            line: 2,
            heading_path: "API".to_string(),
            byte_range: Some((7, 43)),
        }),
        suggestion: Some(":OBSERVE: lang:rust \"fn process_data\"".to_string()),
        fuzzy_suggestion: Some(FuzzySuggestionData {
            original_pattern: "fn process_data".to_string(),
            suggested_pattern: "fn process_records($$$)".to_string(),
            confidence: 0.85,
            source_location: Some("src/lib.rs:42".to_string()),
            replacement_drawer: r#":OBSERVE: lang:rust "fn process_records($$$)""#.to_string(),
        }),
    }];

    let fixes = generate_surgical_fixes(&issues, &file_contents);

    assert_eq!(fixes.len(), 1);
    assert!(fixes[0].is_surgical());
    assert!(fixes[0].base_hash.is_some());
    assert_eq!(fixes[0].byte_range, Some(ByteRange::new(7, 45)));
}
