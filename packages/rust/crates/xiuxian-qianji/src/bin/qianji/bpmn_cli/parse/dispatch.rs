use crate::bpmn_cli::deps::{invalid_input, io};
use crate::bpmn_cli::parse::{resume, start, status};
use crate::bpmn_cli::types::BpmnCliCommand;

pub(crate) fn parse_bpmn_command(args: &[String]) -> io::Result<Option<BpmnCliCommand>> {
    if args.get(1).map(String::as_str) != Some("bpmn") {
        return Ok(None);
    }

    match args.get(2).map(String::as_str) {
        Some("start") => Ok(Some(BpmnCliCommand::Start(
            start::parse_bpmn_start_command(&args[3..])?,
        ))),
        Some("run") => Ok(Some(BpmnCliCommand::Run(start::parse_bpmn_run_command(
            &args[3..],
        )?))),
        Some("resume") => Ok(Some(BpmnCliCommand::Resume(
            resume::parse_bpmn_resume_command(&args[3..])?,
        ))),
        Some("events") => Ok(Some(resume::parse_bpmn_events_command(&args[3..])?)),
        Some("tasks") => Ok(Some(resume::parse_bpmn_tasks_command(&args[3..])?)),
        Some("status") => Ok(Some(BpmnCliCommand::Status(
            status::parse_bpmn_status_command(&args[3..])?,
        ))),
        Some("instances" | "list") => Ok(Some(BpmnCliCommand::Instances(
            status::parse_bpmn_instances_command(&args[3..])?,
        ))),
        Some("cancel") => Ok(Some(BpmnCliCommand::Cancel(
            status::parse_bpmn_cancel_command(&args[3..])?,
        ))),
        Some(other) => Err(invalid_input(format!(
            "unsupported `bpmn` subcommand `{other}`"
        ))),
        None => Err(invalid_input(
            "missing `bpmn` subcommand; expected `start`, `run`, `resume`, `events`, `tasks`, `status`, `instances`, or `cancel`",
        )),
    }
}
