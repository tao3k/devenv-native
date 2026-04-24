//! Tests for `VectorStore` path handling and dataset creation.
//!
//! These tests ensure that:
//! 1. Dataset creation works correctly even with pre-existing empty directories
//! 2. `drop_table` properly cleans up Lance-backed table data

use anyhow::Result;
use xiuxian_vector::VectorStore;

/// Helper to verify `LanceDB` directory has valid structure
fn has_lance_data(path: &std::path::Path) -> bool {
    if !path.exists() {
        return false;
    }
    // Check for LanceDB version directory or data directory
    path.join("_versions").exists() || path.join("data").exists()
}

#[tokio::test]
async fn test_create_store_with_lance_extension() -> Result<()> {
    // Test creating store with .lance extension path
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test_store.lance");
    let db_path_str = db_path.to_string_lossy();

    // Create vector store with .lance extension
    let store = VectorStore::new(db_path_str.as_ref(), Some(1536)).await?;

    // Verify table_path returns the base_path when it ends with .lance
    let table_path = store.table_path("test_table");
    assert_eq!(
        table_path, db_path,
        "table_path should return base_path when it ends with .lance"
    );

    Ok(())
}

#[tokio::test]
async fn test_add_documents_creates_dataset() -> Result<()> {
    // Test that add_documents creates the dataset properly
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test_docs.lance");
    let db_path_str = db_path.to_string_lossy();

    // Create vector store
    let store = VectorStore::new(db_path_str.as_ref(), Some(1536)).await?;

    // Add documents - this should create the dataset
    store
        .add_documents(
            "documents",
            vec!["doc1".to_string()],
            vec![vec![0.1; 1536]],
            vec!["Test content".to_string()],
            vec![r#"{"test": true}"#.to_string()],
        )
        .await?;

    // Verify the dataset was created
    let count = store.count("documents").await?;
    assert_eq!(count, 1, "Should have added one document");

    // Verify the directory now contains LanceDB data
    assert!(
        db_path.exists(),
        "The .lance directory should exist after adding documents"
    );

    // Verify LanceDB structure (_versions, data directories)
    assert!(
        has_lance_data(&db_path),
        "Directory should contain LanceDB data (_versions or data)"
    );

    Ok(())
}

#[tokio::test]
async fn test_drop_table_clears_dataset() -> Result<()> {
    // Test that drop_table properly removes the dataset
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test_drop.lance");
    let db_path_str = db_path.to_string_lossy();

    // Create store and add a document
    let mut store = VectorStore::new(db_path_str.as_ref(), Some(1536)).await?;

    store
        .add_documents(
            "test",
            vec!["id1".to_string()],
            vec![vec![0.1; 1536]],
            vec!["Content".to_string()],
            vec!["{}".to_string()],
        )
        .await?;

    // Verify data is there
    assert_eq!(store.count("test").await?, 1);

    // Drop the table
    store.drop_table("test").await?;

    // After dropping, add_documents should recreate the dataset
    store
        .add_documents(
            "test",
            vec!["id2".to_string()],
            vec![vec![0.2; 1536]],
            vec!["New content".to_string()],
            vec!["{}".to_string()],
        )
        .await?;

    // Verify data is there again
    assert_eq!(store.count("test").await?, 1);

    Ok(())
}

#[tokio::test]
async fn test_memory_mode_path_computation() -> Result<()> {
    // Test that memory mode computes paths correctly
    let store = VectorStore::new(":memory:", Some(1536)).await?;

    // Verify memory mode path computation
    let table_path = store.table_path("mem_test");
    assert_eq!(
        table_path.to_string_lossy(),
        ":memory:_mem_test",
        "Memory mode should use :memory: prefix"
    );

    Ok(())
}

#[tokio::test]
async fn test_memory_mode_add_documents_twice_without_dataset_exists_error() -> Result<()> {
    // Regression: memory mode must not try Dataset::write over an existing dataset.
    // Each store gets a unique temp path (xiuxian_lance/{id}/{table}) so no cross-run collision.
    let store = VectorStore::new(":memory:", Some(1536)).await?;

    store
        .add_documents(
            "skills",
            vec!["id1".to_string()],
            vec![vec![0.1; 1536]],
            vec!["content1".to_string()],
            vec!["{}".to_string()],
        )
        .await?;

    // Second write to same table should append/open, not fail with "Dataset already exists".
    store
        .add_documents(
            "skills",
            vec!["id2".to_string()],
            vec![vec![0.2; 1536]],
            vec!["content2".to_string()],
            vec!["{}".to_string()],
        )
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_recreate_after_empty_directory() -> Result<()> {
    // Regression test: ensure Dataset::write works even when directory exists but is empty
    // This was a bug where std::fs::create_dir_all created empty directories,
    // causing LanceDB to think a dataset already existed
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test_empty_recreate.lance");
    let db_path_str = db_path.to_string_lossy();

    // Create store
    let store = VectorStore::new(db_path_str.as_ref(), Some(1536)).await?;

    // Pre-create empty directory (simulating the bug scenario)
    std::fs::create_dir_all(&db_path)?;
    assert!(db_path.exists(), "Directory should exist");
    assert!(
        !has_lance_data(&db_path),
        "Directory should be empty (no LanceDB data)"
    );

    // Add documents - this should succeed despite empty directory
    store
        .add_documents(
            "test",
            vec!["id1".to_string()],
            vec![vec![0.1; 1536]],
            vec!["Content".to_string()],
            vec!["{}".to_string()],
        )
        .await?;

    // Verify the dataset was created correctly
    let count = store.count("test").await?;
    assert_eq!(count, 1, "Should have added one document");

    // Verify LanceDB structure exists
    assert!(
        has_lance_data(&db_path),
        "Directory should now contain LanceDB data"
    );

    Ok(())
}

#[tokio::test]
async fn test_multiple_cycles_of_drop_and_recreate() -> Result<()> {
    // Stress test: multiple cycles of drop and recreate
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test_multi_cycle.lance");

    let db_path_str = db_path.to_string_lossy();
    let mut store = VectorStore::new(db_path_str.as_ref(), Some(1536)).await?;

    for cycle in 1..=3 {
        // Add data
        store
            .add_documents(
                "test",
                vec![format!("id_cycle{cycle}")],
                vec![vec![0.1; 1536]],
                vec![format!("Content from cycle {cycle}")],
                vec!["{}".to_string()],
            )
            .await?;

        // Verify
        assert_eq!(
            store.count("test").await?,
            1,
            "Should have 1 document in cycle {cycle}"
        );

        // Drop
        store.drop_table("test").await?;

        // Verify empty
        assert_eq!(
            store.count("test").await?,
            0,
            "Should be empty after drop in cycle {cycle}"
        );
    }

    // Final add
    store
        .add_documents(
            "test",
            vec!["final_id".to_string()],
            vec![vec![0.9; 1536]],
            vec!["Final content".to_string()],
            vec!["{}".to_string()],
        )
        .await?;

    assert_eq!(store.count("test").await?, 1);

    Ok(())
}
