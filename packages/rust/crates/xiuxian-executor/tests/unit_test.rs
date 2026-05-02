//! Cargo entry point for `xiuxian-executor` unit tests.

rust_lang_project_harness::rust_project_harness_gate!();

#[path = "unit/ast_analyzer.rs"]
mod ast_analyzer;
#[path = "unit/command_analysis.rs"]
mod command_analysis;
#[path = "unit/lib_policy.rs"]
mod lib_policy;
#[path = "unit/nu_bridge.rs"]
mod nu_bridge;
#[path = "unit/query.rs"]
mod query;
