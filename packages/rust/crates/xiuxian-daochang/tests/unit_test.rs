//! Canonical unit-test harness for `xiuxian-daochang`.
#![recursion_limit = "256"]

#[path = "unit/root_agent.rs"]
mod agent;
#[path = "unit/lib_policy.rs"]
mod lib_policy;
#[path = "unit/root_observability.rs"]
mod observability;
#[path = "unit/root_session.rs"]
mod session;
#[path = "unit/mod.rs"]
mod unit;
