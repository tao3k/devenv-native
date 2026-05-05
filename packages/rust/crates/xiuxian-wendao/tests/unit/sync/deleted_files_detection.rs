use std::fs;

use tempfile::TempDir;
use xiuxian_wendao::{SyncEngine, SyncManifest};

#[test]
fn test_deleted_files_detection() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let manifest_path = temp_dir.path().join("manifest.json");
    let engine = SyncEngine::new(temp_dir.path().to_path_buf(), manifest_path.clone());

    // Create old manifest with files that don't exist on disk
    let mut old_manifest = SyncManifest::default();
    old_manifest
        .0
        .insert("deleted1.py".to_string(), "hash1".to_string());
    old_manifest
        .0
        .insert("deleted2.rs".to_string(), "hash2".to_string());
    old_manifest.0.insert(
        "still_exists.py".to_string(),
        SyncEngine::compute_hash("exists"),
    );

    // Create file for still_exists
    fs::write(temp_dir.path().join("still_exists.py"), "exists")?;

    let files = engine.discover_files();
    let diff = engine.compute_diff(&old_manifest, &files);

    // deleted1.py should be in deleted
    assert!(
        diff.deleted
            .iter()
            .any(|p| p.file_name().is_some_and(|n| n == "deleted1.py"))
    );
    // deleted2.rs should be in deleted
    assert!(
        diff.deleted
            .iter()
            .any(|p| p.file_name().is_some_and(|n| n == "deleted2.rs"))
    );
    Ok(())
}
