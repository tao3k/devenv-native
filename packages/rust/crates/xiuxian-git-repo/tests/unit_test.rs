//! Cargo entry point for xiuxian-git-repo unit tests.

rust_lang_project_harness::rust_project_harness_gate!();

#[path = "unit/diff.rs"]
mod diff;
#[path = "unit/layout.rs"]
mod layout;
#[path = "unit/locks.rs"]
mod locks;
#[path = "unit/materialization.rs"]
mod materialization;
#[path = "unit/retry.rs"]
mod retry;
#[path = "support/mod.rs"]
mod support;
