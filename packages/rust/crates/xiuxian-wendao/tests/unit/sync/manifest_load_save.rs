use tempfile::TempDir;
use xiuxian_wendao::{SyncEngine, SyncManifest};

#[test]
fn test_manifest_load_save() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let manifest_path = temp_dir.path().join("manifest.json");
    let engine = SyncEngine::new(temp_dir.path().to_path_buf(), manifest_path.clone());

    // Create initial manifest
    let mut manifest = SyncManifest::default();
    manifest
        .0
        .insert("test.py".to_string(), "hash123".to_string());

    // Save and load
    engine.save_manifest(&manifest)?;
    let loaded = engine.load_manifest();

    assert_eq!(loaded.0.get("test.py"), Some(&"hash123".to_string()));
    Ok(())
}
