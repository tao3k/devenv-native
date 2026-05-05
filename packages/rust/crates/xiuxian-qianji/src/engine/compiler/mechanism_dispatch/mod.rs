//! Mechanism dispatch chain.
//!
//! Start in `resolver_chain`; `stateless`, `stateful_cfg`, and `leaf_dispatch`
//! contribute ordered resolver stages.

#[path = "facade.rs"]
mod facade;
#[path = "../../../engine_compiler_mechanism_dispatch_leaf_dispatch.rs"]
mod leaf_dispatch;
mod resolver_chain;
#[path = "../../../engine_compiler_mechanism_dispatch_stateful_cfg.rs"]
mod stateful_cfg;
mod stateless;

pub(super) use facade::build;
