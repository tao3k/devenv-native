//! Cargo entry point for xiuxian-vector unit tests.
#![cfg(feature = "vector-store")]

xiuxian_testing::crate_test_policy_harness!();

#[path = "unit/filter_expr.rs"]
mod filter_expr;
#[path = "unit/ops_column_read.rs"]
mod ops_column_read;
#[path = "unit/search_engine/context.rs"]
mod search_engine_context;
#[path = "unit/search_impl.rs"]
mod search_impl;
#[path = "unit/string_match.rs"]
mod string_match;
