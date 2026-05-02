//! Cargo entry point for xiuxian-tokenizer unit tests.

rust_lang_project_harness::rust_project_harness_gate!();

#[path = "unit/tokenizer.rs"]
mod tokenizer;
#[path = "unit/tokenizer_benchmark.rs"]
mod tokenizer_benchmark;
