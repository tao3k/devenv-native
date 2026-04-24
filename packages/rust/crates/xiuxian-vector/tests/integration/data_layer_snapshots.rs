//! Snapshot tests for data-layer behavior and API contracts.

use anyhow::Result;
use insta::assert_json_snapshot;
use serde_json::json;
use xiuxian_vector::{
    ID_COLUMN, SearchOptions, TableColumnAlteration, TableColumnType, TableNewColumn, VectorStore,
};

use crate::snapshot_support::canonical_json;

fn round6(v: f64) -> String {
    format!("{v:.6}")
}

#[tokio::test]
async fn snapshot_data_layer_contract_v1() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("data_layer_snapshot_store");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(4)).await?;
    let table = "skills";

    store
        .add_documents(
            table,
            vec!["tool.alpha".to_string(), "tool.beta".to_string()],
            vec![vec![1.0, 0.0, 0.0, 0.0], vec![0.0, 1.0, 0.0, 0.0]],
            vec!["alpha-old".to_string(), "beta-old".to_string()],
            vec![
                json!({"kind":"seed","rank":1}).to_string(),
                json!({"kind":"seed","rank":2}).to_string(),
            ],
        )
        .await?;

    let merge_stats = store
        .merge_insert_documents(
            table,
            vec!["tool.beta".to_string(), "tool.gamma".to_string()],
            vec![vec![0.0, 1.0, 0.0, 0.0], vec![0.0, 0.0, 1.0, 0.0]],
            vec!["beta-new".to_string(), "gamma-new".to_string()],
            vec![
                json!({"kind":"merged","rank":20}).to_string(),
                json!({"kind":"merged","rank":30}).to_string(),
            ],
            ID_COLUMN,
        )
        .await?;

    let mut results = store
        .search_optimized(
            table,
            vec![0.0, 1.0, 0.0, 0.0],
            10,
            SearchOptions::default(),
        )
        .await?
        .into_iter()
        .map(|r| {
            json!({
                "id": r.id,
                "content": r.content,
                "kind": r.metadata.get("kind").and_then(|v| v.as_str()).unwrap_or(""),
                "rank": r
                    .metadata
                    .get("rank")
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or_default(),
                "distance": round6(r.distance),
            })
        })
        .collect::<Vec<_>>();
    results.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));

    let info = store.get_table_info(table).await?;
    let versions = store.list_versions(table).await?;
    let fragment_stats = store.get_fragment_stats(table).await?;

    let mut frag_view = fragment_stats
        .into_iter()
        .map(|f| {
            json!({
                "id": f.id,
                "num_rows": f.num_rows,
                "num_data_files": f.num_data_files,
                "physical_rows": f.physical_rows.unwrap_or_default(),
            })
        })
        .collect::<Vec<_>>();
    frag_view.sort_by(|a, b| a["id"].as_u64().cmp(&b["id"].as_u64()));

    let view = json!({
        "merge_stats": {
            "inserted": merge_stats.inserted,
            "updated": merge_stats.updated,
            "deleted": merge_stats.deleted,
        },
        "table": {
            "num_rows": info.num_rows,
            "fragment_count": info.fragment_count,
            "versions": versions.len(),
        },
        "search_results": results,
        "fragments": frag_view,
    });

    assert_json_snapshot!("data_layer_contract_v1", canonical_json(view));

    Ok(())
}

#[tokio::test]
async fn test_schema_evolution_guardrails_for_reserved_columns() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("schema_guardrails_store");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(4)).await?;
    let table = "skills";

    store
        .add_documents(
            table,
            vec!["tool.one".to_string()],
            vec![vec![1.0, 0.0, 0.0, 0.0]],
            vec!["one".to_string()],
            vec![json!({"kind":"seed"}).to_string()],
        )
        .await?;

    let drop_result = store
        .drop_columns(table, vec!["metadata".to_string()])
        .await;
    let Err(drop_err) = drop_result else {
        panic!("dropping reserved metadata column should fail");
    };
    assert!(drop_err.to_string().contains("reserved"));

    let alter_result = store
        .alter_columns(
            table,
            vec![TableColumnAlteration::Rename {
                path: "id".to_string(),
                new_name: "id2".to_string(),
            }],
        )
        .await;
    let Err(alter_err) = alter_result else {
        panic!("renaming reserved id column should fail");
    };
    assert!(alter_err.to_string().contains("reserved"));

    Ok(())
}

#[tokio::test]
async fn snapshot_schema_evolution_pipeline_v1() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("schema_pipeline_store");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(4)).await?;
    let table = "skills";

    store
        .add_documents(
            table,
            vec!["tool.one".to_string()],
            vec![vec![1.0, 0.0, 0.0, 0.0]],
            vec!["one".to_string()],
            vec![json!({"kind":"seed"}).to_string()],
        )
        .await?;

    let version_seed = store.get_dataset_version(table).await?;

    store
        .add_columns(
            table,
            vec![TableNewColumn {
                name: "custom_note".to_string(),
                data_type: TableColumnType::Utf8,
                nullable: true,
            }],
        )
        .await?;
    let version_added = store.get_dataset_version(table).await?;
    let ds_added = store.checkout_version(table, version_added).await?;
    assert!(ds_added.schema().field("custom_note").is_some());

    store
        .alter_columns(
            table,
            vec![TableColumnAlteration::Rename {
                path: "custom_note".to_string(),
                new_name: "custom_label".to_string(),
            }],
        )
        .await?;
    let version_renamed = store.get_dataset_version(table).await?;
    let ds_renamed = store.checkout_version(table, version_renamed).await?;
    assert!(ds_renamed.schema().field("custom_note").is_none());
    assert!(ds_renamed.schema().field("custom_label").is_some());

    store
        .drop_columns(table, vec!["custom_label".to_string()])
        .await?;
    let version_dropped = store.get_dataset_version(table).await?;
    let ds_dropped = store.checkout_version(table, version_dropped).await?;
    assert!(ds_dropped.schema().field("custom_label").is_none());

    let view = json!({
        "versions": {
            "seed": version_seed,
            "added": version_added,
            "renamed": version_renamed,
            "dropped": version_dropped,
        },
        "checks": {
            "added_custom_note": ds_added.schema().field("custom_note").is_some(),
            "renamed_custom_label": ds_renamed.schema().field("custom_label").is_some(),
            "dropped_custom_label": ds_dropped.schema().field("custom_label").is_none(),
        }
    });

    assert_json_snapshot!("schema_evolution_pipeline_v1", canonical_json(view));

    Ok(())
}
