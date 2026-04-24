//! Unbounded session store backed by in-memory state with optional Valkey sync.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::RwLock;

use crate::observability::SessionEvent;

use super::message::ChatMessage;
use super::redis_backend::{RedisSessionBackend, RedisSessionRuntimeSnapshot};

/// Unbounded session store used when bounded turn windows are disabled.
#[derive(Clone)]
pub struct SessionStore {
    inner: Arc<RwLock<HashMap<String, Vec<ChatMessage>>>>,
    redis: Option<Arc<RedisSessionBackend>>,
}

impl SessionStore {
    fn from_redis_backend(redis: Option<Arc<RedisSessionBackend>>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            redis,
        }
    }

    #[doc(hidden)]
    #[must_use]
    pub fn new_in_memory_for_test() -> Self {
        Self::from_redis_backend(None)
    }

    /// Create a store using runtime settings or env-driven Valkey fallback.
    ///
    /// # Errors
    /// Returns an error when Valkey-backed runtime initialization fails.
    pub fn new() -> Result<Self> {
        let redis = match RedisSessionBackend::from_env() {
            Some(Ok(backend)) => {
                tracing::info!(
                    event = SessionEvent::SessionBackendEnabled.as_str(),
                    key_prefix = %backend.key_prefix(),
                    ttl_secs = ?backend.ttl_secs(),
                    message_content_max_chars = ?backend.runtime_snapshot().message_content_max_chars,
                    "session store backend enabled: valkey"
                );
                Some(Arc::new(backend))
            }
            Some(Err(error)) => {
                return Err(error).context("failed to initialize valkey session store");
            }
            None => None,
        };
        Ok(Self::from_redis_backend(redis))
    }

    /// Create a store with explicit Valkey backend parameters.
    ///
    /// # Errors
    /// Returns an error when Valkey backend creation fails.
    pub fn new_with_redis(
        redis_url: impl Into<String>,
        key_prefix: Option<String>,
        ttl_secs: Option<u64>,
    ) -> Result<Self> {
        let backend = RedisSessionBackend::new_from_parts(redis_url.into(), key_prefix, ttl_secs)?;
        Ok(Self::from_redis_backend(Some(Arc::new(backend))))
    }

    pub(crate) fn runtime_snapshot(&self) -> Option<RedisSessionRuntimeSnapshot> {
        self.redis
            .as_ref()
            .map(|backend| backend.runtime_snapshot())
    }

    pub(crate) fn redis_runtime_snapshot(&self) -> Option<RedisSessionRuntimeSnapshot> {
        self.runtime_snapshot()
    }

    /// Append messages to the end of a session.
    ///
    /// # Errors
    /// Returns an error when Valkey-backed persistence fails.
    pub async fn append(&self, session_id: &str, messages: Vec<ChatMessage>) -> Result<()> {
        if messages.is_empty() {
            return Ok(());
        }
        if let Some(redis) = &self.redis {
            redis.append_messages(session_id, &messages).await?;
        }
        let mut inner = self.inner.write().await;
        inner
            .entry(session_id.to_string())
            .or_default()
            .extend(messages);
        Ok(())
    }

    /// Replace the stored session messages.
    ///
    /// # Errors
    /// Returns an error when Valkey-backed persistence fails.
    pub async fn replace(&self, session_id: &str, messages: Vec<ChatMessage>) -> Result<()> {
        if let Some(redis) = &self.redis {
            let _ = redis.replace_messages(session_id, &messages).await?;
        }
        let mut inner = self.inner.write().await;
        if messages.is_empty() {
            inner.remove(session_id);
        } else {
            inner.insert(session_id.to_string(), messages);
        }
        Ok(())
    }

    /// Read the full message list for a session.
    ///
    /// # Errors
    /// Returns an error when Valkey-backed persistence fails.
    pub async fn get(&self, session_id: &str) -> Result<Vec<ChatMessage>> {
        if let Some(redis) = &self.redis {
            let messages = redis.get_messages(session_id).await?;
            let mut inner = self.inner.write().await;
            if messages.is_empty() {
                inner.remove(session_id);
            } else {
                inner.insert(session_id.to_string(), messages.clone());
            }
            return Ok(messages);
        }
        let inner = self.inner.read().await;
        Ok(inner.get(session_id).cloned().unwrap_or_default())
    }

    /// Count stored messages for a session.
    ///
    /// # Errors
    /// Returns an error when Valkey-backed persistence fails.
    pub async fn len(&self, session_id: &str) -> Result<usize> {
        Ok(self.get(session_id).await?.len())
    }

    /// Clear one session.
    ///
    /// # Errors
    /// Returns an error when Valkey-backed persistence fails.
    pub async fn clear(&self, session_id: &str) -> Result<()> {
        if let Some(redis) = &self.redis {
            redis.clear_messages(session_id).await?;
        }
        let mut inner = self.inner.write().await;
        inner.remove(session_id);
        Ok(())
    }

    /// Publish one stream event when Valkey-backed runtime is enabled.
    ///
    /// # Errors
    /// Returns an error when Valkey-backed stream publishing fails.
    pub async fn publish_stream_event(
        &self,
        stream_name: &str,
        fields: Vec<(String, String)>,
    ) -> Result<String> {
        let Some(redis) = &self.redis else {
            return Ok(String::new());
        };
        redis.publish_stream_event(stream_name, &fields).await
    }
}
