//! Cargo entry point for dormant `xiuxian-qianji` unit suites.

xiuxian_testing::crate_test_policy_harness!();

#[path = "unit/bpmn_engine_dependency.rs"]
mod bpmn_engine_dependency;
#[path = "unit/unit_adversarial_loop.rs"]
mod unit_adversarial_loop;
#[path = "unit/unit_qianji_execution.rs"]
mod unit_qianji_execution;
#[path = "unit/unit_qianji_safety.rs"]
mod unit_qianji_safety;
