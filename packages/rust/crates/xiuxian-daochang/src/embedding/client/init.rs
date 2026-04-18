use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Semaphore;

use super::super::backend::resolve_backend_settings;
use super::super::cache::EmbeddingCache;
#[cfg(feature = "agent-provider-litellm")]
use super::support::resolve_litellm_embed_api_key;
use super::support::{build_http_client, parse_positive_env_u64, parse_positive_env_usize};
use super::{
    DEFAULT_EMBED_BATCH_MAX_CONCURRENCY, DEFAULT_EMBED_BATCH_MAX_SIZE,
    DEFAULT_EMBED_CACHE_MAX_ENTRIES, DEFAULT_EMBED_CACHE_TTL_SECS, EmbeddingClient,
    MAX_EMBED_BATCH_MAX_CONCURRENCY, MAX_EMBED_BATCH_MAX_SIZE, MAX_EMBED_CACHE_MAX_ENTRIES,
    MAX_EMBED_CACHE_TTL_SECS,
};

impl EmbeddingClient {
    /// Construct a client with default backend resolution.
    #[must_use]
    pub fn new(base_url: &str, timeout_secs: u64) -> Self {
        Self::new_with_backend_and_tuning(base_url, timeout_secs, None, None, None)
    }

    /// Construct a client with explicit backend hint.
    #[must_use]
    pub fn new_with_backend(base_url: &str, timeout_secs: u64, backend_hint: Option<&str>) -> Self {
        Self::new_with_backend_and_tuning(base_url, timeout_secs, backend_hint, None, None)
    }

    /// Construct a client with explicit backend hint and batch tuning overrides.
    #[must_use]
    pub fn new_with_backend_and_tuning(
        base_url: &str,
        timeout_secs: u64,
        backend_hint: Option<&str>,
        batch_max_size_hint: Option<usize>,
        batch_max_concurrency_hint: Option<usize>,
    ) -> Self {
        Self::new_with_backend_and_tuning_impl(
            base_url,
            timeout_secs,
            backend_hint,
            batch_max_size_hint,
            batch_max_concurrency_hint,
        )
    }

    /// Construct a client with full override control for backend and batch tuning.
    #[must_use]
    fn new_with_backend_and_tuning_impl(
        base_url: &str,
        timeout_secs: u64,
        backend_hint: Option<&str>,
        batch_max_size_hint: Option<usize>,
        batch_max_concurrency_hint: Option<usize>,
    ) -> Self {
        let backend_settings = resolve_backend_settings(timeout_secs, backend_hint);
        let cache_ttl_secs = parse_positive_env_u64(
            "XIUXIAN_DAOCHANG_EMBED_CACHE_TTL_SECS",
            DEFAULT_EMBED_CACHE_TTL_SECS,
            MAX_EMBED_CACHE_TTL_SECS,
        );
        let cache_max_entries = parse_positive_env_usize(
            "XIUXIAN_DAOCHANG_EMBED_CACHE_MAX_ENTRIES",
            DEFAULT_EMBED_CACHE_MAX_ENTRIES,
            MAX_EMBED_CACHE_MAX_ENTRIES,
        );
        let batch_max_size = batch_max_size_hint
            .filter(|value| *value > 0)
            .map(|value| value.min(MAX_EMBED_BATCH_MAX_SIZE))
            .map_or_else(
                || {
                    parse_positive_env_usize(
                        "XIUXIAN_DAOCHANG_EMBED_BATCH_MAX_SIZE",
                        DEFAULT_EMBED_BATCH_MAX_SIZE,
                        MAX_EMBED_BATCH_MAX_SIZE,
                    )
                },
                std::convert::identity,
            );
        let batch_max_concurrency = batch_max_concurrency_hint
            .filter(|value| *value > 0)
            .map(|value| value.min(MAX_EMBED_BATCH_MAX_CONCURRENCY))
            .map_or_else(
                || {
                    parse_positive_env_usize(
                        "XIUXIAN_DAOCHANG_EMBED_BATCH_MAX_CONCURRENCY",
                        DEFAULT_EMBED_BATCH_MAX_CONCURRENCY,
                        MAX_EMBED_BATCH_MAX_CONCURRENCY,
                    )
                },
                std::convert::identity,
            );
        let in_flight_gate = backend_settings
            .max_in_flight
            .map(|limit| Arc::new(Semaphore::new(limit)));
        let normalized_base_url = base_url.trim_end_matches('/').to_string();
        let display_base_url = normalized_base_url.clone();
        let default_model = backend_settings.default_model.clone();
        #[cfg(feature = "agent-provider-litellm")]
        let litellm_api_key = resolve_litellm_embed_api_key();
        #[cfg(feature = "agent-provider-litellm")]
        let embed_api_key_source = litellm_api_key.source.as_str();
        #[cfg(not(feature = "agent-provider-litellm"))]
        let embed_api_key_source = "feature_disabled";
        tracing::info!(
            embed_backend = backend_settings.mode.as_str(),
            embed_backend_source = backend_settings.source,
            embed_timeout_secs = backend_settings.timeout_secs,
            embed_max_in_flight = backend_settings.max_in_flight,
            embed_batch_max_size = batch_max_size,
            embed_batch_max_concurrency = batch_max_concurrency,
            embed_base_url = display_base_url.as_str(),
            embed_default_model = default_model.as_deref().unwrap_or(""),
            has_default_model = default_model.is_some(),
            embed_api_key_source,
            "embedding backend selected"
        );
        Self {
            client: build_http_client(backend_settings.timeout_secs),
            base_url: normalized_base_url,
            cache: EmbeddingCache::new(Duration::from_secs(cache_ttl_secs), cache_max_entries),
            backend_mode: backend_settings.mode,
            backend_source: backend_settings.source,
            #[cfg(feature = "agent-provider-litellm")]
            timeout_secs: backend_settings.timeout_secs,
            max_in_flight: backend_settings.max_in_flight,
            in_flight_gate,
            batch_max_size,
            batch_max_concurrency,
            default_model,
            #[cfg(feature = "agent-provider-litellm")]
            litellm_api_key: litellm_api_key.api_key,
        }
    }
}
