//! Tests for Phase 2 maintenance: `auto_index_if_needed`, `compact`, `has_*_index`, bounded
//! dataset cache.

use anyhow::Result;
use xiuxian_vector::{IndexThresholds, VectorStore, ops::DatasetCacheConfig};

use crate::snapshot_support::canonical_json;

async fn add_tools_table(
    store: &VectorStore,
    table: &str,
    n: usize,
    categories: &[&str],
) -> Result<()> {
    let mut ids = Vec::with_capacity(n);
    let mut vectors = Vec::with_capacity(n);
    let mut contents = Vec::with_capacity(n);
    let mut metadatas = Vec::with_capacity(n);
    for i in 0..n {
        let cat = categories[i % categories.len()];
        let skill = format!("skill_{cat}");
        ids.push(format!("{skill}.cmd_{i}"));
        vectors.push(vec![0.1; 64]);
        contents.push(format!("content {i}"));
        metadatas.push(
            serde_json::json!({
                "skill_name": skill,
                "category": cat,
                "file_path": format!("{}/scripts/x.py", skill),
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
async fn test_has_vector_index_false_without_index() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("has_vec");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(64)).await?;
    add_tools_table(&store, "t", 50, &["a", "b"]).await?;

    let has = store.has_vector_index("t").await?;
    assert!(!has);

    Ok(())
}

#[tokio::test]
async fn test_has_vector_index_true_after_create_index() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("has_vec_after");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(64)).await?;
    add_tools_table(&store, "t", 150, &["a", "b"]).await?;
    store.create_index("t").await?;

    let has = store.has_vector_index("t").await?;
    assert!(has);

    Ok(())
}

#[tokio::test]
async fn test_has_scalar_index_true_after_create_scalar() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("has_scalar");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(64)).await?;
    add_tools_table(&store, "t", 50, &["x", "y"]).await?;
    store.create_btree_index("t", "skill_name").await?;

    let has = store.has_scalar_index("t").await?;
    assert!(has);

    Ok(())
}

#[tokio::test]
async fn test_auto_index_if_needed_returns_none_below_threshold() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("auto_low");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(64)).await?;
    add_tools_table(&store, "t", 50, &["a", "b"]).await?;

    let out = store.auto_index_if_needed("t").await?;
    assert!(out.is_none());

    Ok(())
}

#[tokio::test]
async fn test_auto_index_if_needed_creates_indices_over_threshold() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("auto_high");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(64)).await?;
    add_tools_table(&store, "t", 250, &["git", "docker", "python"]).await?;

    let out = store.auto_index_if_needed("t").await?;
    let has_vec = store.has_vector_index("t").await?;
    let has_scalar = store.has_scalar_index("t").await?;
    assert!(has_vec, "vector index should be created");
    assert!(has_scalar, "scalar indices should be created");
    assert!(out.is_some(), "should return at least one IndexStats");

    Ok(())
}

#[tokio::test]
async fn test_compact_returns_stats() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("compact");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(64)).await?;
    add_tools_table(&store, "t", 20, &["a", "b"]).await?;

    let stats = store.compact("t").await?;
    assert!(stats.fragments_before >= 1);
    assert!(stats.fragments_after >= 1);
    assert!(stats.duration_ms <= 60_000);

    Ok(())
}

#[tokio::test]
async fn test_compact_retains_only_recent_versions() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("compact_retain_versions");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(64)).await?;

    store
        .add_documents(
            "t",
            vec!["tool.a".to_string(), "tool.b".to_string()],
            vec![vec![0.1; 64], vec![0.2; 64]],
            vec!["alpha".to_string(), "beta".to_string()],
            vec![
                serde_json::json!({"kind":"seed"}).to_string(),
                serde_json::json!({"kind":"seed"}).to_string(),
            ],
        )
        .await?;
    store
        .add_documents(
            "t",
            vec!["tool.c".to_string(), "tool.d".to_string()],
            vec![vec![0.3; 64], vec![0.4; 64]],
            vec!["gamma".to_string(), "delta".to_string()],
            vec![
                serde_json::json!({"kind":"append"}).to_string(),
                serde_json::json!({"kind":"append"}).to_string(),
            ],
        )
        .await?;
    store.delete_where("t", "id = 'tool.a'").await?;

    let version_history_before = store.list_versions("t").await?;
    assert!(
        version_history_before.len() > 2,
        "test setup should create more than two versions before compaction"
    );

    let dataset_version_before = store.get_dataset_version("t").await?;
    let count_before = store.count("t").await?;

    let stats = store.compact("t").await?;
    assert!(stats.duration_ms <= 60_000);

    let version_history_after = store.list_versions("t").await?;
    let dataset_version_after = store.get_dataset_version("t").await?;
    let count_after = store.count("t").await?;

    assert!(dataset_version_after > dataset_version_before);
    assert!(
        version_history_after.len() <= 2,
        "compaction cleanup should retain only the most recent versions"
    );
    assert!(
        version_history_after
            .iter()
            .any(|version| version.version_id == dataset_version_after),
        "the current version must remain visible after cleanup"
    );
    assert_eq!(count_after, count_before);

    Ok(())
}

#[tokio::test]
async fn test_bounded_cache_eviction() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("bounded_cache");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new_with_cache_options(
        db_path_str.as_ref(),
        Some(64),
        DatasetCacheConfig {
            max_cached_tables: Some(2),
        },
    )
    .await?;
    add_tools_table(&store, "a", 5, &["x"]).await?;
    add_tools_table(&store, "b", 5, &["y"]).await?;
    add_tools_table(&store, "c", 5, &["z"]).await?;
    assert_eq!(store.count("a").await?, 5);
    assert_eq!(store.count("b").await?, 5);
    assert_eq!(store.count("c").await?, 5);

    Ok(())
}

#[tokio::test]
async fn test_create_index_background_finishes() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("bg_index");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(64)).await?;
    add_tools_table(&store, "t", 150, &["a", "b"]).await?;
    assert!(!store.has_vector_index("t").await?);

    store.create_index_background("t");
    for _ in 0..30 {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        if store.has_vector_index("t").await? {
            return Ok(());
        }
    }
    panic!("create_index_background did not create vector index within 15s")
}

#[tokio::test]
async fn test_compact_table_not_found() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("compact_missing");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(64)).await?;

    let compact_result = store.compact("nonexistent").await;
    let Err(err) = compact_result else {
        panic!("compacting a missing table should fail");
    };
    let msg = format!("{err}");
    assert!(msg.contains("not found") || msg.contains("Table"));

    Ok(())
}

/// Snapshot: `auto_index` and compact API contract (`has_*` flags and compaction shape).
#[tokio::test]
async fn snapshot_maintenance_contract_v1() -> Result<()> {
    use insta::assert_json_snapshot;

    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("maint_snap");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(64)).await?;
    add_tools_table(&store, "skills", 200, &["git", "docker", "python"]).await?;

    let auto_result = store
        .auto_index_if_needed_with_thresholds("skills", &IndexThresholds::default())
        .await?;
    let has_vector = store.has_vector_index("skills").await?;
    let has_scalar = store.has_scalar_index("skills").await?;

    let compact_stats = store.compact("skills").await?;

    let view = serde_json::json!({
        "after_auto_index": {
            "has_vector_index": has_vector,
            "has_scalar_index": has_scalar,
            "returned_stats": auto_result.is_some(),
        },
        "compaction": {
            "fragments_before": compact_stats.fragments_before,
            "fragments_after": compact_stats.fragments_after,
            "fragments_removed": compact_stats.fragments_removed,
        },
    });
    assert_json_snapshot!("maintenance_contract_v1", canonical_json(view));

    Ok(())
}
