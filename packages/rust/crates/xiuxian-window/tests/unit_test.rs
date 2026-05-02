//! Cargo entry point for `xiuxian-window` unit tests.

rust_lang_project_harness::rust_project_harness_gate!();

#[path = "unit/lib_policy.rs"]
mod lib_policy;
#[path = "unit/window.rs"]
mod window;
