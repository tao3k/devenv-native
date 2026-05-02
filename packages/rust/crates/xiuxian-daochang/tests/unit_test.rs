//! Canonical unit-test harness for `xiuxian-daochang`.
#![recursion_limit = "256"]

rust_lang_project_harness::rust_project_harness_gate!();

#[path = "unit/root_agent.rs"]
mod agent;
#[path = "unit/root_observability.rs"]
mod observability;
#[path = "unit/root_session.rs"]
mod session;
#[path = "unit/mod.rs"]
mod unit;
