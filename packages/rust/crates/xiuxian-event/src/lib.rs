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

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = {
        rust_lang_project_harness::default_rust_harness_config().with_verification_profile_hint(
            rust_lang_project_harness::RustVerificationProfileHint::new(
                "src/lib.rs",
                [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
            )
            .with_rationale("crate root owns the public package API for cargo-test verification"),
        )
    }
);

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
