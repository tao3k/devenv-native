#![cfg(feature = "julia")]

//! Integration tests for planned-search Julia rerank over `WendaoArrow` Flight.

use axum::routing::post;
use axum::{Json, Router};
use serde_json::json;
use serial_test::serial;
use std::fs;
use xiuxian_db_store::VectorStore;
use xiuxian_wendao::{
    LinkGraphIndex, LinkGraphSearchOptions, set_link_graph_wendao_config_override,
};
use xiuxian_wendao_builtin::{
    LinkedBuiltinWendaoArrowScoreRow as WendaoArrowScoreRow,
    linked_builtin_julia_planned_search_openai_runtime_config_toml as julia_planned_search_openai_runtime_config_toml,
    linked_builtin_spawn_wendaoarrow_custom_scoring_service as spawn_wendaoarrow_custom_scoring_service,
};

#[test]
#[serial(link_graph_runtime_config)]
fn test_planned_search_payload_applies_julia_rerank_scores()
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

    let (embedding_base_url, embedding_server_task) =
        runtime.block_on(start_embedding_mock_server());
    let (server_base_url, mut server_guard) =
        runtime.block_on(spawn_wendaoarrow_custom_scoring_service(&[
            WendaoArrowScoreRow {
                doc_id: "alpha",
                analyzer_score: 0.2,
                final_score: 0.15,
            },
            WendaoArrowScoreRow {
                doc_id: "beta",
                analyzer_score: 0.9,
                final_score: 0.97,
            },
        ]));
    let config_path = temp.path().join("wendao.toml");
    fs::write(
        &config_path,
        julia_planned_search_openai_runtime_config_toml(
            vector_store_path.to_string_lossy().as_ref(),
            embedding_base_url.as_str(),
            server_base_url.as_str(),
        ),
    )?;
    let config_path_string = config_path.to_string_lossy().to_string();
    set_link_graph_wendao_config_override(&config_path_string);

    let index = LinkGraphIndex::build(temp.path())?;
    let payload = index.search_planned_payload_with_agentic(
        "alpha signal",
        2,
        LinkGraphSearchOptions::default(),
        None,
        None,
    );
    embedding_server_task.abort();
    server_guard.kill();

    assert_eq!(
        payload
            .julia_rerank
            .as_ref()
            .map(|telemetry| telemetry.applied),
        Some(true),
        "unexpected Julia rerank telemetry: {:?}",
        payload.julia_rerank
    );
    assert_eq!(
        payload
            .julia_rerank
            .as_ref()
            .map(|telemetry| telemetry.response_row_count),
        Some(2)
    );
    assert!(
        payload
            .julia_rerank
            .as_ref()
            .and_then(|telemetry| telemetry.error.as_ref())
            .is_none()
    );
    assert_eq!(payload.quantum_contexts.len(), 2);
    assert_eq!(payload.quantum_contexts[0].doc_id, "beta");
    assert!((payload.quantum_contexts[0].saliency_score - 0.97).abs() < f64::EPSILON);
    assert_eq!(payload.quantum_contexts[1].doc_id, "alpha");
    assert!((payload.quantum_contexts[1].saliency_score - 0.15).abs() < f64::EPSILON);

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
        Err(error) => panic!("failed to bind test Julia server: {error}"),
    };
    let addr = match listener.local_addr() {
        Ok(addr) => addr,
        Err(error) => panic!("failed to resolve test Julia server address: {error}"),
    };
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    (format!("http://{addr}"), handle)
}
