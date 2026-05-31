//! Cargo entry point for `xiuxian-qianji-runtime` unit suites.

#[path = "unit/bpmn_host_work_activity/mod.rs"]
mod bpmn_host_work_activity;
#[path = "unit/flowhub_service_activity.rs"]
mod flowhub_service_activity;
#[path = "unit/flowhub_worker_loop.rs"]
mod flowhub_worker_loop;
#[path = "unit/workflow_control.rs"]
mod workflow_control;
