use super::{
    OpenAiCompatiblePluginRerankRequestError, OpenAiCompatibleSemanticIgnition,
    OpenAiCompatibleSemanticIgnitionError,
};
use crate::link_graph::models::{QuantumAnchorHit, QuantumSemanticSearchRequest};
use xiuxian_db_store::VectorStore;

fn tempdir_or_panic() -> tempfile::TempDir {
    tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"))
}

#[tokio::test]
async fn build_plugin_rerank_request_batch_with_metadata_uses_explicit_query_vector() {
    let temp_dir = tempdir_or_panic();
    let db_path = temp_dir.path().join("openai_ignition_julia");
    let db_path_str = db_path.to_string_lossy();
    let mut store = VectorStore::new(db_path_str.as_ref(), Some(3))
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

    let ignition = OpenAiCompatibleSemanticIgnition::new(store, "anchors", "http://127.0.0.1:9999");
    let request = QuantumSemanticSearchRequest {
        query_text: Some("demo"),
        query_vector: &[9.0, 8.0, 7.0],
        candidate_limit: 1,
        min_vector_score: None,
        max_vector_score: None,
    };
    let batch = ignition
        .build_plugin_rerank_request_batch_with_metadata(
            request,
            &[QuantumAnchorHit {
                anchor_id: "doc-1#alpha".to_string(),
                vector_score: 0.31,
            }],
            "xiuxian-julia-core",
            "demo",
            "v1",
        )
        .await
        .unwrap_or_else(|error| panic!("request batch should build: {error}"));

    assert_eq!(batch.num_rows(), 1);
    assert!(batch.column_by_name("query_embedding").is_some());
    assert_eq!(
        batch.schema().metadata().get("trace_id"),
        Some(&"plugin-rerank:xiuxian-julia-core:demo".to_string())
    );
    assert_eq!(
        batch.schema().metadata().get("wendao.schema_version"),
        Some(&"v1".to_string())
    );
}

#[tokio::test]
async fn build_plugin_rerank_request_batch_rejects_missing_query_signal() {
    let temp_dir = tempdir_or_panic();
    let db_path = temp_dir.path().join("openai_ignition_julia_error");
    let db_path_str = db_path.to_string_lossy();
    let store = VectorStore::new(db_path_str.as_ref(), Some(3))
        .await
        .unwrap_or_else(|error| panic!("create vector store: {error}"));

    let ignition = OpenAiCompatibleSemanticIgnition::new(store, "anchors", "http://127.0.0.1:9999");
    let Err(error) = ignition
        .build_plugin_rerank_request_batch(
            QuantumSemanticSearchRequest {
                query_text: None,
                query_vector: &[],
                candidate_limit: 1,
                min_vector_score: None,
                max_vector_score: None,
            },
            &[QuantumAnchorHit {
                anchor_id: "doc-1#alpha".to_string(),
                vector_score: 0.31,
            }],
        )
        .await
    else {
        panic!("missing query signal should fail");
    };

    assert!(matches!(
        error,
        OpenAiCompatiblePluginRerankRequestError::Ignition(
            OpenAiCompatibleSemanticIgnitionError::MissingQuerySignal
        )
    ));
}
