use super::support::{err_or_panic, tempdir_or_panic};
use crate::transport::plugin_arrow_exchange::{
    PluginArrowRequestRow, PluginArrowVectorStoreRequestBatchInput,
    PluginArrowVectorStoreRequestBuildError,
    build_plugin_arrow_request_batch_from_vector_store_with_metadata,
    prepare_plugin_arrow_request_rows_from_vector_store,
};

#[tokio::test]
async fn prepare_plugin_arrow_request_rows_from_vector_store_collects_embeddings() {
    let temp_dir = tempdir_or_panic();
    let db_path = temp_dir.path().join("plugin_arrow_prepare_rows");
    let db_path_str = db_path.to_string_lossy();
    let mut store = xiuxian_db_store::VectorStore::new(db_path_str.as_ref(), Some(3))
        .await
        .unwrap_or_else(|error| panic!("create vector store: {error}"));
    store
        .replace_documents(
            "anchors",
            vec!["doc-1#alpha".to_string(), "doc-2#beta".to_string()],
            vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]],
            vec!["alpha".to_string(), "beta".to_string()],
            vec!["{}".to_string(), "{}".to_string()],
        )
        .await
        .unwrap_or_else(|error| panic!("seed vector table: {error}"));

    let rows = prepare_plugin_arrow_request_rows_from_vector_store(
        &store,
        "anchors",
        [
            ("doc-1#alpha".to_string(), 0.31),
            ("doc-2#beta".to_string(), 0.42),
        ],
    )
    .await
    .unwrap_or_else(|error| panic!("request rows should build: {error}"));

    assert_eq!(
        rows,
        vec![
            PluginArrowRequestRow {
                doc_id: "doc-1#alpha".to_string(),
                vector_score: 0.31,
                embedding: vec![1.0, 2.0, 3.0],
            },
            PluginArrowRequestRow {
                doc_id: "doc-2#beta".to_string(),
                vector_score: 0.42,
                embedding: vec![4.0, 5.0, 6.0],
            },
        ]
    );
}

#[tokio::test]
async fn build_plugin_arrow_request_batch_from_vector_store_with_metadata_sets_trace_id() {
    let temp_dir = tempdir_or_panic();
    let db_path = temp_dir.path().join("plugin_arrow_prepare_metadata");
    let db_path_str = db_path.to_string_lossy();
    let mut store = xiuxian_db_store::VectorStore::new(db_path_str.as_ref(), Some(3))
        .await
        .unwrap_or_else(|error| panic!("create vector store: {error}"));
    store
        .replace_documents(
            "anchors",
            vec!["doc-1#alpha".to_string()],
            vec![vec![1.0, 2.0, 3.0]],
            vec!["alpha".to_string()],
            vec!["{}".to_string()],
        )
        .await
        .unwrap_or_else(|error| panic!("seed vector table: {error}"));

    let batch = build_plugin_arrow_request_batch_from_vector_store_with_metadata(
        PluginArrowVectorStoreRequestBatchInput {
            store: &store,
            table_name: "anchors",
            rows: [("doc-1#alpha".to_string(), 0.25)],
            query_vector: &[9.0, 8.0, 7.0],
            provider_id: "xiuxian-wendao-julia",
            query_text: "alpha signal",
            schema_version: "v1",
        },
    )
    .await
    .unwrap_or_else(|error| panic!("request batch with metadata should build: {error}"));

    assert_eq!(
        batch.schema().metadata().get("trace_id"),
        Some(&"plugin-rerank:xiuxian-wendao-julia:alpha_signal".to_string())
    );
    assert_eq!(
        batch.schema().metadata().get("wendao.schema_version"),
        Some(&"v1".to_string())
    );
}

#[tokio::test]
async fn prepare_plugin_arrow_request_rows_from_vector_store_rejects_missing_embeddings() {
    let temp_dir = tempdir_or_panic();
    let db_path = temp_dir.path().join("plugin_arrow_prepare_missing");
    let db_path_str = db_path.to_string_lossy();
    let mut store = xiuxian_db_store::VectorStore::new(db_path_str.as_ref(), Some(3))
        .await
        .unwrap_or_else(|error| panic!("create vector store: {error}"));
    store
        .replace_documents(
            "anchors",
            vec!["doc-2#beta".to_string()],
            vec![vec![4.0, 5.0, 6.0]],
            vec!["beta".to_string()],
            vec!["{}".to_string()],
        )
        .await
        .unwrap_or_else(|error| panic!("seed vector table: {error}"));

    let error = err_or_panic(
        prepare_plugin_arrow_request_rows_from_vector_store(
            &store,
            "anchors",
            [("doc-1#alpha".to_string(), 0.31)],
        )
        .await,
        "missing embedding should fail",
    );

    assert!(matches!(
        error,
        PluginArrowVectorStoreRequestBuildError::MissingEmbedding { doc_id }
        if doc_id == "doc-1#alpha"
    ));
}
