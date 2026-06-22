use crate::qianji_cli::bpmn_cli::deps::QianjiBpmnWorkflowCheckpointBackend;
use crate::qianji_cli::bpmn_cli::deps::{PathBuf, invalid_input, io, parse_flag_value};
use crate::qianji_cli::bpmn_cli::types::{
    BpmnCliCommand, BpmnEventPollCliCommand, BpmnResumeCliCommand, BpmnTaskClaimCliCommand,
    BpmnTaskCompleteCliCommand, BpmnTaskCompleteCliKind, BpmnTaskReleaseCliCommand,
    BpmnTaskWorklistCliCommand,
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
        Some("claim") => Ok(BpmnCliCommand::TaskClaim(parse_bpmn_task_claim_command(
            &args[1..],
        )?)),
        Some("release") => Ok(BpmnCliCommand::TaskRelease(
            parse_bpmn_task_release_command(&args[1..])?,
        )),
        Some("worklist") => Ok(BpmnCliCommand::TaskWorklist(
            parse_bpmn_task_worklist_command(&args[1..])?,
        )),
        Some(other) => Err(invalid_input(format!(
            "unsupported `bpmn tasks` subcommand `{other}`"
        ))),
        None => Err(invalid_input(
            "missing `bpmn tasks` subcommand; expected `complete`, `claim`, `release`, or `worklist`",
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

    let checkpoint_backend =
        resolve_bpmn_task_checkpoint_backend("bpmn tasks complete", state.checkpoint_runtime)?;

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
        claimant: state.claimant,
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
    claimant: Option<String>,
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
            state.token_id = Some(parse_bpmn_task_token_id(args, index)?);
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
        "--claimant" => {
            state.claimant = Some(parse_flag_value(args, index, "--claimant")?);
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

fn parse_bpmn_task_claim_command(args: &[String]) -> io::Result<BpmnTaskClaimCliCommand> {
    let state = parse_bpmn_task_claim_like_command(args, "bpmn tasks claim")?;
    Ok(BpmnTaskClaimCliCommand {
        instance_id: state.instance_id,
        checkpoint_backend: state.checkpoint_backend,
        token_id: state.token_id,
        process_id: state.process_id,
        activity_id: state.activity_id,
        claimant: state.claimant,
    })
}

fn parse_bpmn_task_release_command(args: &[String]) -> io::Result<BpmnTaskReleaseCliCommand> {
    let state = parse_bpmn_task_claim_like_command(args, "bpmn tasks release")?;
    Ok(BpmnTaskReleaseCliCommand {
        instance_id: state.instance_id,
        checkpoint_backend: state.checkpoint_backend,
        token_id: state.token_id,
        process_id: state.process_id,
        activity_id: state.activity_id,
        claimant: state.claimant,
    })
}

struct BpmnTaskClaimLikeCommand {
    instance_id: String,
    checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
    token_id: u64,
    process_id: String,
    activity_id: String,
    claimant: String,
}

fn parse_bpmn_task_claim_like_command(
    args: &[String],
    command_name: &str,
) -> io::Result<BpmnTaskClaimLikeCommand> {
    let mut state = BpmnTaskClaimLikeParseState::default();
    let mut index = 0;
    while index < args.len() {
        parse_bpmn_task_claim_like_option(args, &mut index, command_name, &mut state)?;
        index += 1;
    }

    let checkpoint_backend =
        resolve_bpmn_task_checkpoint_backend(command_name, state.checkpoint_runtime)?;

    Ok(BpmnTaskClaimLikeCommand {
        instance_id: state.instance_id.ok_or_else(|| {
            invalid_input(format!(
                "missing `--instance-id <id>` for `{command_name}` command"
            ))
        })?,
        checkpoint_backend,
        token_id: state.token_id.ok_or_else(|| {
            invalid_input(format!(
                "missing `--token-id <id>` for `{command_name}` command"
            ))
        })?,
        process_id: state.process_id.ok_or_else(|| {
            invalid_input(format!(
                "missing `--process-id <id>` for `{command_name}` command"
            ))
        })?,
        activity_id: state.activity_id.ok_or_else(|| {
            invalid_input(format!(
                "missing `--activity-id <id>` for `{command_name}` command"
            ))
        })?,
        claimant: state.claimant.ok_or_else(|| {
            invalid_input(format!(
                "missing `--claimant <id>` for `{command_name}` command"
            ))
        })?,
    })
}

#[derive(Default)]
struct BpmnTaskClaimLikeParseState {
    instance_id: Option<String>,
    token_id: Option<u64>,
    process_id: Option<String>,
    activity_id: Option<String>,
    claimant: Option<String>,
    checkpoint_runtime: bool,
}

fn parse_bpmn_task_claim_like_option(
    args: &[String],
    index: &mut usize,
    command_name: &str,
    state: &mut BpmnTaskClaimLikeParseState,
) -> io::Result<()> {
    match args[*index].as_str() {
        "--instance-id" => {
            state.instance_id = Some(parse_flag_value(args, index, "--instance-id")?);
        }
        "--token-id" => {
            state.token_id = Some(parse_bpmn_task_token_id(args, index)?);
        }
        "--process-id" => {
            state.process_id = Some(parse_flag_value(args, index, "--process-id")?);
        }
        "--activity-id" => {
            state.activity_id = Some(parse_flag_value(args, index, "--activity-id")?);
        }
        "--claimant" => {
            state.claimant = Some(parse_flag_value(args, index, "--claimant")?);
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

fn parse_bpmn_task_worklist_command(args: &[String]) -> io::Result<BpmnTaskWorklistCliCommand> {
    let mut state = BpmnTaskWorklistParseState::default();
    let mut index = 0;
    while index < args.len() {
        parse_bpmn_task_worklist_option(args, &mut index, &mut state)?;
        index += 1;
    }

    Ok(BpmnTaskWorklistCliCommand {
        checkpoint_backend: resolve_bpmn_task_checkpoint_backend(
            "bpmn tasks worklist",
            state.checkpoint_runtime,
        )?,
        claimant: state.claimant,
        assignment_resource: state.assignment_resource,
        lane: state.lane,
    })
}

#[derive(Default)]
struct BpmnTaskWorklistParseState {
    claimant: Option<String>,
    assignment_resource: Option<String>,
    lane: Option<String>,
    checkpoint_runtime: bool,
}

fn parse_bpmn_task_worklist_option(
    args: &[String],
    index: &mut usize,
    state: &mut BpmnTaskWorklistParseState,
) -> io::Result<()> {
    match args[*index].as_str() {
        "--claimant" => {
            state.claimant = Some(parse_flag_value(args, index, "--claimant")?);
        }
        "--assignment-resource" => {
            state.assignment_resource =
                Some(parse_flag_value(args, index, "--assignment-resource")?);
        }
        "--lane" => {
            state.lane = Some(parse_flag_value(args, index, "--lane")?);
        }
        "--checkpoint-runtime" => {
            state.checkpoint_runtime = true;
        }
        other => {
            return Err(invalid_input(format!(
                "unsupported `bpmn tasks worklist` option `{other}`"
            )));
        }
    }

    Ok(())
}

fn parse_bpmn_task_token_id(args: &[String], index: &mut usize) -> io::Result<u64> {
    let raw_token_id = parse_flag_value(args, index, "--token-id")?;
    raw_token_id.parse::<u64>().map_err(|error| {
        invalid_input(format!(
            "failed to parse `--token-id` as unsigned integer: {error}"
        ))
    })
}

fn resolve_bpmn_task_checkpoint_backend(
    command_name: &str,
    checkpoint_runtime: bool,
) -> io::Result<QianjiBpmnWorkflowCheckpointBackend> {
    if checkpoint_runtime {
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
    })
}

fn parse_task_complete_kind(raw: &str) -> io::Result<BpmnTaskCompleteCliKind> {
    match raw {
        "task" => Ok(BpmnTaskCompleteCliKind::Task),
        "send" => Ok(BpmnTaskCompleteCliKind::Send),
        "service" => Ok(BpmnTaskCompleteCliKind::Service),
        "script" => Ok(BpmnTaskCompleteCliKind::Script),
        "user" => Ok(BpmnTaskCompleteCliKind::User),
        "manual" => Ok(BpmnTaskCompleteCliKind::Manual),
        other => Err(invalid_input(format!(
            "unsupported `bpmn tasks complete --kind` value `{other}`; expected `task`, `send`, `service`, `script`, `user`, or `manual`"
        ))),
    }
}

#[derive(Default)]
struct BpmnResumeLikeRuntimeFlags {
    trace_stream: bool,
    external_host: bool,
    continue_until_human_boundary: bool,
}

#[derive(Default)]
struct BpmnResumeLikeParseState {
    bpmn_path: Option<PathBuf>,
    dmn_paths: Vec<PathBuf>,
    instance_id: Option<String>,
    host_fixture_path: Option<PathBuf>,
    event_fixture_path: Option<PathBuf>,
    runtime_flags: BpmnResumeLikeRuntimeFlags,
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
        trace_stream: state.runtime_flags.trace_stream,
        external_host: state.runtime_flags.external_host,
        continue_until_human_boundary: state.runtime_flags.continue_until_human_boundary,
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
            state.runtime_flags.trace_stream = true;
        }
        "--external-host" => {
            state.runtime_flags.external_host = true;
        }
        "--continue-until-human-boundary" => {
            state.runtime_flags.continue_until_human_boundary = true;
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
