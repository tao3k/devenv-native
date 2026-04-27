use crate::bpmn_cli::deps::io;
use crate::bpmn_cli::types::BpmnHostSessionCliCommand;

use super::start;

pub(super) fn parse_bpmn_host_session_command(
    args: &[String],
) -> io::Result<BpmnHostSessionCliCommand> {
    let mut start = if args
        .iter()
        .any(|arg| arg == "--node" || arg == "--start-at-node")
    {
        start::parse_bpmn_start_at_command(args)?
    } else {
        start::parse_bpmn_run_command(args)?
    };
    start.external_host = true;
    start.continue_until_human_boundary = true;
    Ok(BpmnHostSessionCliCommand { start })
}
