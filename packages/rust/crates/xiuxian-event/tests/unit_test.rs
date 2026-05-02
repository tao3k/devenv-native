//! Cargo entry point for `xiuxian-event` unit tests.

rust_lang_project_harness::rust_project_harness_gate!();

#[path = "unit/event_bus.rs"]
mod event_bus;
#[path = "unit/lib_policy.rs"]
mod lib_policy;
