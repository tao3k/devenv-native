use crate::bpmn_cli::deps::QianjiBpmnWorkflowCheckpointBackend;
use crate::bpmn_cli::deps::{PathBuf, invalid_input, io, parse_flag_value};
use crate::bpmn_cli::types::{
    BpmnCliCommand, BpmnEventPollCliCommand, BpmnResumeCliCommand, BpmnTaskCompleteCliCommand,
    BpmnTaskCompleteCliKind,
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
    let mut state = BpmnTaskCompleteParseState::default();
    let mut index = 0;
    while index < args.len() {
        parse_bpmn_task_complete_option(args, &mut index, &mut state)?;
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
        invalid_input(
            "missing checkpoint backend for `bpmn tasks complete`; use `--checkpoint-runtime` or enable local DuckDB",
        )
    })?;

    Ok(BpmnTaskCompleteCliCommand {
        bpmn_path: state.bpmn_path.ok_or_else(|| {
            invalid_input("missing `--bpmn <path>` for `bpmn tasks complete` command")
        })?,
        dmn_paths: state.dmn_paths,
        instance_id: state.instance_id.ok_or_else(|| {
            invalid_input("missing `--instance-id <id>` for `bpmn tasks complete` command")
        })?,
        checkpoint_backend,
        token_id: state.token_id.ok_or_else(|| {
            invalid_input("missing `--token-id <id>` for `bpmn tasks complete` command")
        })?,
        process_id: state.process_id.ok_or_else(|| {
            invalid_input("missing `--process-id <id>` for `bpmn tasks complete` command")
        })?,
        activity_id: state.activity_id.ok_or_else(|| {
            invalid_input("missing `--activity-id <id>` for `bpmn tasks complete` command")
        })?,
        kind: state.kind.ok_or_else(|| {
            invalid_input(
                "missing `--kind send|service|script|user|manual` for `bpmn tasks complete` command",
            )
        })?,
        data_json: state.data_json.ok_or_else(|| {
            invalid_input("missing `--data-json <json>` for `bpmn tasks complete` command")
        })?,
        host_fixture_path: state.host_fixture_path,
        event_fixture_path: state.event_fixture_path,
        trace_stream: state.trace_stream,
        continue_until_human_boundary: state.continue_until_human_boundary,
    })
}

#[derive(Default)]
struct BpmnTaskCompleteParseState {
    bpmn_path: Option<PathBuf>,
    dmn_paths: Vec<PathBuf>,
    instance_id: Option<String>,
    token_id: Option<u64>,
    process_id: Option<String>,
    activity_id: Option<String>,
    kind: Option<BpmnTaskCompleteCliKind>,
    data_json: Option<String>,
    host_fixture_path: Option<PathBuf>,
    event_fixture_path: Option<PathBuf>,
    trace_stream: bool,
    continue_until_human_boundary: bool,
    checkpoint_runtime: bool,
}

fn parse_bpmn_task_complete_option(
    args: &[String],
    index: &mut usize,
    state: &mut BpmnTaskCompleteParseState,
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
        "--token-id" => {
            let raw_token_id = parse_flag_value(args, index, "--token-id")?;
            state.token_id = Some(raw_token_id.parse::<u64>().map_err(|error| {
                invalid_input(format!(
                    "failed to parse `--token-id` as unsigned integer: {error}"
                ))
            })?);
        }
        "--process-id" => {
            state.process_id = Some(parse_flag_value(args, index, "--process-id")?);
        }
        "--activity-id" => {
            state.activity_id = Some(parse_flag_value(args, index, "--activity-id")?);
        }
        "--kind" => {
            state.kind = Some(parse_task_complete_kind(&parse_flag_value(
                args, index, "--kind",
            )?)?);
        }
        "--data-json" => {
            state.data_json = Some(parse_flag_value(args, index, "--data-json")?);
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
        "--continue-until-human-boundary" => {
            state.continue_until_human_boundary = true;
        }
        "--checkpoint-runtime" => {
            state.checkpoint_runtime = true;
        }
        other => {
            return Err(invalid_input(format!(
                "unsupported `bpmn tasks complete` option `{other}`"
            )));
        }
    }

    Ok(())
}

fn parse_task_complete_kind(raw: &str) -> io::Result<BpmnTaskCompleteCliKind> {
    match raw {
        "send" => Ok(BpmnTaskCompleteCliKind::Send),
        "service" => Ok(BpmnTaskCompleteCliKind::Service),
        "script" => Ok(BpmnTaskCompleteCliKind::Script),
        "user" => Ok(BpmnTaskCompleteCliKind::User),
        "manual" => Ok(BpmnTaskCompleteCliKind::Manual),
        other => Err(invalid_input(format!(
            "unsupported `bpmn tasks complete --kind` value `{other}`; expected `send`, `service`, `script`, `user`, or `manual`"
        ))),
    }
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
    continue_until_human_boundary: bool,
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
        continue_until_human_boundary: state.continue_until_human_boundary,
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
        "--continue-until-human-boundary" => {
            state.continue_until_human_boundary = true;
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
