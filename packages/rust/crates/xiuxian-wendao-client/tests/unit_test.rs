//! Cargo entry point for `xiuxian-wendao-client` unit tests.

rust_lang_project_harness::rust_project_harness_gate!();

#[path = "unit/cli.rs"]
mod cli;
#[path = "unit/get_runtime.rs"]
mod get_runtime;
#[path = "unit/lib_policy.rs"]
mod lib_policy;
#[path = "unit/lint_discovery.rs"]
mod lint_discovery;
#[path = "unit/lint_run/mod.rs"]
mod lint_run;
