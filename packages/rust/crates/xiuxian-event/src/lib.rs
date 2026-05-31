//! High-Performance Event Bus for Agentic OS
//!
//! Provides a pub/sub event system backed by tokio's broadcast channel.
//! Used to decouple components: Watcher -> Cortex -> Kernel -> Agent.
//!
//! # Architecture
//!
//! ```text
//! Event (source, topic, payload)
//!      ↓
//! EventBus.publish() → broadcast::Sender
//!      ↓
//! Fan-out to multiple Subscribers
//!      ↓
//! Each component receives events asynchronously
//! ```

mod bus;
mod event;
mod global;

/// Event source constants.
pub mod sources;
/// Event topic constants for type-safe routing
pub mod topics;

pub use bus::EventBus;
pub use event::OmniEvent;
pub use global::{GLOBAL_BUS, emit, publish, subscribe};
