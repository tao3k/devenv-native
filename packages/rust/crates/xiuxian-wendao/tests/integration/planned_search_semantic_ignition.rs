//! Integration tests for planned search semantic ignition payload enrichment.

use axum::Json;
use axum::Router;
use axum::routing::post;
use serde_json::json;
use serial_test::serial;
use std::fs;
use xiuxian_db_store::VectorStore;
use xiuxian_wendao::{
    LinkGraphIndex, LinkGraphSearchOptions, set_link_graph_wendao_config_override,
};

#[test]
#[serial(link_graph_runtime_config)]
fn test_planned_search_payload_includes_quantum_contexts_from_openai_semantic_ignition()
-> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let temp = tempfile::tempdir()?;
    fs::write(
        temp.path().join("alpha.md"),
        "# Alpha\n\nalpha signal remains dominant.\n",
    )?;
    fs::write(
        temp.path().join("beta.md"),
        "# Beta\n\nbeta remains as the contrasting note.\n",
    )?;

    let vector_store_path = temp.path().join("vector-store");
    let store = runtime.block_on(VectorStore::new(
        vector_store_path.to_string_lossy().as_ref(),
        Some(3),
    ))?;
    runtime.block_on(store.add_documents(
        "wendao_semantic_docs",
        vec!["alpha".to_string(), "beta".to_string()],
        vec![vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 0.0]],
        vec!["alpha anchor".to_string(), "beta anchor".to_string()],
        vec!["{}".to_string(), "{}".to_string()],
    ))?;

    let (embedding_base_url, server_task) = runtime.block_on(start_embedding_mock_server());
    let config_path = temp.path().join("wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[link_graph.retrieval]
mode = "hybrid"
candidate_multiplier = 1
max_sources = 1
graph_rows_per_source = 1

[link_graph.retrieval.semantic_ignition]
backend = "openai-compatible"
vector_store_path = "{}"
table_name = "wendao_semantic_docs"
embedding_base_url = "{}"
embedding_model = "glm-5"
"#,
            vector_store_path.to_string_lossy(),
            embedding_base_url,
        ),
    )?;
    let config_path_string = config_path.to_string_lossy().to_string();
    set_link_graph_wendao_config_override(&config_path_string);

    let index = LinkGraphIndex::build(temp.path())?;
    let payload = index.search_planned_payload_with_agentic(
        "alpha signal",
        1,
        LinkGraphSearchOptions::default(),
        None,
        None,
    );
    server_task.abort();

    assert_eq!(
        payload.hits.first().map(|hit| hit.stem.as_str()),
        Some("alpha")
    );
    assert!(!payload.quantum_contexts.is_empty());
    assert!(
        payload
            .quantum_contexts
            .iter()
            .any(|context| context.doc_id == "alpha")
    );
    assert_eq!(
        payload
            .semantic_ignition
            .as_ref()
            .map(|telemetry| telemetry.backend.as_str()),
        Some("openai_compatible")
    );
    assert_eq!(
        payload
            .semantic_ignition
            .as_ref()
            .map(|telemetry| telemetry.context_count),
        Some(payload.quantum_contexts.len())
    );
    assert!(
        payload
            .semantic_ignition
            .as_ref()
            .and_then(|telemetry| telemetry.error.as_ref())
            .is_none()
    );

    Ok(())
}

async fn start_embedding_mock_server() -> (String, tokio::task::JoinHandle<()>) {
    let app = Router::new().route(
        "/v1/embeddings",
        post(|| async {
            Json(json!({
                "data": [
                    {
                        "embedding": [1.0, 0.0, 0.0]
                    }
                ]
            }))
        }),
    );
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(error) => panic!("failed to bind embedding mock server: {error}"),
    };
    let addr = match listener.local_addr() {
        Ok(addr) => addr,
        Err(error) => panic!("failed to resolve embedding mock server address: {error}"),
    };
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    (format!("http://{addr}"), handle)
}
