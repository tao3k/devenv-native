//! Cargo entry point for `xiuxian-logging` unit tests.

rust_lang_project_harness::rust_project_harness_gate!();

#[path = "unit/lib_policy.rs"]
mod lib_policy;
#[path = "unit/logging_args.rs"]
mod logging_args;
