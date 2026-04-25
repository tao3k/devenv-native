use crate::bpmn_cli::deps::QianjiBpmnWorkflowCheckpointBackend;
use crate::bpmn_cli::deps::{PathBuf, invalid_input, io, parse_flag_value};
use crate::bpmn_cli::types::{
    BpmnCliCommand, BpmnEventPollCliCommand, BpmnResumeCliCommand, BpmnTaskCompleteCliCommand,
};

pub(super) fn parse_bpmn_events_command(args: &[String]) -> io::Result<BpmnCliCommand> {
    match args.first().map(String::as_str) {
        Some("poll") => Ok(BpmnCliCommand::EventPoll(parse_bpmn_event_poll_command(
            &args[1..],
        )?)),
        Some(other) => Err(invalid_input(format!(
            "unsupported `bpmn events` subcommand `{other}`"
        ))),
        None => Err(invalid_input(
            "missing `bpmn events` subcommand; expected `poll`",
        )),
    }
}

pub(super) fn parse_bpmn_tasks_command(args: &[String]) -> io::Result<BpmnCliCommand> {
    match args.first().map(String::as_str) {
        Some("complete") => Ok(BpmnCliCommand::TaskComplete(
            parse_bpmn_task_complete_command(&args[1..])?,
        )),
        Some(other) => Err(invalid_input(format!(
            "unsupported `bpmn tasks` subcommand `{other}`"
        ))),
        None => Err(invalid_input(
            "missing `bpmn tasks` subcommand; expected `complete`",
        )),
    }
}

pub(super) fn parse_bpmn_resume_command(args: &[String]) -> io::Result<BpmnResumeCliCommand> {
    parse_bpmn_resume_like_command(args, "bpmn resume")
}

fn parse_bpmn_event_poll_command(args: &[String]) -> io::Result<BpmnEventPollCliCommand> {
    parse_bpmn_resume_like_command(args, "bpmn events poll")
}

fn parse_bpmn_task_complete_command(args: &[String]) -> io::Result<BpmnTaskCompleteCliCommand> {
    parse_bpmn_resume_like_command(args, "bpmn tasks complete")
}

#[derive(Default)]
struct BpmnResumeLikeParseState {
    bpmn_path: Option<PathBuf>,
    dmn_paths: Vec<PathBuf>,
    instance_id: Option<String>,
    host_fixture_path: Option<PathBuf>,
    event_fixture_path: Option<PathBuf>,
    trace_stream: bool,
    external_host: bool,
    checkpoint_runtime: bool,
}

fn parse_bpmn_resume_like_command(
    args: &[String],
    command_name: &str,
) -> io::Result<BpmnResumeCliCommand> {
    let mut state = BpmnResumeLikeParseState::default();
    let mut index = 0;
    while index < args.len() {
        parse_bpmn_resume_like_option(args, &mut index, command_name, &mut state)?;
        index += 1;
    }

    let checkpoint_backend = if state.checkpoint_runtime {
        Some(QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey)
    } else {
        #[cfg(feature = "duckdb")]
        {
            Some(QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb)
        }
        #[cfg(not(feature = "duckdb"))]
        {
            None
        }
    }
    .ok_or_else(|| {
        invalid_input(format!(
            "missing checkpoint backend for `{command_name}`; use `--checkpoint-runtime` or enable local DuckDB"
        ))
    })?;

    Ok(BpmnResumeCliCommand {
        bpmn_path: state.bpmn_path.ok_or_else(|| {
            invalid_input(format!(
                "missing `--bpmn <path>` for `{command_name}` command"
            ))
        })?,
        dmn_paths: state.dmn_paths,
        instance_id: state.instance_id.ok_or_else(|| {
            invalid_input(format!(
                "missing `--instance-id <id>` for `{command_name}` command"
            ))
        })?,
        checkpoint_backend,
        host_fixture_path: state.host_fixture_path,
        event_fixture_path: state.event_fixture_path,
        trace_stream: state.trace_stream,
        external_host: state.external_host,
    })
}

fn parse_bpmn_resume_like_option(
    args: &[String],
    index: &mut usize,
    command_name: &str,
    state: &mut BpmnResumeLikeParseState,
) -> io::Result<()> {
    match args[*index].as_str() {
        "--bpmn" => {
            state.bpmn_path = Some(PathBuf::from(parse_flag_value(args, index, "--bpmn")?));
        }
        "--dmn" => {
            state
                .dmn_paths
                .push(PathBuf::from(parse_flag_value(args, index, "--dmn")?));
        }
        "--instance-id" => {
            state.instance_id = Some(parse_flag_value(args, index, "--instance-id")?);
        }
        "--host-fixture" => {
            state.host_fixture_path = Some(PathBuf::from(parse_flag_value(
                args,
                index,
                "--host-fixture",
            )?));
        }
        "--event-fixture" => {
            state.event_fixture_path = Some(PathBuf::from(parse_flag_value(
                args,
                index,
                "--event-fixture",
            )?));
        }
        "--trace-stream" => {
            state.trace_stream = true;
        }
        "--external-host" => {
            state.external_host = true;
        }
        "--checkpoint-runtime" => {
            state.checkpoint_runtime = true;
        }
        other => {
            return Err(invalid_input(format!(
                "unsupported `{command_name}` option `{other}`"
            )));
        }
    }

    Ok(())
}
