//! Tests for scalar index creation (`BTree`, `Bitmap`) and optimal type selection.

use std::sync::{Arc, Mutex};

use anyhow::{Result, anyhow};
use xiuxian_vector::{IndexBuildProgress, IndexProgressCallback, ScalarIndexType, VectorStore};

use crate::snapshot_support::canonical_json;

async fn add_tools_table(store: &VectorStore, n: usize, categories: &[&str]) -> Result<()> {
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
        .add_documents("tools", ids, vectors, contents, metadatas)
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_create_btree_index_returns_stats() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("btree_stats");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(64)).await?;
    add_tools_table(&store, 5, &["a", "b", "c"]).await?;

    let stats = store.create_btree_index("tools", "skill_name").await?;

    assert_eq!(stats.column, "skill_name");
    assert_eq!(stats.index_type, "btree");
    assert!(
        stats.duration_ms <= 10000,
        "build should finish in reasonable time"
    );

    Ok(())
}

#[tokio::test]
async fn test_index_build_progress_callback_receives_started_and_done() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("progress_cb");
    let db_path_str = db_path.to_string_lossy();
    let events: Arc<Mutex<Vec<IndexBuildProgress>>> = Arc::new(Mutex::new(Vec::new()));
    let events_clone = Arc::clone(&events);
    let cb: IndexProgressCallback = Arc::new(move |p| {
        if let Ok(mut locked) = events_clone.lock() {
            locked.push(p);
        }
    });
    let store = VectorStore::new(db_path_str.as_ref(), Some(64))
        .await?
        .with_index_progress_callback(cb);
    add_tools_table(&store, 5, &["a", "b"]).await?;

    let _ = store.create_btree_index("tools", "skill_name").await?;

    let collected = events
        .lock()
        .map_err(|error| anyhow!("failed to lock progress events: {error}"))?;
    assert!(collected.len() >= 2, "expected Started and Done");
    match &collected[0] {
        IndexBuildProgress::Started {
            table_name,
            index_type,
        } => {
            assert_eq!(table_name, "tools");
            assert_eq!(index_type, "btree");
        }
        _ => panic!("expected Started first"),
    }
    match collected.last() {
        Some(IndexBuildProgress::Done { duration_ms }) => assert!(*duration_ms <= 10000),
        Some(_) => panic!("expected Done last"),
        None => panic!("expected Done event"),
    }

    Ok(())
}

#[tokio::test]
async fn test_create_bitmap_index_returns_stats() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("bitmap_stats");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(64)).await?;
    add_tools_table(&store, 5, &["x", "y"]).await?;

    let stats = store.create_bitmap_index("tools", "category").await?;

    assert_eq!(stats.column, "category");
    assert_eq!(stats.index_type, "bitmap");
    assert!(stats.duration_ms <= 10000);

    Ok(())
}

#[tokio::test]
async fn test_estimate_cardinality() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("cardinality");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(64)).await?;
    add_tools_table(&store, 10, &["cat_a", "cat_b", "cat_c"]).await?;

    let card = store.estimate_cardinality("tools", "category").await?;
    assert!((1..=10).contains(&card), "cardinality in 1..10, got {card}");

    Ok(())
}

#[tokio::test]
async fn test_create_optimal_scalar_index_low_cardinality_uses_bitmap() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("optimal");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(64)).await?;
    add_tools_table(&store, 50, &["low1", "low2", "low3"]).await?;

    let stats = store
        .create_optimal_scalar_index("tools", "category")
        .await?;

    assert_eq!(stats.column, "category");
    assert_eq!(stats.index_type, "bitmap");

    Ok(())
}

#[tokio::test]
async fn test_create_optimal_scalar_index_high_cardinality_uses_btree() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("optimal_btree");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(64)).await?;
    let categories: Vec<String> = (0..150).map(|i| format!("cat_{i}")).collect();
    let categories: Vec<&str> = categories.iter().map(String::as_str).collect();
    add_tools_table(&store, 150, &categories).await?;

    let stats = store
        .create_optimal_scalar_index("tools", "skill_name")
        .await?;

    assert_eq!(stats.column, "skill_name");
    assert_eq!(stats.index_type, "btree");

    Ok(())
}

#[tokio::test]
async fn test_create_scalar_index_after_add_documents() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("scalar_idx");
    let db_path_str = db_path.to_string_lossy();

    let store = VectorStore::new(db_path_str.as_ref(), Some(64)).await?;

    let metadata1 = serde_json::json!({
        "skill_name": "git",
        "category": "vcs",
        "file_path": "skills/git/SKILL.md",
        "tool_name": "commit",
    })
    .to_string();
    let metadata2 = serde_json::json!({
        "skill_name": "python",
        "category": "runtime",
        "file_path": "python/run.py",
        "tool_name": "run",
    })
    .to_string();

    store
        .add_documents(
            "tools",
            vec!["git.commit".to_string(), "python.run".to_string()],
            vec![vec![0.1; 64], vec![0.2; 64]],
            vec!["commit msg".to_string(), "run script".to_string()],
            vec![metadata1, metadata2],
        )
        .await?;

    store
        .create_scalar_index("tools", "skill_name", ScalarIndexType::BTree)
        .await?;
    store
        .create_scalar_index("tools", "category", ScalarIndexType::Bitmap)
        .await?;

    let count = store.count("tools").await?;
    assert_eq!(count, 2);

    Ok(())
}

/// Snapshot contract: `IndexStats` shape and index type selection (duration redacted for stability).
#[tokio::test]
async fn snapshot_scalar_index_stats_contract_v1() -> Result<()> {
    use insta::assert_json_snapshot;

    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("scalar_snapshot");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(64)).await?;
    add_tools_table(&store, 20, &["git", "docker", "python"]).await?;

    let btree_stats = store.create_btree_index("tools", "skill_name").await?;
    let bitmap_stats = store.create_bitmap_index("tools", "category").await?;
    let optimal_stats = store
        .create_optimal_scalar_index("tools", "category")
        .await?;

    let view = serde_json::json!({
        "btree": { "column": btree_stats.column, "index_type": btree_stats.index_type },
        "bitmap": { "column": bitmap_stats.column, "index_type": bitmap_stats.index_type },
        "optimal_category": {
            "column": optimal_stats.column,
            "index_type": optimal_stats.index_type,
        },
    });
    assert_json_snapshot!("scalar_index_stats_contract_v1", canonical_json(view));

    Ok(())
}
