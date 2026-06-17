//! Integration tests for OpenAI-compatible quantum-fusion semantic ignition.

#[cfg(feature = "llm")]
use axum::Json;
#[cfg(feature = "llm")]
use axum::Router;
#[cfg(feature = "llm")]
use axum::routing::post;
#[cfg(feature = "llm")]
use serde_json::json;
#[cfg(feature = "llm")]
use std::fs;
use tempfile::TempDir;
use xiuxian_db_store::VectorStore;
use xiuxian_wendao::link_graph::{
    OpenAiCompatibleSemanticIgnition, OpenAiCompatibleSemanticIgnitionError,
    QuantumSemanticIgnition, QuantumSemanticSearchRequest,
};
#[cfg(feature = "llm")]
use xiuxian_wendao::{LinkGraphIndex, link_graph::QuantumFusionOptions};

#[test]
fn test_openai_ignition_uses_precomputed_query_vector() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let (_temp, store, table_name) = runtime.block_on(build_test_store())?;
    let ignition = OpenAiCompatibleSemanticIgnition::new(
        store,
        table_name,
        "http://127.0.0.1:9/ignored-for-vector-path",
    );
    let query_vector = [1.0_f32, 0.0, 0.0];
    let anchors = runtime.block_on(ignition.search_anchors(QuantumSemanticSearchRequest {
        query_text: None,
        query_vector: &query_vector,
        candidate_limit: 2,
        min_vector_score: None,
        max_vector_score: None,
    }))?;

    assert_eq!(
        anchors.first().map(|item| item.anchor_id.as_str()),
        Some("alpha#h1")
    );
    Ok(())
}

#[test]
fn test_openai_ignition_requires_query_vector_or_text() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let (_temp, store, table_name) = runtime.block_on(build_test_store())?;
    let ignition = OpenAiCompatibleSemanticIgnition::new(store, table_name, "http://127.0.0.1:9");
    let empty_query_vector: [f32; 0] = [];
    let Err(error) = runtime.block_on(ignition.search_anchors(QuantumSemanticSearchRequest {
        query_text: None,
        query_vector: &empty_query_vector,
        candidate_limit: 1,
        min_vector_score: None,
        max_vector_score: None,
    })) else {
        return Err("expected MissingQuerySignal".into());
    };

    assert!(matches!(
        error,
        OpenAiCompatibleSemanticIgnitionError::MissingQuerySignal
    ));
    Ok(())
}

#[test]
#[cfg(not(feature = "llm"))]
fn test_openai_ignition_rejects_text_embedding_without_llm_feature()
-> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let (_temp, store, table_name) = runtime.block_on(build_test_store())?;
    let ignition = OpenAiCompatibleSemanticIgnition::new(store, table_name, "http://127.0.0.1:9");
    let empty_query_vector: [f32; 0] = [];
    let Err(error) = runtime.block_on(ignition.search_anchors(QuantumSemanticSearchRequest {
        query_text: Some("alpha signal"),
        query_vector: &empty_query_vector,
        candidate_limit: 1,
        min_vector_score: None,
        max_vector_score: None,
    })) else {
        return Err("expected EmbeddingProviderDisabled".into());
    };

    assert!(matches!(
        error,
        OpenAiCompatibleSemanticIgnitionError::EmbeddingProviderDisabled
    ));
    Ok(())
}

#[test]
#[cfg(feature = "llm")]
fn test_openai_ignition_embeds_query_text_with_openai_compatible_path()
-> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let (_temp, store, table_name) = runtime.block_on(build_test_store())?;
    let (base_url, server_task) = runtime.block_on(start_embedding_mock_server())?;
    let ignition = OpenAiCompatibleSemanticIgnition::new(store, table_name, base_url);
    let empty_query_vector: [f32; 0] = [];

    let anchors = runtime.block_on(ignition.search_anchors(QuantumSemanticSearchRequest {
        query_text: Some("alpha signal"),
        query_vector: &empty_query_vector,
        candidate_limit: 1,
        min_vector_score: None,
        max_vector_score: None,
    }))?;
    server_task.abort();

    assert_eq!(anchors.len(), 1);
    assert_eq!(anchors[0].anchor_id, "alpha#h1");
    Ok(())
}

#[test]
#[cfg(feature = "llm")]
fn test_quantum_contexts_from_retrieval_plan_embeds_text_only_queries()
-> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let (_index_temp, index) = build_test_index()?;
    let (_temp, store, table_name) = runtime.block_on(build_test_store())?;
    let (base_url, server_task) = runtime.block_on(start_embedding_mock_server())?;
    let ignition = OpenAiCompatibleSemanticIgnition::new(store, table_name, base_url);
    let empty_query_vector: [f32; 0] = [];

    let contexts = runtime.block_on(index.quantum_contexts_from_retrieval_plan(
        &ignition,
        Some("alpha signal"),
        &empty_query_vector,
        None,
        None,
        &QuantumFusionOptions::default(),
    ))?;
    server_task.abort();

    assert_eq!(contexts.len(), 1);
    assert_eq!(contexts[0].doc_id, "alpha");
    Ok(())
}

async fn build_test_store() -> Result<(TempDir, VectorStore, String), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let store = VectorStore::new(temp.path().to_string_lossy().as_ref(), Some(3)).await?;
    let table_name = "wendao_semantic_anchors".to_string();

    store
        .add_documents(
            &table_name,
            vec!["alpha#h1".to_string(), "beta#h1".to_string()],
            vec![vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 0.0]],
            vec![
                "alpha semantic anchor".to_string(),
                "beta semantic anchor".to_string(),
            ],
            vec!["{}".to_string(), "{}".to_string()],
        )
        .await?;

    Ok((temp, store, table_name))
}

#[cfg(feature = "llm")]
fn build_test_index() -> Result<(TempDir, LinkGraphIndex), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    fs::write(
        temp.path().join("alpha.md"),
        "# Alpha\n\nalpha semantic anchor remains dominant.\n",
    )?;
    fs::write(
        temp.path().join("beta.md"),
        "# Beta\n\nbeta semantic anchor remains secondary.\n",
    )?;
    let index = LinkGraphIndex::build(temp.path())?;
    Ok((temp, index))
}

#[cfg(feature = "llm")]
async fn start_embedding_mock_server()
-> Result<(String, tokio::task::JoinHandle<()>), Box<dyn std::error::Error>> {
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
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    Ok((format!("http://{addr}"), handle))
}
