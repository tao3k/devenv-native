//! `qianji` command implementation.

mod bpmn_cli;
mod common;
mod construct_cli;
mod contract_feedback_cli;
mod dir_cli;
mod dispatch;
mod emit_cli;
mod graph_export;
mod json_output;
mod lint_cli;
mod manifest_exec;
mod template_cli;
#[cfg(test)]
#[path = "../tests/unit/bin/qianji/test_exports.rs"]
pub(crate) mod test_exports;
mod usage;
mod workspace;

pub(crate) use common::{invalid_input, parse_flag_value, resolve_cli_path};
#[cfg(test)]
pub(crate) use test_exports::{
    BpmnCliCheckpointBackend, BpmnCliCommand, BpmnHostSessionCliCommand, BpmnRunCliCommand,
    BpmnStartAtCliCommand, BpmnStartCliCommand, BpmnStatusCliCommand, BpmnTaskClaimCliCommand,
    BpmnTaskCompleteCliCommand, BpmnTaskCompleteCliKind, BpmnTaskReleaseCliCommand,
    BpmnTaskWorklistCliCommand, ConstructCliCommand, ContractFeedbackCliCommand,
    DEFAULT_CONTRACT_FEEDBACK_TABLE_NAME, DirCliCommand, EmitCliCommand, LintCliCommand,
    MaterializeCliTarget, REST_DOCS_PACK_ID, RestDocsCliCommand, ShowCliTarget, TemplateCliCommand,
    build_contract_feedback_config, build_rest_docs_collection_context, parse_bpmn_command,
    parse_construct_command, parse_contract_feedback_command, parse_dir_command,
    parse_emit_command, parse_lint_command, parse_template_command,
    resolve_bpmn_checkpoint_store_with_env, resolve_workspace_root, run_bpmn_command,
    run_bpmn_run_command_with_runtime_env, run_bpmn_start_at_command_with_runtime_env,
    run_bpmn_status_command_with_runtime_env, run_bpmn_task_claim_command_with_runtime_env,
    run_bpmn_task_complete_command_with_runtime_env,
    run_bpmn_task_release_command_with_runtime_env,
    run_bpmn_task_worklist_command_with_runtime_env, run_construct_command,
    run_deterministic_rest_docs_contract_feedback, run_dir_command, run_emit_command,
    run_lint_command, run_scaffold_rest_docs_contract_feedback, run_template_command,
    sanitize_prj_cache_home,
};

/// Runs the `qianji` command-line interface.
///
/// # Errors
/// Returns an error if argument parsing, environment resolution, compilation, or execution fails.
pub async fn run_qianji_cli() -> Result<(), Box<dyn std::error::Error>> {
    Box::pin(dispatch::run()).await
}

#[cfg(test)]
#[path = "../tests/unit/bin/qianji/mod.rs"]
mod tests;
