use crate::qianji_cli::bpmn_cli::types::{BpmnCliCommand, BpmnCliOutput};

use super::{cancel, execution, instances, interrupt, session, status, tasks};

#[cfg(test)]
pub(crate) use super::control_service::resolve_bpmn_checkpoint_store_with_env;
#[cfg(test)]
pub(crate) use super::execution::{
    run_bpmn_run_command_with_runtime_env, run_bpmn_start_at_command_with_runtime_env,
    run_bpmn_task_complete_command_with_runtime_env,
};
#[cfg(test)]
pub(crate) use super::status::run_bpmn_status_command_with_runtime_env;
#[cfg(test)]
pub(crate) use super::tasks::{
    run_bpmn_task_claim_command_with_runtime_env, run_bpmn_task_release_command_with_runtime_env,
    run_bpmn_task_worklist_command_with_runtime_env,
};

pub(crate) async fn handle_bpmn_command(
    command: BpmnCliCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = Box::pin(run_bpmn_command(command)).await?;
    if !output.rendered.is_empty() {
        println!("{}", output.rendered);
    }
    if output.exit_code == 0 {
        Ok(())
    } else {
        std::process::exit(output.exit_code);
    }
}

pub(crate) async fn run_bpmn_command(
    command: BpmnCliCommand,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    match command {
        BpmnCliCommand::Start(command) => execution::run_bpmn_start_command(&command).await,
        BpmnCliCommand::StartAt(command) => execution::run_bpmn_start_at_command(&command).await,
        BpmnCliCommand::Run(command) => execution::run_bpmn_run_command(&command).await,
        BpmnCliCommand::HostSession(command) => {
            session::run_bpmn_host_session_command(&command).await
        }
        BpmnCliCommand::Resume(command) => execution::run_bpmn_resume_command(&command).await,
        BpmnCliCommand::EventPoll(command) => {
            execution::run_bpmn_event_poll_command(&command).await
        }
        BpmnCliCommand::TaskComplete(command) => {
            execution::run_bpmn_task_complete_command(&command).await
        }
        BpmnCliCommand::TaskClaim(command) => tasks::run_bpmn_task_claim_command(&command).await,
        BpmnCliCommand::TaskRelease(command) => {
            tasks::run_bpmn_task_release_command(&command).await
        }
        BpmnCliCommand::TaskWorklist(command) => {
            tasks::run_bpmn_task_worklist_command(&command).await
        }
        BpmnCliCommand::Status(command) => status::run_bpmn_status_command(&command).await,
        BpmnCliCommand::Instances(command) => instances::run_bpmn_instances_command(&command).await,
        BpmnCliCommand::Cancel(command) => cancel::run_bpmn_cancel_command(&command).await,
        BpmnCliCommand::Interrupt(command) => interrupt::run_bpmn_interrupt_command(&command).await,
    }
}
