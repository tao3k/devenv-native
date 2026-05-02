//! Process-global event bus convenience surface.

use serde_json::Value;
use std::sync::{Arc, LazyLock};
use tokio::sync::broadcast;

use crate::{EventBus, OmniEvent};

/// Global event bus singleton.
pub static GLOBAL_BUS: LazyLock<Arc<EventBus>> = LazyLock::new(|| Arc::new(EventBus::new(2048)));

/// Convenience function to publish to the global bus.
pub fn publish(source: &str, topic: &str, payload: Value) {
    let event = OmniEvent::new(source, topic, payload);
    let _ = GLOBAL_BUS.publish(event);
}

/// Convenience function to emit to the global bus.
pub fn emit(source: &str, topic: &str, payload: Value) {
    let _ = GLOBAL_BUS.emit(source, topic, payload);
}

/// Get a subscriber for the global bus.
#[must_use]
pub fn subscribe() -> broadcast::Receiver<OmniEvent> {
    GLOBAL_BUS.subscribe()
}
