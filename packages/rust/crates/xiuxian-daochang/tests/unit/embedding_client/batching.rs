use std::sync::atomic::Ordering;
use std::time::Duration;

use anyhow::Result;
use xiuxian_daochang::EmbeddingClient;

use super::support::{http_vectors_for_texts, require_vectors, spawn_embedding_mock_server};

#[tokio::test]
async fn embed_batch_splits_payload_by_chunk_size_and_preserves_order() -> Result<()> {
    let Some((base_url, http_calls, _litellm_calls)) =
        spawn_embedding_mock_server(Duration::from_millis(5), false, false).await?
    else {
        return Ok(());
    };
    let client =
        EmbeddingClient::new_with_backend_and_tuning(&base_url, 5, Some("http"), Some(2), Some(1));
    let texts = vec![
        "chunk-0".to_string(),
        "chunk-1".to_string(),
        "chunk-2".to_string(),
        "chunk-3".to_string(),
        "chunk-4".to_string(),
    ];
    let vectors = require_vectors(
        client.embed_batch_with_model(&texts, None).await,
        "chunked embedding should succeed",
    );
    assert_eq!(vectors, http_vectors_for_texts(&texts));
    assert_eq!(
        http_calls.load(Ordering::Relaxed),
        3,
        "5 texts with chunk_size=2 should trigger 3 HTTP calls"
    );
    Ok(())
}

#[tokio::test]
async fn embed_batch_chunk_concurrency_reduces_wall_time() -> Result<()> {
    let texts = vec![
        "alpha".to_string(),
        "bravo".to_string(),
        "charlie".to_string(),
        "delta".to_string(),
        "echo".to_string(),
        "foxtrot".to_string(),
    ];

    let Some((seq_url, seq_http_calls, _seq_litellm_calls)) =
        spawn_embedding_mock_server(Duration::from_millis(200), false, false).await?
    else {
        return Ok(());
    };
    let seq_client =
        EmbeddingClient::new_with_backend_and_tuning(&seq_url, 5, Some("http"), Some(2), Some(1));
    let seq_started = std::time::Instant::now();
    let seq_vectors = require_vectors(
        seq_client.embed_batch_with_model(&texts, None).await,
        "sequential chunked embedding should succeed",
    );
    let seq_elapsed = seq_started.elapsed();
    assert_eq!(seq_vectors, http_vectors_for_texts(&texts));
    assert_eq!(seq_http_calls.load(Ordering::Relaxed), 3);

    let Some((con_url, con_http_calls, _con_litellm_calls)) =
        spawn_embedding_mock_server(Duration::from_millis(200), false, false).await?
    else {
        return Ok(());
    };
    let con_client =
        EmbeddingClient::new_with_backend_and_tuning(&con_url, 5, Some("http"), Some(2), Some(3));
    let con_started = std::time::Instant::now();
    let con_vectors = require_vectors(
        con_client.embed_batch_with_model(&texts, None).await,
        "concurrent chunked embedding should succeed",
    );
    let con_elapsed = con_started.elapsed();
    assert_eq!(con_vectors, http_vectors_for_texts(&texts));
    assert_eq!(con_http_calls.load(Ordering::Relaxed), 3);

    assert!(
        con_elapsed + Duration::from_millis(180) < seq_elapsed,
        "expected concurrent chunk execution to reduce wall time (seq={seq_elapsed:?}, concurrent={con_elapsed:?})"
    );
    Ok(())
}
