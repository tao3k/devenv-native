//! Cargo entry point for `xiuxian-ast` performance tests.

rust_lang_project_harness::rust_project_harness_gate!();

#[path = "performance/ast_benchmark.rs"]
mod ast_benchmark;
