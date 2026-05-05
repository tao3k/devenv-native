use crate::qianji_cli::bpmn_cli::deps::QianjiBpmnWorkflowCheckpointBackend;
use crate::qianji_cli::bpmn_cli::deps::{PathBuf, invalid_input, io, parse_flag_value};
use crate::qianji_cli::bpmn_cli::types::{
    BpmnRunCliCommand, BpmnStartAtCliCommand, BpmnStartCliCommand,
};

pub(super) fn parse_bpmn_run_command(args: &[String]) -> io::Result<BpmnRunCliCommand> {
    parse_bpmn_start_like_command(args, "bpmn run")
}

pub(super) fn parse_bpmn_start_command(args: &[String]) -> io::Result<BpmnStartCliCommand> {
    parse_bpmn_start_like_command(args, "bpmn start")
}

pub(super) fn parse_bpmn_start_at_command(args: &[String]) -> io::Result<BpmnStartAtCliCommand> {
    let command = parse_bpmn_start_like_command(args, "bpmn start-at")?;
    if command.start_at_node_id.is_none() {
        return Err(invalid_input(
            "missing `--node <id>` for `bpmn start-at` command",
        ));
    }
    Ok(command)
}

#[derive(Default)]
struct BpmnStartLikeRuntimeFlags {
    trace_stream: bool,
    external_host: bool,
    continue_until_human_boundary: bool,
}

#[derive(Default)]
struct BpmnStartLikeParseState {
    bpmn_path: Option<PathBuf>,
    dmn_paths: Vec<PathBuf>,
    process_id: Option<String>,
    instance_id: Option<String>,
    context_json: Option<String>,
    start_at_node_id: Option<String>,
    host_fixture_path: Option<PathBuf>,
    event_fixture_path: Option<PathBuf>,
    runtime_flags: BpmnStartLikeRuntimeFlags,
    checkpoint_runtime: bool,
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
        start_at_node_id: state.start_at_node_id,
        checkpoint_backend,
        host_fixture_path: state.host_fixture_path,
        event_fixture_path: state.event_fixture_path,
        trace_stream: state.runtime_flags.trace_stream,
        external_host: state.runtime_flags.external_host,
        continue_until_human_boundary: state.runtime_flags.continue_until_human_boundary,
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
        "--node" | "--start-at-node" => {
            if command_name != "bpmn start-at" {
                return Err(invalid_input(format!(
                    "unsupported `{command_name}` option `{}`; use `bpmn start-at`",
                    args[*index]
                )));
            }
            state.start_at_node_id = Some(parse_flag_value(args, index, args[*index].as_str())?);
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
