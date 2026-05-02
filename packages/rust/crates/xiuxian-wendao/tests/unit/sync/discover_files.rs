use std::fs;

use tempfile::TempDir;
use xiuxian_wendao::SyncEngine;

#[test]
fn test_discover_files() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;

    // Create test files
    fs::write(temp_dir.path().join("test.py"), "print('hello')")?;
    fs::write(temp_dir.path().join("test.md"), "# Hello")?;
    fs::write(temp_dir.path().join("test.txt"), "hello")?; // Should be skipped

    // Create subdirectory with file
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir_all(&subdir)?;
    fs::write(subdir.join("module.py"), "def foo(): pass")?;

    let manifest_path = temp_dir.path().join("manifest.json");
    let engine = SyncEngine::new(temp_dir.path().to_path_buf(), manifest_path.clone());
    let files = engine.discover_files();

    // Should find .py and .md files, not .txt
    assert!(
        files
            .iter()
            .any(|p| p.extension().is_some_and(|e| e == "py"))
    );
    assert!(
        files
            .iter()
            .any(|p| p.extension().is_some_and(|e| e == "md"))
    );
    assert!(
        !files
            .iter()
            .any(|p| p.extension().is_some_and(|e| e == "txt"))
    );
    Ok(())
}
