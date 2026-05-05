//! Broadcast-channel event bus implementation.

use serde_json::Value;
use tokio::sync::broadcast;

use crate::OmniEvent;

/// High-performance async event bus.
///
/// Uses `tokio::sync::broadcast` channel for:
/// - Thread-safe 1-to-many fan-out.
/// - Non-blocking publish.
/// - Automatic cleanup on receiver drop.
#[derive(Clone)]
pub struct EventBus {
    /// Broadcast sender (clonable for multiple publishers).
    tx: broadcast::Sender<OmniEvent>,
    /// Bus capacity for backpressure handling.
    capacity: usize,
}

impl EventBus {
    /// Create a new event bus with specified capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx, capacity }
    }

    /// Get the bus capacity.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Publish an event to all subscribers.
    ///
    /// Returns the number of subscribers who received the event.
    /// Returns 0 if there are no subscribers (not an error).
    #[must_use]
    pub fn publish(&self, event: OmniEvent) -> usize {
        self.tx.send(event).unwrap_or(0)
    }

    /// Publish an event with topic and payload convenience.
    #[must_use]
    pub fn emit(&self, source: &str, topic: &str, payload: Value) -> usize {
        self.publish(OmniEvent::new(source, topic, payload))
    }

    /// Subscribe to the event bus.
    ///
    /// Returns a receiver that will receive all future events.
    /// Dropping the receiver automatically unsubscribes.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<OmniEvent> {
        self.tx.subscribe()
    }

    /// Get current subscriber count.
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}
