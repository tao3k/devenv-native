use crate::bpmn_cli::deps::QianjiBpmnWorkflowCheckpointBackend;
use crate::bpmn_cli::deps::{PathBuf, invalid_input, io, parse_flag_value};
use crate::bpmn_cli::types::{BpmnRunCliCommand, BpmnStartCliCommand};

pub(super) fn parse_bpmn_run_command(args: &[String]) -> io::Result<BpmnRunCliCommand> {
    parse_bpmn_start_like_command(args, "bpmn run")
}

pub(super) fn parse_bpmn_start_command(args: &[String]) -> io::Result<BpmnStartCliCommand> {
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
        other => {
            return Err(invalid_input(format!(
                "unsupported `{command_name}` option `{other}`"
            )));
        }
    }

    Ok(())
}
