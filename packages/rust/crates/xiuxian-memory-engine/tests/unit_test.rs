//! Cargo entry point for `xiuxian-memory-engine` unit tests.

xiuxian_testing::crate_test_policy_harness!();

#[path = "unit/common/mod.rs"]
mod common;
#[path = "unit/complex_scenarios/mod.rs"]
mod complex_scenarios;
#[path = "unit/encoder.rs"]
mod encoder;
#[path = "unit/episode.rs"]
mod episode;
#[path = "unit/feedback_tracking.rs"]
mod feedback_tracking;
#[path = "unit/gate.rs"]
mod gate;
#[path = "unit/memory_engine/mod.rs"]
mod memory_engine;
#[path = "unit/projection.rs"]
mod projection;
#[path = "unit/q_table.rs"]
mod q_table;
#[path = "unit/schema.rs"]
mod schema;
#[path = "unit/scope.rs"]
mod scope;
#[path = "unit/state_backend.rs"]
mod state_backend;
#[path = "unit/state_persistence.rs"]
mod state_persistence;
#[path = "unit/store.rs"]
mod store;
#[path = "unit/two_phase.rs"]
mod two_phase;
