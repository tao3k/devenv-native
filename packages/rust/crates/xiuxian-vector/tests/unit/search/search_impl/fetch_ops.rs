use crate::VectorStore;
use anyhow::Result;

#[tokio::test]
async fn fetch_embeddings_by_ids_returns_requested_vectors() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("embedding_fetch");
    let db_path_str = db_path.to_string_lossy();
    let mut store = VectorStore::new(db_path_str.as_ref(), Some(3)).await?;

    store
        .replace_documents(
            "docs",
            vec!["doc-a".to_string(), "doc-b".to_string()],
            vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]],
            vec!["alpha".to_string(), "beta".to_string()],
            vec!["{}".to_string(), "{}".to_string()],
        )
        .await?;

    let embeddings = store
        .fetch_embeddings_by_ids("docs", &["doc-b".to_string(), "missing".to_string()])
        .await?;

    assert_eq!(embeddings.len(), 1);
    assert_eq!(embeddings.get("doc-b"), Some(&vec![4.0, 5.0, 6.0]));

    Ok(())
}

#[tokio::test]
async fn fetch_embeddings_by_ids_returns_empty_map_for_empty_request() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("embedding_fetch_empty");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(3)).await?;

    let embeddings = store.fetch_embeddings_by_ids("docs", &[]).await?;

    assert!(embeddings.is_empty());

    Ok(())
}
