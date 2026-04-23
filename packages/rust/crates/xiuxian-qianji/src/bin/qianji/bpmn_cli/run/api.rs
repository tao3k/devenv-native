use crate::bpmn_cli::types::{BpmnCliCommand, BpmnCliOutput};

use super::{cancel, execution, status};

#[cfg(test)]
pub(crate) use super::execution::run_bpmn_run_command_with_runtime_env;
#[cfg(test)]
pub(crate) use super::shared::resolve_bpmn_checkpoint_store_with_env;

pub(crate) async fn handle_bpmn_command(
    command: BpmnCliCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = run_bpmn_command(command).await?;
    println!("{}", output.rendered);
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
        BpmnCliCommand::Run(command) => execution::run_bpmn_run_command(&command).await,
        BpmnCliCommand::Resume(command) => execution::run_bpmn_resume_command(&command).await,
        BpmnCliCommand::EventPoll(command) => {
            execution::run_bpmn_event_poll_command(&command).await
        }
        BpmnCliCommand::TaskComplete(command) => {
            execution::run_bpmn_task_complete_command(&command).await
        }
        BpmnCliCommand::Status(command) => status::run_bpmn_status_command(&command).await,
        BpmnCliCommand::Cancel(command) => cancel::run_bpmn_cancel_command(&command).await,
    }
}
