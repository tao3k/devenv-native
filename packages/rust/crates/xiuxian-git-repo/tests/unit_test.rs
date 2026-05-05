//! Cargo entry point for xiuxian-git-repo unit tests.

#[path = "unit/diff.rs"]
mod diff;
#[path = "unit/layout.rs"]
mod layout;
#[path = "unit/lib_policy.rs"]
mod lib_policy;
#[path = "unit/locks.rs"]
mod locks;
#[path = "unit/materialization.rs"]
mod materialization;
#[path = "unit/retry.rs"]
mod retry;
#[path = "support/mod.rs"]
mod support;
