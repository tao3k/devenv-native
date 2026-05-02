//! Cargo entry point for `xiuxian-lance` unit tests.

rust_lang_project_harness::rust_project_harness_gate!();

#[path = "unit/lance.rs"]
mod lance;
#[path = "unit/lib_policy.rs"]
mod lib_policy;
