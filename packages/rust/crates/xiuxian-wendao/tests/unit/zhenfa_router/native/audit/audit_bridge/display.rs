use super::*;

#[test]
fn test_fix_result_display_success() {
    let result = FixResult::Success;
    assert_eq!(format!("{result}"), "Fix applied successfully");
}

#[test]
fn test_fix_result_display_hash_mismatch() {
    let result = FixResult::HashMismatch {
        expected: "a1b2c3d4e5f6".to_string(),
        actual: "x1y2z3a4b5c6".to_string(),
    };
    let display = format!("{result}");
    assert!(display.contains("Hash mismatch"));
    assert!(display.contains("a1b2c3d4"));
    assert!(display.contains("x1y2z3a4"));
}

#[test]
fn test_fix_result_display_out_of_bounds() {
    let result = FixResult::OutOfBounds {
        range: ByteRange::new(100, 200),
        file_size: 50,
    };
    let display = format!("{result}");
    assert!(display.contains("Byte range"));
    assert!(display.contains("exceeds file size"));
    assert!(display.contains("50"));
}

#[test]
fn test_fix_result_display_content_mismatch() {
    let result = FixResult::ContentMismatch {
        expected: "expected content".to_string(),
        actual: "actual content".to_string(),
    };
    let display = format!("{result}");
    assert!(display.contains("Content mismatch"));
    assert!(display.contains("expected"));
    assert!(display.contains("actual"));
}
