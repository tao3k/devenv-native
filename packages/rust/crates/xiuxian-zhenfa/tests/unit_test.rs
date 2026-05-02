//! Cargo entry point for xiuxian-zhenfa unit tests.

rust_lang_project_harness::rust_project_harness_gate!();

#[path = "unit/notification.rs"]
mod notification;
#[path = "unit/signal_registry.rs"]
mod signal_registry;
