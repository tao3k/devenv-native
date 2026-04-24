//! Cargo entry point for xiuxian-vector performance tests.
#![cfg(feature = "vector-store")]

xiuxian_testing::crate_test_policy_harness!();

#[path = "performance/search_perf_guard.rs"]
mod search_perf_guard;
#[path = "performance/vector_benchmark.rs"]
mod vector_benchmark;
