//! Signal Registry: Global Broadcast Hub for Heterogeneous Event Distribution.
//!
//! This module provides a global signal registry that enables:
//!
//! 1. **Broadcast Distribution**: One signal from Sentinel can notify all active Pipelines
//! 2. **Type Bridging**: Converts `ObservationSignal` from wendao to `ExternalSignal` for pipelines
//! 3. **Subscription Management**: Pipelines can subscribe/unsubscribe dynamically
//! 4. **Rate Limiting**: Token bucket prevents signal storms during large refactors
//!
//! # Architecture
//!
//! ```text
//! Sentinel (wendao)
//!      │
//!      ▼ ObservationSignal
//! SignalRegistry.broadcast()
//!      │
//!      ├── Token Bucket (rate limit check)
//!      │
//!      ├── Subscriber 1: Pipeline A
//!      ├── Subscriber 2: Pipeline B
//!      └── Subscriber N: Pipeline N
//! ```
//!
//! # Usage
//!
//! ```ignore
//! // In application startup:
//! let registry = Arc::new(SignalRegistry::new());
//!
//! // Sentinel broadcasts:
//! registry.broadcast_observation(&observation_signal);
//!
//! // Each Pipeline subscribes:
//! let subscriber = registry.subscribe();
//! pipeline.attach_signal_receiver(subscriber);
//! ```

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::{broadcast, mpsc};

// Re-export ExternalSignal for convenience
use crate::ZhenfaSignalType;
pub use crate::transmuter::streaming::ExternalSignal;

/// Maximum number of subscribers supported by the registry.
const DEFAULT_CHANNEL_CAPACITY: usize = 256;
/// Default tokens per second for rate limiting.
const DEFAULT_TOKENS_PER_SECOND: u64 = 100;
/// Default bucket capacity for rate limiting.
const DEFAULT_BUCKET_CAPACITY: u64 = 200;

/// Token bucket rate limiter for preventing signal storms.
///
/// Uses a simple token bucket algorithm with atomic operations
/// for lock-free rate limiting.
#[derive(Debug)]
struct TokenBucket {
    /// Available tokens.
    tokens: AtomicU64,
    /// Maximum tokens the bucket can hold.
    capacity: u64,
    /// Tokens added per second.
    refill_rate: u64,
    /// Last refill timestamp (monotonic clock nanos).
    last_refill: AtomicU64,
}

impl TokenBucket {
    /// Create a new token bucket.
    fn new(capacity: u64, refill_rate: u64) -> Self {
        let now = Self::current_nanos();
        Self {
            tokens: AtomicU64::new(capacity),
            capacity,
            refill_rate,
            last_refill: AtomicU64::new(now),
        }
    }

    /// Get current time in nanoseconds using monotonic clock.
    fn current_nanos() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|duration| u64::try_from(duration.as_nanos()).ok())
            .unwrap_or(0)
    }

    /// Try to consume one token. Returns true if successful.
    fn try_consume(&self) -> bool {
        self.refill();

        loop {
            let tokens = self.tokens.load(Ordering::Relaxed);
            if tokens == 0 {
                return false;
            }
            if self
                .tokens
                .compare_exchange_weak(
                    tokens,
                    tokens.saturating_sub(1),
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                return true;
            }
        }
    }

    /// Refill tokens based on elapsed time.
    fn refill(&self) {
        let now = Self::current_nanos();
        let last = self.last_refill.load(Ordering::Relaxed);

        if last == 0 {
            // Initialize if not set
            self.last_refill
                .compare_exchange(0, now, Ordering::Relaxed, Ordering::Relaxed)
                .ok();
            return;
        }

        let elapsed = now.saturating_sub(last);
        // Refill every 100ms worth of tokens
        let refill_interval = 100_000_000; // 100ms in nanos

        if elapsed >= refill_interval {
            let intervals = elapsed / refill_interval;
            let tokens_to_add = (self.refill_rate * intervals) / 10; // Per 100ms

            if self
                .last_refill
                .compare_exchange(last, now, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                let current = self.tokens.load(Ordering::Relaxed);
                let new_tokens = (current.saturating_add(tokens_to_add)).min(self.capacity);
                self.tokens.store(new_tokens, Ordering::Relaxed);
            }
        }
    }

    /// Get current token count (for diagnostics).
    fn available(&self) -> u64 {
        self.refill();
        self.tokens.load(Ordering::Relaxed)
    }
}

/// Result of a broadcast attempt with rate limiting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BroadcastResult {
    /// Signal was successfully broadcast.
    Delivered {
        /// Number of subscribers that received the signal.
        subscriber_count: usize,
    },
    /// Signal was dropped due to rate limiting.
    RateLimited {
        /// Tokens available before rate limit hit.
        tokens_remaining: u64,
    },
    /// No subscribers were registered.
    NoSubscribers,
}

/// Input used to convert one observation into a pipeline external signal.
#[derive(Debug, Clone, PartialEq)]
pub struct ObservationSignalInput {
    /// Signal source identifier.
    pub source: String,
    /// Typed signal kind.
    pub signal_type: ZhenfaSignalType,
    /// Human-readable summary.
    pub summary: String,
    /// Confidence score from 0.0 to 1.0.
    pub confidence: f32,
    /// Affected document identifiers.
    pub affected_docs: Vec<String>,
    /// Whether an automatic fix path exists.
    pub auto_fix_available: bool,
}

/// Global signal registry for broadcasting external signals to multiple subscribers.
///
/// This registry enables the "fan-out" pattern where a single signal from Sentinel
/// can be delivered to all active Pipeline instances simultaneously.
///
/// # Lifecycle Safety
///
/// The registry should be wrapped in `Arc<SignalRegistry>` and shared between
/// `ZhenfaOrchestrator` and all `Sentinel` instances to ensure proper lifecycle
/// alignment. The registry will remain active as long as at least one `Arc`
/// reference exists.
///
/// # Rate Limiting
///
/// A token bucket rate limiter prevents "signal storms" during large refactors.
/// Default: 100 tokens/sec, 200 burst capacity.
pub struct SignalRegistry {
    /// Broadcast sender for external signals.
    tx: broadcast::Sender<ExternalSignal>,
    /// Registry identifier for debugging.
    id: String,
    /// Token bucket for rate limiting.
    rate_limiter: TokenBucket,
}

impl Clone for SignalRegistry {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            id: self.id.clone(),
            rate_limiter: TokenBucket::new(DEFAULT_BUCKET_CAPACITY, DEFAULT_TOKENS_PER_SECOND),
        }
    }
}

impl SignalRegistry {
    /// Create a new signal registry with default capacity and rate limiting.
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CHANNEL_CAPACITY)
    }

    /// Create a signal registry with custom broadcast capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let (tx, _rx) = broadcast::channel(capacity);
        Self {
            tx,
            id: format!("registry-{}", uuid::Uuid::new_v4()),
            rate_limiter: TokenBucket::new(DEFAULT_BUCKET_CAPACITY, DEFAULT_TOKENS_PER_SECOND),
        }
    }

    /// Create a signal registry with custom rate limiting.
    #[must_use]
    pub fn with_rate_limit(bucket_capacity: u64, tokens_per_second: u64) -> Self {
        let (tx, _rx) = broadcast::channel(DEFAULT_CHANNEL_CAPACITY);
        Self {
            tx,
            id: format!("registry-{}", uuid::Uuid::new_v4()),
            rate_limiter: TokenBucket::new(bucket_capacity, tokens_per_second),
        }
    }

    /// Subscribe to the signal registry.
    ///
    /// Returns an `mpsc::UnboundedReceiver` that can be attached to a `ZhenfaPipeline`.
    /// The receiver will receive all signals broadcast after subscription.
    ///
    /// Note: We convert from `broadcast::Receiver` to `mpsc::UnboundedReceiver`
    /// to maintain compatibility with `ZhenfaPipeline::attach_signal_receiver()`.
    #[must_use]
    pub fn subscribe(&self) -> mpsc::UnboundedReceiver<ExternalSignal> {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut broadcast_rx = self.tx.subscribe();

        // Spawn a task to forward broadcast messages to the mpsc channel
        tokio::spawn(async move {
            while let Ok(signal) = broadcast_rx.recv().await {
                if tx.send(signal).is_err() {
                    // Receiver dropped, exit the task
                    break;
                }
            }
        });

        rx
    }

    /// Broadcast an external signal to all subscribers with rate limiting.
    ///
    /// Returns a `BroadcastResult` indicating whether the signal was delivered,
    /// rate-limited, or had no subscribers.
    ///
    /// # Rate Limiting
    ///
    /// Signals are rate-limited using a token bucket to prevent "signal storms"
    /// during large refactors. If rate-limited, the signal is dropped and
    /// `BroadcastResult::RateLimited` is returned.
    pub fn broadcast(&self, signal: &ExternalSignal) -> BroadcastResult {
        // Check rate limit first
        if !self.rate_limiter.try_consume() {
            let tokens = self.rate_limiter.available();
            tracing::warn!(
                "SignalRegistry: Rate limited signal {:?}, {} tokens remaining",
                signal.signal_type,
                tokens
            );
            return BroadcastResult::RateLimited {
                tokens_remaining: tokens,
            };
        }

        let receiver_count = self.tx.receiver_count();
        if receiver_count == 0 {
            tracing::debug!(
                "SignalRegistry: No subscribers for signal: {:?}",
                signal.signal_type
            );
            return BroadcastResult::NoSubscribers;
        }

        match self.tx.send(signal.clone()) {
            Ok(_) => {
                tracing::debug!(
                    "SignalRegistry: Broadcast {} to {} subscriber(s)",
                    signal.signal_type,
                    receiver_count
                );
                BroadcastResult::Delivered {
                    subscriber_count: receiver_count,
                }
            }
            Err(e) => {
                tracing::warn!("SignalRegistry: Failed to broadcast signal: {}", e);
                BroadcastResult::NoSubscribers
            }
        }
    }

    /// Broadcast without rate limiting (for high-priority signals).
    ///
    /// Use sparingly for critical signals that must bypass rate limiting.
    pub fn broadcast_unlimited(&self, signal: ExternalSignal) -> usize {
        let receiver_count = self.tx.receiver_count();
        if receiver_count == 0 {
            return 0;
        }

        match self.tx.send(signal) {
            Ok(_) => receiver_count,
            Err(_) => 0,
        }
    }

    /// Get the number of active subscribers.
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }

    /// Get available rate limit tokens.
    #[must_use]
    pub fn available_tokens(&self) -> u64 {
        self.rate_limiter.available()
    }

    /// Get the registry identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Convert an observation signal from wendao to an external signal.
    ///
    /// This is the bridge function that maps wendao's `ObservationSignal`
    /// to `ExternalSignal` for pipeline consumption.
    #[must_use]
    pub fn convert_observation_signal(input: ObservationSignalInput) -> ExternalSignal {
        ExternalSignal {
            source: input.source,
            signal_type: input.signal_type,
            summary: input.summary,
            confidence: input.confidence,
            affected_docs: input.affected_docs,
            auto_fix_available: input.auto_fix_available,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or_else(
                    |_| "unknown".to_string(),
                    |duration| format!("{}s", duration.as_secs()),
                ),
        }
    }
}

impl Default for SignalRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SignalRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalRegistry")
            .field("id", &self.id)
            .field("subscriber_count", &self.subscriber_count())
            .field("available_tokens", &self.available_tokens())
            .finish_non_exhaustive()
    }
}

/// Extension trait for accessing `SignalRegistry` from `ZhenfaContext`.
pub trait SignalRegistryExt {
    /// Get the `SignalRegistry` if attached.
    fn signal_registry(&self) -> Option<Arc<SignalRegistry>>;
}
