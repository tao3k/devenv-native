//! Integration tests for memory embedding runtime guard behavior.

use std::time::Duration;

use xiuxian_llm::embedding::{
    EMBEDDING_SOURCE_EMBEDDING_REPAIRED, EmbeddingRuntime, MemoryEmbeddingErrorKind,
};

#[tokio::test]
async fn runtime_repairs_dimension_mismatch() {
    let runtime = EmbeddingRuntime::new(Duration::from_millis(50), Duration::from_millis(100));

    let result = runtime
        .embed_with_source("intent", 4, |_| async { Some(vec![0.2, 0.3]) })
        .await
        .unwrap_or_else(|error| panic!("embedding request should succeed: {error:?}"));

    assert_eq!(result.1, EMBEDDING_SOURCE_EMBEDDING_REPAIRED);
    assert_eq!(result.0.len(), 4);
}

#[tokio::test]
async fn runtime_enters_cooldown_after_timeout() {
    let runtime = EmbeddingRuntime::new(Duration::from_millis(10), Duration::from_millis(100));

    let first = runtime
        .embed_with_source("intent", 4, |_| async {
            tokio::time::sleep(Duration::from_millis(30)).await;
            Some(vec![0.1, 0.2, 0.3, 0.4])
        })
        .await;
    assert_eq!(first, Err(MemoryEmbeddingErrorKind::Timeout));

    let second = runtime
        .embed_with_source("intent", 4, |_| async { Some(vec![0.1, 0.2, 0.3, 0.4]) })
        .await;
    assert_eq!(second, Err(MemoryEmbeddingErrorKind::CooldownActive));
}

#[tokio::test]
async fn runtime_maps_missing_vector_to_unavailable() {
    let runtime = EmbeddingRuntime::new(Duration::from_millis(50), Duration::from_millis(100));

    let result = runtime
        .embed_with_source("intent", 4, |_| async { None })
        .await;
    assert_eq!(result, Err(MemoryEmbeddingErrorKind::Unavailable));
}
