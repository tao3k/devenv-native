use super::{AtomicFixBatch, BatchFix, FixPreview, FixReport};
use crate::zhenfa_router::native::audit::ByteRange;
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;

fn create_test_fix(path: &Path, line: usize, original: &str, replacement: &str) -> BatchFix {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let base_hash = blake3::hash(content.as_bytes()).to_hex().to_string();
    let byte_range = content.find(original).map_or_else(
        || ByteRange::new(0, 0),
        |s| ByteRange::new(s, s + original.len()),
    );

    BatchFix::surgical(
        path.to_string_lossy().to_string(),
        line,
        byte_range,
        base_hash,
        original.to_string(),
        replacement.to_string(),
        0.9,
    )
}

fn create_temp_dir() -> TempDir {
    match TempDir::new() {
        Ok(temp_dir) => temp_dir,
        Err(error) => panic!("Failed to create temp dir: {error}"),
    }
}

fn create_file(path: &Path) -> std::fs::File {
    match std::fs::File::create(path) {
        Ok(file) => file,
        Err(error) => panic!("Failed to create file {}: {error}", path.display()),
    }
}

fn write_file_line(file: &mut std::fs::File, content: &str) {
    if let Err(error) = writeln!(file, "{content}") {
        panic!("Failed to write test content: {error}");
    }
}

fn read_file(path: &Path) -> String {
    match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) => panic!("Failed to read {}: {error}", path.display()),
    }
}

#[test]
fn test_atomic_fix_batch_new() {
    let fixes = vec![
        BatchFix::new(
            "issue1".to_string(),
            "file1.md".to_string(),
            1,
            "old1".to_string(),
            "new1".to_string(),
            0.8,
        ),
        BatchFix::new(
            "issue2".to_string(),
            "file2.md".to_string(),
            2,
            "old2".to_string(),
            "new2".to_string(),
            0.9,
        ),
        BatchFix::new(
            "issue3".to_string(),
            "file1.md".to_string(),
            3,
            "old3".to_string(),
            "new3".to_string(),
            0.7,
        ),
    ];

    let batch = AtomicFixBatch::new(fixes);

    assert_eq!(batch.files_affected(), 2);
    assert_eq!(batch.total_fixes(), 3);
}

#[test]
fn test_confidence_threshold() {
    let fixes = vec![
        BatchFix::new(
            "i1".to_string(),
            "f1.md".to_string(),
            1,
            "a".to_string(),
            "b".to_string(),
            0.5,
        ),
        BatchFix::new(
            "i2".to_string(),
            "f1.md".to_string(),
            2,
            "c".to_string(),
            "d".to_string(),
            0.9,
        ),
    ];

    let batch = AtomicFixBatch::new(fixes).confidence_threshold(0.7);
    let filtered = batch.filter_by_confidence();

    assert_eq!(filtered.values().map(std::vec::Vec::len).sum::<usize>(), 1);
}

#[test]
fn test_dry_run_mode() {
    let temp_dir = create_temp_dir();
    let file_path = temp_dir.path().join("test.md");

    // Create test file
    let mut file = create_file(&file_path);
    write_file_line(&mut file, "Hello World");

    let fix = create_test_fix(&file_path, 1, "Hello World", "Goodbye World");
    let batch = AtomicFixBatch::new(vec![fix]).dry_run(true);

    let report = batch.apply_all();

    assert!(report.is_success());
    assert_eq!(report.files_modified, 1);

    // Verify file was NOT modified (dry run)
    let content = read_file(&file_path);
    assert!(content.contains("Hello World"));
    assert!(!content.contains("Goodbye World"));
}

#[test]
fn test_apply_all_success() {
    let temp_dir = create_temp_dir();
    let file_path = temp_dir.path().join("test.md");

    // Create test file
    let mut file = create_file(&file_path);
    write_file_line(&mut file, "line1\nHello World\nline3");

    let fix = create_test_fix(&file_path, 2, "Hello World", "Goodbye World");
    let batch = AtomicFixBatch::new(vec![fix]);

    let report = batch.apply_all();

    assert!(report.is_success());
    assert_eq!(report.successes, 1);
    assert_eq!(report.files_modified, 1);

    // Verify file WAS modified
    let content = read_file(&file_path);
    assert!(content.contains("Goodbye World"));
}

#[test]
fn test_apply_all_hash_mismatch() {
    let temp_dir = create_temp_dir();
    let file_path = temp_dir.path().join("test.md");

    // Create test file
    let mut file = create_file(&file_path);
    write_file_line(&mut file, "Hello World");

    // Create fix with wrong hash
    let fix = BatchFix::surgical(
        file_path.to_string_lossy().to_string(),
        1,
        ByteRange::new(0, 11),
        "wrong_hash".to_string(),
        "Hello World".to_string(),
        "Goodbye World".to_string(),
        0.9,
    );

    let batch = AtomicFixBatch::new(vec![fix]);
    let report = batch.apply_all();

    assert!(!report.is_success());
    assert_eq!(report.failures, 1);
}

#[test]
fn test_apply_all_creates_missing_file() {
    let temp_dir = create_temp_dir();
    let file_path = temp_dir.path().join("docs/index.md");

    let fix = BatchFix::create_file(
        file_path.to_string_lossy().to_string(),
        "# Demo: Map of Content\n".to_string(),
        1.0,
    );
    let batch = AtomicFixBatch::new(vec![fix]);

    let report = batch.apply_all();

    assert!(report.is_success());
    assert_eq!(report.successes, 1);
    assert_eq!(report.files_modified, 1);
    assert_eq!(read_file(&file_path), "# Demo: Map of Content\n");
}

#[test]
fn test_fix_preview_display() {
    let preview = FixPreview {
        line_number: 42,
        original: "old code".to_string(),
        replacement: "new code".to_string(),
        confidence: 0.85,
        is_surgical: true,
        preview_content: "file content".to_string(),
    };

    let display = format!("{preview}");
    assert!(display.contains("Line 42"));
    assert!(display.contains("85%"));
    assert!(display.contains("surgical"));
}

#[test]
fn test_fix_report_summary() {
    let mut report = FixReport {
        successes: 5,
        files_modified: 3,
        ..FixReport::default()
    };

    assert!(report.is_success());
    assert!(report.summary().starts_with("✓"));

    report.failures = 1;
    assert!(!report.is_success());
    assert!(report.summary().starts_with("✗"));
}
