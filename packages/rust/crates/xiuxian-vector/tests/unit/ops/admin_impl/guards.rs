use crate::VectorStore;
use crate::ops::DatasetCacheConfig;
use anyhow::Result;

#[tokio::test]
async fn open_table_or_err_populates_dataset_cache_for_query_paths() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("query_cache");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new_with_cache_options(
        db_path_str.as_ref(),
        Some(8),
        DatasetCacheConfig {
            max_cached_tables: Some(4),
        },
    )
    .await?;
    let schema = store.create_schema();
    let empty = lance::deps::arrow_array::RecordBatch::new_empty(schema.clone());
    store
        .replace_record_batches("cached", schema, vec![empty])
        .await?;
    {
        let mut cache = store.datasets.write().await;
        cache.remove("cached");
        assert!(!cache.contains_key("cached"));
    }

    store.open_table_or_err("cached").await?;
    {
        let cache = store.datasets.read().await;
        assert!(cache.contains_key("cached"));
        assert_eq!(cache.len(), 1);
    }

    store.open_table_or_err("cached").await?;
    {
        let cache = store.datasets.read().await;
        assert!(cache.contains_key("cached"));
        assert_eq!(cache.len(), 1);
    }

    Ok(())
}

#[tokio::test]
async fn invalidate_cached_table_drops_stale_dataset_handles() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("query_cache_invalidate");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new_with_cache_options(
        db_path_str.as_ref(),
        Some(8),
        DatasetCacheConfig {
            max_cached_tables: Some(4),
        },
    )
    .await?;
    let schema = store.create_schema();
    let empty = lance::deps::arrow_array::RecordBatch::new_empty(schema.clone());
    store
        .replace_record_batches("cached", schema, vec![empty])
        .await?;

    store.open_table_or_err("cached").await?;
    {
        let cache = store.datasets.read().await;
        assert!(cache.contains_key("cached"));
    }

    store.invalidate_cached_table("cached").await;
    {
        let cache = store.datasets.read().await;
        assert!(!cache.contains_key("cached"));
    }

    Ok(())
}
