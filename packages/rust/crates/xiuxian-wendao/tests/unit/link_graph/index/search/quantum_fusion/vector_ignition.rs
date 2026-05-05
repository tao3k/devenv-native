use super::VectorStoreSemanticIgnition;
use crate::link_graph::models::{QuantumAnchorHit, QuantumSemanticSearchRequest};
use xiuxian_db_store::VectorStore;

fn tempdir_or_panic() -> tempfile::TempDir {
    tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"))
}

#[tokio::test]
async fn build_plugin_rerank_request_batch_with_metadata_uses_anchor_ids_as_request_doc_ids() {
    let temp_dir = tempdir_or_panic();
    let db_path = temp_dir.path().join("vector_ignition_julia");
    let db_path_str = db_path.to_string_lossy();
    let mut store = VectorStore::new(db_path_str.as_ref(), Some(3))
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

    let ignition = VectorStoreSemanticIgnition::new(store, "anchors");
    let request = QuantumSemanticSearchRequest {
        query_text: Some("demo"),
        query_vector: &[9.0, 8.0, 7.0],
        candidate_limit: 2,
        min_vector_score: None,
        max_vector_score: None,
    };
    let batch = ignition
        .build_plugin_rerank_request_batch_with_metadata(
            request,
            &[
                QuantumAnchorHit {
                    anchor_id: "doc-1#alpha".to_string(),
                    vector_score: 0.31,
                },
                QuantumAnchorHit {
                    anchor_id: "doc-2#beta".to_string(),
                    vector_score: 0.42,
                },
            ],
            "xiuxian-wendao-julia",
            "demo",
            "v1",
        )
        .await
        .unwrap_or_else(|error| panic!("request batch should build: {error}"));

    let Some(doc_ids) = batch
        .column_by_name("doc_id")
        .and_then(|column| column.as_any().downcast_ref::<arrow::array::StringArray>())
    else {
        panic!("doc_id column");
    };
    assert_eq!(doc_ids.value(0), "doc-1#alpha");
    assert_eq!(doc_ids.value(1), "doc-2#beta");
    assert_eq!(
        batch.schema().metadata().get("trace_id"),
        Some(&"plugin-rerank:xiuxian-wendao-julia:demo".to_string())
    );
}
