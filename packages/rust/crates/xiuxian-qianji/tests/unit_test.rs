//! Cargo entry point for dormant `xiuxian-qianji` unit suites.

xiuxian_testing::crate_test_policy_harness!();

pub use xiuxian_qianji::runtime_config;
pub use xiuxian_qianji::*;

#[path = "unit/bpmn_engine_dependency.rs"]
mod bpmn_engine_dependency;
#[path = "unit/bpmn/mod.rs"]
mod bpmn_tests;
#[cfg(feature = "qianji-full")]
#[path = "unit/unit_adversarial_loop.rs"]
mod unit_adversarial_loop;
#[cfg(feature = "qianji-full")]
#[path = "unit/unit_qianji_execution.rs"]
mod unit_qianji_execution;
#[cfg(feature = "qianji-full")]
#[path = "unit/unit_qianji_safety.rs"]
mod unit_qianji_safety;
