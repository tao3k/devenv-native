//! Agentic expansion execution interface.

mod api;
#[path = "worker.rs"]
mod worker;

pub(super) use api::agentic_expansion_execute_with_config;
