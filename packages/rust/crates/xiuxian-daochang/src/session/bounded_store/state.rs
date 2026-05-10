//! Bounded session store state and constructors.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::RwLock;
use xiuxian_window::SessionWindow;

use crate::observability::SessionEvent;
use crate::session::SessionSummarySegment;

use crate::session::redis_backend::RedisSessionBackend;

const DEFAULT_SUMMARY_MAX_SEGMENTS: usize = 8;
const DEFAULT_SUMMARY_MAX_CHARS: usize = 480;

/// Bounded session store: one ring buffer (`SessionWindow`) per `session_id`.
/// Thread-safe via `RwLock`.
#[derive(Clone)]
pub struct BoundedSessionStore {
    pub(crate) inner: Arc<RwLock<HashMap<String, SessionWindow>>>,
    pub(crate) summaries: Arc<RwLock<HashMap<String, VecDeque<SessionSummarySegment>>>>,
    pub(crate) max_slots: usize,
    pub(crate) summary_max_segments: usize,
    pub(crate) summary_max_chars: usize,
    pub(crate) redis: Option<Arc<RedisSessionBackend>>,
}

/// Bounded session window counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundedSessionStats {
    /// Number of complete user/assistant turns.
    pub turn_count: u64,
    /// Total tool calls recorded in the bounded window.
    pub total_tool_calls: u64,
    /// Number of occupied slots in the bounded ring.
    pub ring_len: usize,
}

/// Bounded snapshot message and summary counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundedSessionSnapshotStats {
    /// Number of message/window slots affected by the snapshot operation.
    pub messages: usize,
    /// Number of summary segments affected by the snapshot operation.
    pub summary_segments: usize,
}

impl From<(usize, usize)> for BoundedSessionSnapshotStats {
    fn from((messages, summary_segments): (usize, usize)) -> Self {
        Self {
            messages,
            summary_segments,
        }
    }
}

impl BoundedSessionStore {
    pub(crate) fn from_redis_backend(
        max_turns: usize,
        summary_max_segments: usize,
        summary_max_chars: usize,
        redis: Option<Arc<RedisSessionBackend>>,
    ) -> Self {
        let max_turns = max_turns.max(1);
        let max_slots = max_turns.saturating_mul(2).max(2);
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            summaries: Arc::new(RwLock::new(HashMap::new())),
            max_slots,
            summary_max_segments: summary_max_segments.max(1),
            summary_max_chars: summary_max_chars.max(1),
            redis,
        }
    }

    /// Create a store with the given max turns per session.
    ///
    /// # Errors
    /// Returns an error when Valkey-backed runtime initialization fails.
    pub fn new(max_turns: usize) -> Result<Self> {
        Self::new_with_limits(
            max_turns,
            DEFAULT_SUMMARY_MAX_SEGMENTS,
            DEFAULT_SUMMARY_MAX_CHARS,
        )
    }

    /// Create a store with explicit summary limits.
    ///
    /// # Errors
    /// Returns an error when Valkey-backed runtime initialization fails.
    pub fn new_with_limits(
        max_turns: usize,
        summary_max_segments: usize,
        summary_max_chars: usize,
    ) -> Result<Self> {
        let redis = match RedisSessionBackend::from_env() {
            Some(Ok(backend)) => {
                tracing::info!(
                    event = SessionEvent::SessionBackendEnabled.as_str(),
                    key_prefix = %backend.key_prefix(),
                    ttl_secs = ?backend.ttl_secs(),
                    message_content_max_chars = ?backend.runtime_snapshot().message_content_max_chars,
                    max_turns,
                    "bounded session store backend enabled: valkey"
                );
                Some(Arc::new(backend))
            }
            Some(Err(error)) => {
                return Err(error).context("failed to initialize valkey bounded session store");
            }
            None => None,
        };
        Ok(Self::from_redis_backend(
            max_turns,
            summary_max_segments,
            summary_max_chars,
            redis,
        ))
    }

    /// Create a bounded store with explicit Valkey backend parameters.
    ///
    /// # Errors
    /// Returns an error when Valkey backend creation fails.
    pub fn new_with_redis(
        max_turns: usize,
        redis_url: impl Into<String>,
        key_prefix: Option<String>,
        ttl_secs: Option<u64>,
    ) -> Result<Self> {
        Self::new_with_redis_and_limits(
            max_turns,
            redis_url,
            key_prefix,
            ttl_secs,
            DEFAULT_SUMMARY_MAX_SEGMENTS,
            DEFAULT_SUMMARY_MAX_CHARS,
        )
    }

    /// Create a bounded store with explicit Valkey backend and summary limits.
    ///
    /// # Errors
    /// Returns an error when Valkey backend creation fails.
    pub fn new_with_redis_and_limits(
        max_turns: usize,
        redis_url: impl Into<String>,
        key_prefix: Option<String>,
        ttl_secs: Option<u64>,
        summary_max_segments: usize,
        summary_max_chars: usize,
    ) -> Result<Self> {
        let backend = RedisSessionBackend::new_from_parts(redis_url.into(), key_prefix, ttl_secs)?;
        Ok(Self::from_redis_backend(
            max_turns,
            summary_max_segments,
            summary_max_chars,
            Some(Arc::new(backend)),
        ))
    }
}
