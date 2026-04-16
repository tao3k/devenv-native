use super::*;

#[test]
fn test_surgical_fix_empty_content() {
    let content = "";
    let base_hash = compute_hash(content);
    let byte_range = ByteRange::new(0, 0);

    let fix = BatchFix::surgical(
        "test.md".to_string(),
        1,
        byte_range,
        base_hash,
        String::new(),
        "new content".to_string(),
        0.9,
    );

    let mut content = String::new();
    let result = fix.apply_surgical(&mut content);

    assert_eq!(result, FixResult::Success);
    assert_eq!(content, "new content");
}

#[test]
fn test_surgical_fix_same_content_replacement() {
    let content = test_file_content();
    let base_hash = compute_hash(&content);
    let byte_range = observe_line_range();
    let observe_content = r#":OBSERVE: lang:rust "fn process_data""#;

    let fix = BatchFix::surgical(
        "test.md".to_string(),
        2,
        byte_range,
        base_hash,
        observe_content.to_string(),
        observe_content.to_string(),
        0.9,
    );

    let mut content = test_file_content();
    let result = fix.apply_surgical(&mut content);

    assert_eq!(result, FixResult::Success);
    assert_eq!(content, test_file_content());
}

#[test]
fn test_surgical_fix_byte_range_at_file_boundary() {
    let content = "test content";
    let base_hash = compute_hash(content);
    let byte_range = ByteRange::new(0, 12);

    let fix = BatchFix::surgical(
        "test.md".to_string(),
        1,
        byte_range,
        base_hash,
        "test content".to_string(),
        "replaced all".to_string(),
        0.9,
    );

    let mut content = "test content".to_string();
    let result = fix.apply_surgical(&mut content);

    assert_eq!(result, FixResult::Success);
    assert_eq!(content, "replaced all");
}

#[test]
fn test_surgical_fix_start_equals_end() {
    let content = test_file_content();
    let base_hash = compute_hash(&content);
    let byte_range = ByteRange::new(7, 7);

    let fix = BatchFix::surgical(
        "test.md".to_string(),
        2,
        byte_range,
        base_hash,
        String::new(),
        "inserted".to_string(),
        0.9,
    );

    let mut content = test_file_content();
    let result = fix.apply_surgical(&mut content);

    assert_eq!(result, FixResult::Success);
    assert!(content.contains("inserted"));
}

#[test]
fn test_legacy_fallback_not_found() {
    let fix = BatchFix::new(
        "test".to_string(),
        "test.md".to_string(),
        1,
        "nonexistent".to_string(),
        "new".to_string(),
        0.9,
    );

    let mut content = "some other content".to_string();
    let result = fix.apply_surgical(&mut content);

    assert!(matches!(result, FixResult::ContentMismatch { .. }));
}

#[test]
fn test_preview_error() {
    let content = "different content";
    let base_hash = compute_hash(content);

    let fix = BatchFix::surgical(
        "test.md".to_string(),
        1,
        ByteRange::new(0, 3),
        base_hash,
        "old".to_string(),
        "new".to_string(),
        0.9,
    );

    let result = fix.preview(content);

    match result {
        Err(FixResult::ContentMismatch { .. }) => {}
        Err(other) => panic!("unexpected preview error: {other:?}"),
        Ok(preview) => panic!("preview should fail, got: {preview}"),
    }
}

#[test]
fn test_surgical_fix_multibyte_utf8() {
    let content = "line 1\n:OBSERVE: lang:rust \"fn 处理数据\"\nline 3";
    let base_hash = compute_hash(content);

    let observe = r#":OBSERVE: lang:rust "fn 处理数据""#;
    let Some(start) = content.find(observe) else {
        panic!("expected to find multibyte observation");
    };
    let end = start + observe.len();

    let fix = BatchFix::surgical(
        "test.md".to_string(),
        2,
        ByteRange::new(start, end),
        base_hash,
        observe.to_string(),
        r#":OBSERVE: lang:rust "fn process_data""#.to_string(),
        0.9,
    );

    let mut content = content.to_string();
    let result = fix.apply_surgical(&mut content);

    assert_eq!(result, FixResult::Success);
    assert!(content.contains("process_data"));
}

#[test]
fn test_is_surgical_method() {
    let non_surgical = BatchFix::new(
        "test".to_string(),
        "test.md".to_string(),
        1,
        "old".to_string(),
        "new".to_string(),
        0.9,
    );
    assert!(!non_surgical.is_surgical());

    let surgical = BatchFix::surgical(
        "test.md".to_string(),
        1,
        ByteRange::new(0, 3),
        "hash".to_string(),
        "old".to_string(),
        "new".to_string(),
        0.9,
    );
    assert!(surgical.is_surgical());

    let partial = BatchFix {
        issue_type: "test".to_string(),
        doc_path: "test.md".to_string(),
        line_number: 1,
        original_content: "old".to_string(),
        replacement: "new".to_string(),
        confidence: 0.9,
        source_location: None,
        mode: BatchFixMode::Replace,
        base_hash: None,
        byte_range: Some(ByteRange::new(0, 3)),
    };
    assert!(!partial.is_surgical());
}
