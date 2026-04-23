//! Cargo entry point for `qianji-bpmn-engine` unit tests.

xiuxian_testing::crate_test_policy_harness!();

#[path = "unit/test_support.rs"]
mod test_support;

#[path = "unit/public_api/mod.rs"]
mod public_api;

#[path = "unit/checkpoint/mod.rs"]
mod checkpoint;

#[path = "unit/bpmn/mod.rs"]
mod bpmn;

#[path = "unit/dmn/mod.rs"]
mod dmn;

#[path = "unit/lint/mod.rs"]
mod lint;

#[path = "unit/host_dispatch/mod.rs"]
mod host_dispatch;

#[path = "unit/host_resume/mod.rs"]
mod host_resume;

#[path = "unit/external_wait.rs"]
mod external_wait;

#[path = "unit/runtime/mod.rs"]
mod runtime;

#[path = "unit/performance/mod.rs"]
mod performance;
