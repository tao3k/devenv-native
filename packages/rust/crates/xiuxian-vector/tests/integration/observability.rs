//! Tests for Phase 5 observability: `analyze_table_health`.

use anyhow::Result;
use xiuxian_vector::{Recommendation, VectorStore};

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
async fn test_analyze_table_health_returns_report() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("health");
    let db_path_str = db_path.to_string_lossy().into_owned();
    let store = VectorStore::new(&db_path_str, Some(64)).await?;
    add_tools_table(&store, "t", 50).await?;

    let report = store.analyze_table_health("t").await?;

    assert_eq!(report.row_count, 50);
    assert!(report.fragment_count >= 1);
    assert!(report.fragmentation_ratio >= 0.0);
    assert!(!report.recommendations.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_analyze_table_health_recommends_create_indices_when_missing() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("health_idx");
    let db_path_str = db_path.to_string_lossy().into_owned();
    let store = VectorStore::new(&db_path_str, Some(64)).await?;
    add_tools_table(&store, "t", 1500).await?;

    let report = store.analyze_table_health("t").await?;

    assert!(
        report
            .recommendations
            .contains(&Recommendation::CreateIndices)
    );
    Ok(())
}

#[tokio::test]
async fn test_analyze_table_health_table_not_found() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("health_missing");
    let db_path_str = db_path.to_string_lossy().into_owned();
    let store = VectorStore::new(&db_path_str, Some(64)).await?;

    match store.analyze_table_health("nonexistent").await {
        Ok(_) => panic!("expected nonexistent table error"),
        Err(error) => {
            let msg = format!("{error}");
            assert!(msg.contains("not found") || msg.contains("Table"));
        }
    }
    Ok(())
}

#[tokio::test]
async fn test_query_metrics_in_process_record_and_read() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("qm");
    let db_path_str = db_path.to_string_lossy().into_owned();
    let store = VectorStore::new(&db_path_str, Some(64)).await?;

    let before = store.get_query_metrics("t");
    assert_eq!(before.query_count, 0);
    assert!(before.last_query_ms.is_none());

    store.record_query("t", 42);
    let after = store.get_query_metrics("t");
    assert_eq!(after.query_count, 1);
    assert_eq!(after.last_query_ms, Some(42));

    store.record_query("t", 100);
    let again = store.get_query_metrics("t");
    assert_eq!(again.query_count, 2);
    assert_eq!(again.last_query_ms, Some(100));
    Ok(())
}

/// Snapshot: table health report shape (indices and recommendations).
#[tokio::test]
async fn snapshot_observability_contract_v1() -> Result<()> {
    use insta::assert_json_snapshot;

    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("obs_snap");
    let db_path_str = db_path.to_string_lossy().into_owned();
    let store = VectorStore::new(&db_path_str, Some(64)).await?;
    add_tools_table(&store, "skills", 50).await?;

    let report = store.analyze_table_health("skills").await?;

    let view = serde_json::json!({
        "row_count": report.row_count,
        "fragment_count": report.fragment_count,
        "fragmentation_ratio": report.fragmentation_ratio,
        "indices_count": report.indices_status.len(),
        "recommendations": report.recommendations,
    });
    assert_json_snapshot!("observability_contract_v1", canonical_json(view));
    Ok(())
}
