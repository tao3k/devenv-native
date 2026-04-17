use std::sync::atomic::Ordering;
use std::time::Duration;

use anyhow::Result;
use xiuxian_daochang::EmbeddingClient;

use super::support::{
    http_vectors_for_texts, openai_vectors_for_texts, require_vectors, spawn_embedding_mock_server,
    spawn_embedding_mock_server_with_openai_failure,
};

#[tokio::test]
async fn embed_batch_litellm_ollama_prefers_openai_http_direct_path() -> Result<()> {
    let Some((base_url, http_calls, litellm_calls)) =
        spawn_embedding_mock_server(Duration::from_millis(5), false, false).await?
    else {
        return Ok(());
    };
    let client = EmbeddingClient::new_with_backend(&base_url, 5, Some("litellm_rs"));
    let texts = vec!["hello".to_string()];
    let vectors = require_vectors(
        client
            .embed_batch_with_model(&texts, Some("ollama/qwen3-embedding:0.6b"))
            .await,
        "expected embeddings from OpenAI-compatible direct path",
    );
    assert_eq!(vectors, openai_vectors_for_texts(&texts));
    assert_eq!(http_calls.load(Ordering::Relaxed), 0);
    assert_eq!(
        litellm_calls.load(Ordering::Relaxed),
        1,
        "ollama direct path should call /v1/embeddings once",
    );
    Ok(())
}

#[tokio::test]
async fn embed_batch_openai_backend_uses_v1_embeddings_endpoint() -> Result<()> {
    let Some((base_url, http_calls, litellm_calls)) =
        spawn_embedding_mock_server(Duration::from_millis(5), false, false).await?
    else {
        return Ok(());
    };
    let client = EmbeddingClient::new_with_backend(&base_url, 5, Some("openai_http"));
    let texts = vec!["hello".to_string()];
    let vectors = require_vectors(
        client
            .embed_batch_with_model(&texts, Some("qwen3-embedding:0.6b"))
            .await,
        "expected embeddings from /v1/embeddings",
    );
    assert_eq!(vectors, openai_vectors_for_texts(&texts));
    assert_eq!(http_calls.load(Ordering::Relaxed), 0);
    assert_eq!(litellm_calls.load(Ordering::Relaxed), 1);
    Ok(())
}

#[cfg(feature = "agent-provider-litellm")]
#[tokio::test]
async fn embed_batch_litellm_mistral_falls_back_to_http_when_provider_fails() -> Result<()> {
    let Some((base_url, http_calls, litellm_calls)) =
        spawn_embedding_mock_server_with_openai_failure(
            Duration::from_millis(5),
            false,
            false,
            true,
        )
        .await?
    else {
        return Ok(());
    };
    let client = EmbeddingClient::new_with_backend(&base_url, 5, Some("litellm_rs"));
    let texts = vec!["hello".to_string()];
    let vectors = require_vectors(
        client
            .embed_batch_with_model(&texts, Some("mistral/mistral-embed"))
            .await,
        "expected embeddings from /embed/batch fallback",
    );

    assert_eq!(vectors, http_vectors_for_texts(&texts));
    assert_eq!(
        http_calls.load(Ordering::Relaxed),
        1,
        "expected one /embed/batch fallback request"
    );
    assert!(
        litellm_calls.load(Ordering::Relaxed) <= 1,
        "provider path should be attempted at most once before http fallback"
    );
    Ok(())
}

#[cfg(feature = "agent-provider-litellm")]
#[tokio::test]
async fn embed_batch_litellm_mistral_returns_none_when_provider_and_http_fail() -> Result<()> {
    let Some((base_url, http_calls, litellm_calls)) =
        spawn_embedding_mock_server_with_openai_failure(
            Duration::from_millis(5),
            true,
            false,
            true,
        )
        .await?
    else {
        return Ok(());
    };
    let client = EmbeddingClient::new_with_backend(&base_url, 5, Some("litellm_rs"));
    let texts = vec!["hello".to_string()];
    let vectors = client
        .embed_batch_with_model(&texts, Some("mistral/mistral-embed"))
        .await;

    assert!(vectors.is_none());
    assert!(
        http_calls.load(Ordering::Relaxed) >= 1,
        "expected /embed/batch fallback attempts when provider path fails"
    );
    assert!(
        litellm_calls.load(Ordering::Relaxed) <= 1,
        "provider path should be attempted at most once before fallback chain completes"
    );
    Ok(())
}

#[cfg(feature = "agent-provider-litellm")]
#[tokio::test]
async fn embed_batch_litellm_ollama_direct_path_ignores_embed_batch_errors() -> Result<()> {
    let Some((base_url, http_calls, litellm_calls)) =
        spawn_embedding_mock_server(Duration::from_millis(5), true, false).await?
    else {
        return Ok(());
    };
    let client = EmbeddingClient::new_with_backend(&base_url, 5, Some("litellm_rs"));
    let texts = vec!["hello".to_string()];
    let vectors = require_vectors(
        client
            .embed_batch_with_model(&texts, Some("ollama/qwen3-embedding:0.6b"))
            .await,
        "expected embeddings from OpenAI-compatible fallback path",
    );

    assert_eq!(vectors, openai_vectors_for_texts(&texts));
    assert_eq!(
        http_calls.load(Ordering::Relaxed),
        0,
        "ollama direct path should skip /embed/batch when OpenAI-compatible endpoint is available"
    );
    assert_eq!(litellm_calls.load(Ordering::Relaxed), 1);
    Ok(())
}

#[cfg(feature = "agent-provider-litellm")]
#[tokio::test]
async fn embed_batch_litellm_ollama_returns_none_when_all_primary_paths_fail() -> Result<()> {
    let Some((base_url, http_calls, litellm_calls)) =
        spawn_embedding_mock_server_with_openai_failure(
            Duration::from_millis(5),
            true,
            false,
            true,
        )
        .await?
    else {
        return Ok(());
    };
    let client = EmbeddingClient::new_with_backend(&base_url, 5, Some("litellm_rs"));
    let texts = vec!["hello".to_string()];
    let vectors = client
        .embed_batch_with_model(&texts, Some("ollama/qwen3-embedding:0.6b"))
        .await;

    assert!(vectors.is_none());
    assert!(
        http_calls.load(Ordering::Relaxed) >= 1,
        "expected /embed/batch fallback attempts before marking embedding unavailable"
    );
    assert!(
        litellm_calls.load(Ordering::Relaxed) >= 1,
        "expected OpenAI-compatible path to be attempted before failure"
    );
    Ok(())
}
