//! xiuxian-memory: `MemRL` self-evolving memory system.

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!();

pub mod core;

pub use core::learner::MemRLCortex;
pub use core::types::{MemoryAction, MemoryState};
