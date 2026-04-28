//! BPMN CLI feature folder.
//!
//! Start with `api`; it is the single visible entry seam for this folder.

mod api;
mod deps;
mod host;
mod parse;
mod render;
mod run;
mod types;

#[cfg(all(test, feature = "duckdb"))]
pub(crate) use api::{
    BpmnCancelCliCommand, BpmnEventPollCliCommand, BpmnInterruptCliCommand, BpmnResumeCliCommand,
    BpmnStatusCliCommand, BpmnTaskClaimCliCommand, BpmnTaskCompleteCliCommand,
    BpmnTaskCompleteCliKind, BpmnTaskReleaseCliCommand, BpmnTaskWorklistCliCommand,
};
#[cfg(test)]
pub(crate) use api::{
    BpmnCliCommand, BpmnHostSessionCliCommand, BpmnRunCliCommand, BpmnStartAtCliCommand,
    BpmnStartCliCommand, render_bpmn_pending_host_work_stream_lines,
    resolve_bpmn_checkpoint_store_with_env, run_bpmn_command,
    run_bpmn_run_command_with_runtime_env, run_bpmn_start_at_command_with_runtime_env,
    run_bpmn_status_command_with_runtime_env, run_bpmn_task_claim_command_with_runtime_env,
    run_bpmn_task_release_command_with_runtime_env,
    run_bpmn_task_worklist_command_with_runtime_env,
};
pub(crate) use api::{handle_bpmn_command, parse_bpmn_command};
