//! Tests for Phase 4 partitioning: `suggest_partition_column` and
//! `add_documents_partitioned`.

use anyhow::Result;
use xiuxian_vector::VectorStore;

use crate::snapshot_support::canonical_json;

async fn add_tools_table(store: &VectorStore, table: &str, n: usize) -> Result<()> {
    let mut ids = Vec::with_capacity(n);
    let mut vectors = Vec::with_capacity(n);
    let mut contents = Vec::with_capacity(n);
    let mut metadatas = Vec::with_capacity(n);
    for i in 0..n {
        ids.push(format!("skill.cmd_{i}"));
        vectors.push(vec![0.1; 64]);
        contents.push(format!("content {i}"));
        metadatas.push(
            serde_json::json!({
                "skill_name": "skill",
                "category": "test",
                "file_path": "skill/scripts/x.py",
                "tool_name": format!("cmd_{i}"),
            })
            .to_string(),
        );
    }
    store
        .add_documents(table, ids, vectors, contents, metadatas)
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_suggest_partition_column_none_for_missing_table() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("part_missing");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(64)).await?;

    let out = store.suggest_partition_column("nonexistent").await?;
    assert_eq!(out, None);

    Ok(())
}

#[tokio::test]
async fn test_suggest_partition_column_none_for_small_table() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("part_small");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(64)).await?;
    add_tools_table(&store, "t", 100).await?;

    let out = store.suggest_partition_column("t").await?;
    assert_eq!(out, None);

    Ok(())
}

#[tokio::test]
async fn test_suggest_partition_column_some_for_large_tools_table() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("part_large");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(64)).await?;
    add_tools_table(&store, "t", 10_050).await?;

    let out = store.suggest_partition_column("t").await?;
    assert_eq!(out.as_deref(), Some("skill_name"));

    Ok(())
}

/// Snapshot: suggested partition column for large tools table.
#[tokio::test]
async fn snapshot_partition_suggestion_contract_v1() -> Result<()> {
    use insta::assert_json_snapshot;

    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("part_snap");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(64)).await?;
    add_tools_table(&store, "skills", 10_050).await?;

    let suggested = store.suggest_partition_column("skills").await?;

    let view = serde_json::json!({
        "suggested_column": suggested,
        "threshold": 10_000_usize,
    });
    assert_json_snapshot!("partition_suggestion_contract_v1", canonical_json(view));

    Ok(())
}

#[tokio::test]
async fn test_add_documents_partitioned_groups_by_column() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("part_write");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(64)).await?;

    let ids = vec!["a.1".to_string(), "a.2".to_string(), "b.1".to_string()];
    let vectors = vec![vec![0.1f32; 64], vec![0.2f32; 64], vec![0.3f32; 64]];
    let contents = vec!["c1".to_string(), "c2".to_string(), "c3".to_string()];
    let metadatas = vec![
        serde_json::json!({"skill_name": "a", "category": "x"}).to_string(),
        serde_json::json!({"skill_name": "a", "category": "x"}).to_string(),
        serde_json::json!({"skill_name": "b", "category": "y"}).to_string(),
    ];

    store
        .add_documents_partitioned("t", "skill_name", ids, vectors, contents, metadatas)
        .await?;

    let n = store.count("t").await?;
    assert_eq!(n, 3);

    let info = store.get_table_info("t").await?;
    assert_eq!(
        info.num_rows, 3,
        "table info should report every persisted partitioned row"
    );
    assert!(
        info.fragment_count >= 1,
        "partitioned write should leave at least one persisted fragment"
    );

    Ok(())
}
