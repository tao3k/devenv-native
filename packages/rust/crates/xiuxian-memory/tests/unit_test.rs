//! Cargo entry point for `xiuxian-memory` unit tests.

rust_lang_project_harness::rust_project_harness_gate!();

#[path = "unit/lib_policy.rs"]
mod lib_policy;
#[path = "unit/memrl.rs"]
mod memrl;
