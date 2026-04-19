//! Tests for batch module - batch structural refactoring.

use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;

use crate::{BatchConfig, StructuralEditor};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_batch_replace_apply() -> TestResult {
    let dir = TempDir::new()?;
    let file1 = dir.path().join("test1.py");
    let file2 = dir.path().join("test2.py");

    let content1 = "old_func(a, b)\n";
    let content2 = "old_func(c, d)\n";

    File::create(&file1)?.write_all(content1.as_bytes())?;

    File::create(&file2)?.write_all(content2.as_bytes())?;

    let stats = StructuralEditor::batch_replace(
        dir.path(),
        "old_func($$$ARGS)",
        "new_func($$$ARGS)",
        &BatchConfig {
            file_pattern: "**/*.py".to_string(),
            dry_run: false,
            ..Default::default()
        },
    );

    assert_eq!(stats.files_scanned, 2);
    assert_eq!(stats.files_changed, 2);
    assert_eq!(stats.replacements, 2);
    assert!(stats.errors.is_empty());
    assert_eq!(stats.modified_files.len(), 2);
    assert!(
        stats
            .modified_files
            .iter()
            .any(|path| path.ends_with("test1.py"))
    );
    assert!(
        stats
            .modified_files
            .iter()
            .any(|path| path.ends_with("test2.py"))
    );

    let modified1 = std::fs::read_to_string(&file1)?;
    let modified2 = std::fs::read_to_string(&file2)?;

    assert!(modified1.contains("new_func"));
    assert!(modified2.contains("new_func"));
    Ok(())
}

#[test]
fn test_batch_replace_python() -> TestResult {
    let dir = TempDir::new()?;
    let py_file = dir.path().join("test.py");
    let rs_file = dir.path().join("test.rs");

    let py_content = "connect(a, b)\n";
    let rs_content = "connect(c, d);\n";

    File::create(&py_file)?.write_all(py_content.as_bytes())?;

    File::create(&rs_file)?.write_all(rs_content.as_bytes())?;

    let stats = StructuralEditor::batch_replace(
        dir.path(),
        "connect($$$ARGS)",
        "safe_connect($$$ARGS)",
        &BatchConfig {
            file_pattern: "**/*.py".to_string(), // Only Python files
            dry_run: false,
            ..Default::default()
        },
    );

    assert_eq!(stats.files_scanned, 1);
    assert_eq!(stats.files_changed, 1);
    assert_eq!(stats.replacements, 1);

    let modified_py = std::fs::read_to_string(&py_file)?;
    let modified_rs = std::fs::read_to_string(&rs_file)?;

    assert!(modified_py.contains("safe_connect"));
    assert!(!modified_rs.contains("safe_connect")); // Rust file unchanged
    Ok(())
}

#[test]
fn test_matches_glob() {
    // Test glob patterns
    assert!(matches_glob(Path::new("test.py"), "*.py"));
    assert!(matches_glob(Path::new("src/test.py"), "**/*.py"));
    assert!(matches_glob(Path::new("deep/nested/test.py"), "**/*.py"));
    assert!(!matches_glob(Path::new("test.rs"), "*.py"));
}

/// Simple glob matching for testing (copied from batch module).
fn matches_glob(path: &Path, pattern: &str) -> bool {
    let path_str = path.to_string_lossy();
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();

    if pattern.starts_with("**/*") {
        let suffix = pattern.trim_start_matches("**/*");
        if suffix.is_empty() {
            return true;
        }
        path_str.ends_with(suffix) || path_str.contains(suffix)
    } else if let Some(suffix) = pattern.strip_prefix('*') {
        file_name.ends_with(suffix)
    } else if let Some((prefix, _)) = pattern.split_once('*') {
        file_name.starts_with(prefix)
    } else {
        path_str == pattern
    }
}
