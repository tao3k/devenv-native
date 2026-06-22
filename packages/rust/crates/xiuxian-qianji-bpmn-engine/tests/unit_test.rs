//! Cargo entry point for `xiuxian-qianji-bpmn-engine` unit tests.

#[path = "unit/lib_policy.rs"]
mod lib_policy;
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
