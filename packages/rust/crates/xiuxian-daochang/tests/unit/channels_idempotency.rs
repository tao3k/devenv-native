#![allow(missing_docs)]

use std::time::Duration;

use xiuxian_daochang::{DEFAULT_REDIS_KEY_PREFIX, WebhookDedupBackend, WebhookDedupConfig};

#[tokio::test]
async fn memory_store_marks_duplicate_ids() -> anyhow::Result<()> {
    let store = WebhookDedupConfig::default().build_store()?;
    assert!(!store.is_duplicate(42).await?);
    assert!(store.is_duplicate(42).await?);
    Ok(())
}

#[tokio::test]
async fn memory_store_expires_ids_after_ttl() -> anyhow::Result<()> {
    const TTL_SECS: u64 = 1;
    const MAX_WAIT_SECS: u64 = 2;
    const POLL_INTERVAL_MS: u64 = 50;

    let store = WebhookDedupConfig {
        backend: WebhookDedupBackend::Memory,
        ttl_secs: TTL_SECS,
    }
    .build_store()?;
    assert!(!store.is_duplicate(7).await?);

    let wait_started = tokio::time::Instant::now();
    loop {
        if !store.is_duplicate(7).await? {
            break;
        }

        assert!(
            wait_started.elapsed() < Duration::from_secs(MAX_WAIT_SECS),
            "memory dedup entry did not expire within {MAX_WAIT_SECS}s"
        );

        tokio::time::sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    }

    Ok(())
}

#[test]
fn redis_config_normalizes_empty_prefix() {
    let config = WebhookDedupConfig {
        backend: WebhookDedupBackend::Redis {
            url: "redis://127.0.0.1:6379/0".to_string(),
            key_prefix: String::new(),
        },
        ttl_secs: 0,
    }
    .normalized();
    assert_eq!(config.ttl_secs, 1);
    match config.backend {
        WebhookDedupBackend::Redis { key_prefix, .. } => {
            assert_eq!(key_prefix, DEFAULT_REDIS_KEY_PREFIX);
        }
        WebhookDedupBackend::Memory => panic!("unexpected memory backend"),
    }
}
