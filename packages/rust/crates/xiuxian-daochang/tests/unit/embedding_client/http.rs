use std::sync::atomic::Ordering;
use std::time::Duration;

use anyhow::Result;
use xiuxian_daochang::EmbeddingClient;

use super::support::{http_vectors_for_texts, require_vectors, spawn_embedding_mock_server};

#[tokio::test]
async fn embed_batch_prefers_http_primary_path() -> Result<()> {
    let Some((base_url, http_calls, _litellm_calls)) =
        spawn_embedding_mock_server(Duration::from_millis(900), false, false).await?
    else {
        return Ok(());
    };
    let client = EmbeddingClient::new_with_backend(&base_url, 5, Some("http"));
    let texts = vec!["hello".to_string()];
    let started = std::time::Instant::now();
    let vectors = require_vectors(
        client.embed_batch_with_model(&texts, None).await,
        "expected embeddings from primary HTTP path",
    );
    let elapsed = started.elapsed();

    assert_eq!(vectors, http_vectors_for_texts(&texts));
    assert!(
        elapsed >= Duration::from_millis(700),
        "expected HTTP-first completion, got elapsed={elapsed:?}"
    );
    assert_eq!(http_calls.load(Ordering::Relaxed), 1);
    Ok(())
}

#[tokio::test]
async fn embed_batch_returns_none_when_http_fails() -> Result<()> {
    let Some((base_url, http_calls, _litellm_calls)) =
        spawn_embedding_mock_server(Duration::from_millis(5), true, false).await?
    else {
        return Ok(());
    };
    let client = EmbeddingClient::new_with_backend(&base_url, 5, Some("http"));
    let texts = vec!["hello".to_string()];
    let vectors = client.embed_batch_with_model(&texts, None).await;
    assert!(vectors.is_none());
    assert!(
        http_calls.load(Ordering::Relaxed) >= 2,
        "persistent server error should trigger at least one retry on HTTP path"
    );
    Ok(())
}

#[tokio::test]
async fn embed_batch_retries_once_on_transient_http_server_error() -> Result<()> {
    let Some((base_url, http_calls, _litellm_calls)) =
        spawn_embedding_mock_server(Duration::from_millis(5), false, true).await?
    else {
        return Ok(());
    };
    let client = EmbeddingClient::new_with_backend(&base_url, 5, Some("http"));
    let texts = vec!["hello".to_string()];
    let vectors = require_vectors(
        client.embed_batch_with_model(&texts, None).await,
        "expected embeddings from retried HTTP path",
    );

    assert_eq!(vectors, http_vectors_for_texts(&texts));
    assert_eq!(
        http_calls.load(Ordering::Relaxed),
        2,
        "transient server error should be recovered by one retry"
    );
    Ok(())
}

#[tokio::test]
async fn embed_batch_uses_http_backend_when_configured() -> Result<()> {
    let Some((base_url, http_calls, _litellm_calls)) =
        spawn_embedding_mock_server(Duration::from_millis(5), false, false).await?
    else {
        return Ok(());
    };
    let client = EmbeddingClient::new_with_backend(&base_url, 5, Some("http"));
    let texts = vec!["hello".to_string()];
    let vectors = require_vectors(
        client.embed_batch_with_model(&texts, None).await,
        "expected embeddings from http fallback path",
    );
    assert_eq!(vectors, http_vectors_for_texts(&texts));
    assert_eq!(http_calls.load(Ordering::Relaxed), 1);
    Ok(())
}
