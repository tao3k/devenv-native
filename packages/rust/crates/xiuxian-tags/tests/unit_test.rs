//! Cargo entry point for xiuxian-tags unit tests.

rust_lang_project_harness::rust_project_harness_gate!();

#[path = "unit/extractor.rs"]
mod extractor;
