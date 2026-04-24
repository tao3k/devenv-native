use super::deps::{
    PathBuf, QianjiBpmnWorkflowCheckpointBackend, invalid_input, io, parse_flag_value,
};
use super::types::{
    BpmnCancelCliCommand, BpmnCliCommand, BpmnEventPollCliCommand, BpmnResumeCliCommand,
    BpmnRunCliCommand, BpmnStartCliCommand, BpmnStatusCliCommand, BpmnTaskCompleteCliCommand,
};

pub(crate) fn parse_bpmn_command(args: &[String]) -> io::Result<Option<BpmnCliCommand>> {
    if args.get(1).map(String::as_str) != Some("bpmn") {
        return Ok(None);
    }

    match args.get(2).map(String::as_str) {
        Some("start") => Ok(Some(BpmnCliCommand::Start(parse_bpmn_start_command(
            &args[3..],
        )?))),
        Some("run") => Ok(Some(BpmnCliCommand::Run(parse_bpmn_run_command(
            &args[3..],
        )?))),
        Some("resume") => Ok(Some(BpmnCliCommand::Resume(parse_bpmn_resume_command(
            &args[3..],
        )?))),
        Some("events") => Ok(Some(parse_bpmn_events_command(&args[3..])?)),
        Some("tasks") => Ok(Some(parse_bpmn_tasks_command(&args[3..])?)),
        Some("status") => Ok(Some(BpmnCliCommand::Status(parse_bpmn_status_command(
            &args[3..],
        )?))),
        Some("cancel") => Ok(Some(BpmnCliCommand::Cancel(parse_bpmn_cancel_command(
            &args[3..],
        )?))),
        Some(other) => Err(invalid_input(format!(
            "unsupported `bpmn` subcommand `{other}`"
        ))),
        None => Err(invalid_input(
            "missing `bpmn` subcommand; expected `start`, `run`, `resume`, `events`, `tasks`, `status`, or `cancel`",
        )),
    }
}

fn parse_bpmn_events_command(args: &[String]) -> io::Result<BpmnCliCommand> {
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

fn parse_bpmn_tasks_command(args: &[String]) -> io::Result<BpmnCliCommand> {
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

fn parse_bpmn_run_command(args: &[String]) -> io::Result<BpmnRunCliCommand> {
    parse_bpmn_start_like_command(args, "bpmn run")
}

fn parse_bpmn_start_command(args: &[String]) -> io::Result<BpmnStartCliCommand> {
    parse_bpmn_start_like_command(args, "bpmn start")
}

#[derive(Default)]
struct BpmnStartLikeParseState {
    bpmn_path: Option<PathBuf>,
    dmn_paths: Vec<PathBuf>,
    process_id: Option<String>,
    instance_id: Option<String>,
    context_json: Option<String>,
    host_fixture_path: Option<PathBuf>,
    event_fixture_path: Option<PathBuf>,
    trace_stream: bool,
    external_host: bool,
    checkpoint_runtime: bool,
    #[cfg(feature = "sqlite")]
    checkpoint_sqlite: Option<PathBuf>,
}

fn parse_bpmn_start_like_command(
    args: &[String],
    command_name: &str,
) -> io::Result<BpmnRunCliCommand> {
    let mut state = BpmnStartLikeParseState::default();

    let mut index = 0;
    while index < args.len() {
        parse_bpmn_start_like_option(args, &mut index, command_name, &mut state)?;
        index += 1;
    }

    let checkpoint_backend = {
        #[cfg(feature = "sqlite")]
        {
            parse_bpmn_start_like_checkpoint_backend(command_name, &state)?
        }
        #[cfg(not(feature = "sqlite"))]
        {
            parse_bpmn_start_like_checkpoint_backend(&state)
        }
    };
    if state.context_json.is_none() && checkpoint_backend.is_none() {
        return Err(invalid_input(format!(
            "missing `--context-json <json>` for fresh `{command_name}` command"
        )));
    }

    Ok(BpmnRunCliCommand {
        bpmn_path: state.bpmn_path.ok_or_else(|| {
            invalid_input(format!(
                "missing `--bpmn <path>` for `{command_name}` command"
            ))
        })?,
        dmn_paths: state.dmn_paths,
        process_id: state.process_id.ok_or_else(|| {
            invalid_input(format!(
                "missing `--process <id>` for `{command_name}` command"
            ))
        })?,
        instance_id: state.instance_id.ok_or_else(|| {
            invalid_input(format!(
                "missing `--instance-id <id>` for `{command_name}` command"
            ))
        })?,
        context_json: state.context_json,
        checkpoint_backend,
        host_fixture_path: state.host_fixture_path,
        event_fixture_path: state.event_fixture_path,
        trace_stream: state.trace_stream,
        external_host: state.external_host,
    })
}

fn parse_bpmn_start_like_option(
    args: &[String],
    index: &mut usize,
    command_name: &str,
    state: &mut BpmnStartLikeParseState,
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
        "--process" => {
            state.process_id = Some(parse_flag_value(args, index, "--process")?);
        }
        "--instance-id" => {
            state.instance_id = Some(parse_flag_value(args, index, "--instance-id")?);
        }
        "--context-json" => {
            state.context_json = Some(parse_flag_value(args, index, "--context-json")?);
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
        "--checkpoint-sqlite" => {
            #[cfg(feature = "sqlite")]
            {
                state.checkpoint_sqlite = Some(PathBuf::from(parse_flag_value(
                    args,
                    index,
                    "--checkpoint-sqlite",
                )?));
            }
            #[cfg(not(feature = "sqlite"))]
            {
                return Err(invalid_input(
                    "`--checkpoint-sqlite` requires the `sqlite` feature",
                ));
            }
        }
        other => {
            return Err(invalid_input(format!(
                "unsupported `{command_name}` option `{other}`"
            )));
        }
    }

    Ok(())
}

#[cfg(feature = "sqlite")]
fn parse_bpmn_start_like_checkpoint_backend(
    command_name: &str,
    state: &BpmnStartLikeParseState,
) -> io::Result<Option<QianjiBpmnWorkflowCheckpointBackend>> {
    parse_bpmn_cli_checkpoint_backend(
        command_name,
        state.checkpoint_runtime,
        state.checkpoint_sqlite.clone(),
    )
}

#[cfg(not(feature = "sqlite"))]
fn parse_bpmn_start_like_checkpoint_backend(
    state: &BpmnStartLikeParseState,
) -> Option<QianjiBpmnWorkflowCheckpointBackend> {
    parse_bpmn_cli_checkpoint_backend(state.checkpoint_runtime)
}

fn parse_bpmn_resume_command(args: &[String]) -> io::Result<BpmnResumeCliCommand> {
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
    #[cfg(feature = "sqlite")]
    checkpoint_sqlite: Option<PathBuf>,
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

    let checkpoint_backend = {
        #[cfg(feature = "sqlite")]
        {
            parse_bpmn_cli_checkpoint_backend(
                command_name,
                state.checkpoint_runtime,
                state.checkpoint_sqlite.clone(),
            )?
        }
        #[cfg(not(feature = "sqlite"))]
        {
            parse_bpmn_cli_checkpoint_backend(state.checkpoint_runtime)
        }
    };

    let checkpoint_backend = checkpoint_backend.ok_or_else(|| {
        invalid_input(format!(
            "missing checkpoint backend for `{command_name}`; use `--checkpoint-runtime` or `--checkpoint-sqlite <path>`"
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
        "--checkpoint-sqlite" => {
            #[cfg(feature = "sqlite")]
            {
                state.checkpoint_sqlite = Some(PathBuf::from(parse_flag_value(
                    args,
                    index,
                    "--checkpoint-sqlite",
                )?));
            }
            #[cfg(not(feature = "sqlite"))]
            {
                return Err(invalid_input(
                    "`--checkpoint-sqlite` requires the `sqlite` feature",
                ));
            }
        }
        other => {
            return Err(invalid_input(format!(
                "unsupported `{command_name}` option `{other}`"
            )));
        }
    }

    Ok(())
}

fn parse_bpmn_status_command(args: &[String]) -> io::Result<BpmnStatusCliCommand> {
    let mut instance_id = None;
    let mut checkpoint_runtime = false;
    #[cfg(feature = "sqlite")]
    let mut checkpoint_sqlite = None;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--instance-id" => {
                instance_id = Some(parse_flag_value(args, &mut index, "--instance-id")?);
            }
            "--checkpoint-runtime" => {
                checkpoint_runtime = true;
            }
            "--checkpoint-sqlite" => {
                #[cfg(feature = "sqlite")]
                {
                    checkpoint_sqlite = Some(PathBuf::from(parse_flag_value(
                        args,
                        &mut index,
                        "--checkpoint-sqlite",
                    )?));
                }
                #[cfg(not(feature = "sqlite"))]
                {
                    return Err(invalid_input(
                        "`--checkpoint-sqlite` requires the `sqlite` feature",
                    ));
                }
            }
            other => {
                return Err(invalid_input(format!(
                    "unsupported `bpmn status` option `{other}`"
                )));
            }
        }

        index += 1;
    }

    let checkpoint_backend = {
        #[cfg(feature = "sqlite")]
        {
            parse_bpmn_cli_checkpoint_backend("bpmn status", checkpoint_runtime, checkpoint_sqlite)?
        }
        #[cfg(not(feature = "sqlite"))]
        {
            parse_bpmn_cli_checkpoint_backend(checkpoint_runtime)
        }
    };

    let checkpoint_backend = checkpoint_backend.ok_or_else(|| {
        invalid_input("missing checkpoint backend for `bpmn status`; use `--checkpoint-runtime` or `--checkpoint-sqlite <path>`")
    })?;

    Ok(BpmnStatusCliCommand {
        instance_id: instance_id.ok_or_else(|| {
            invalid_input("missing `--instance-id <id>` for `bpmn status` command")
        })?,
        checkpoint_backend,
    })
}

fn parse_bpmn_cancel_command(args: &[String]) -> io::Result<BpmnCancelCliCommand> {
    let mut instance_id = None;
    let mut checkpoint_runtime = false;
    #[cfg(feature = "sqlite")]
    let mut checkpoint_sqlite = None;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--instance-id" => {
                instance_id = Some(parse_flag_value(args, &mut index, "--instance-id")?);
            }
            "--checkpoint-runtime" => {
                checkpoint_runtime = true;
            }
            "--checkpoint-sqlite" => {
                #[cfg(feature = "sqlite")]
                {
                    checkpoint_sqlite = Some(PathBuf::from(parse_flag_value(
                        args,
                        &mut index,
                        "--checkpoint-sqlite",
                    )?));
                }
                #[cfg(not(feature = "sqlite"))]
                {
                    return Err(invalid_input(
                        "`--checkpoint-sqlite` requires the `sqlite` feature",
                    ));
                }
            }
            other => {
                return Err(invalid_input(format!(
                    "unsupported `bpmn cancel` option `{other}`"
                )));
            }
        }

        index += 1;
    }

    let checkpoint_backend = {
        #[cfg(feature = "sqlite")]
        {
            parse_bpmn_cli_checkpoint_backend("bpmn cancel", checkpoint_runtime, checkpoint_sqlite)?
        }
        #[cfg(not(feature = "sqlite"))]
        {
            parse_bpmn_cli_checkpoint_backend(checkpoint_runtime)
        }
    };

    let checkpoint_backend = checkpoint_backend.ok_or_else(|| {
        invalid_input("missing checkpoint backend for `bpmn cancel`; use `--checkpoint-runtime` or `--checkpoint-sqlite <path>`")
    })?;

    Ok(BpmnCancelCliCommand {
        instance_id: instance_id.ok_or_else(|| {
            invalid_input("missing `--instance-id <id>` for `bpmn cancel` command")
        })?,
        checkpoint_backend,
    })
}

#[cfg(feature = "sqlite")]
fn parse_bpmn_cli_checkpoint_backend(
    command_name: &str,
    checkpoint_runtime: bool,
    checkpoint_sqlite: Option<PathBuf>,
) -> io::Result<Option<QianjiBpmnWorkflowCheckpointBackend>> {
    match (checkpoint_runtime, checkpoint_sqlite) {
        (false, None) => Ok(None),
        (true, None) => Ok(Some(QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey)),
        (false, Some(path)) => Ok(Some(QianjiBpmnWorkflowCheckpointBackend::Sqlite(path))),
        (true, Some(_)) => Err(invalid_input(format!(
            "`{command_name}` accepts at most one checkpoint backend option"
        ))),
    }
}

#[cfg(not(feature = "sqlite"))]
fn parse_bpmn_cli_checkpoint_backend(
    checkpoint_runtime: bool,
) -> Option<QianjiBpmnWorkflowCheckpointBackend> {
    if checkpoint_runtime {
        Some(QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey)
    } else {
        None
    }
}
