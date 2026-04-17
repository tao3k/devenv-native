//! Cargo entry point for `xiuxian-wendao-client` unit tests.

xiuxian_testing::crate_test_policy_harness!();

#[path = "unit/cli.rs"]
mod cli;
#[path = "unit/lint_discovery.rs"]
mod lint_discovery;
#[path = "unit/lint_run/mod.rs"]
mod lint_run;
