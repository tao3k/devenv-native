//! Cargo entry point for xiuxian-types unit tests.

rust_lang_project_harness::rust_project_harness_gate!();

#[path = "unit/scenarios.rs"]
mod scenarios;
#[path = "unit/skill_definition.rs"]
mod skill_definition;
