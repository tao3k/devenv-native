use super::deps::{PathBuf, invalid_input, io, parse_flag_value};
use super::types::{BpmnCliCheckpointBackend, BpmnCliCommand, BpmnRunCliCommand};

pub(crate) fn parse_bpmn_command(args: &[String]) -> io::Result<Option<BpmnCliCommand>> {
    if args.get(1).map(String::as_str) != Some("bpmn") {
        return Ok(None);
    }

    match args.get(2).map(String::as_str) {
        Some("run") => Ok(Some(BpmnCliCommand::Run(parse_bpmn_run_command(
            &args[3..],
        )?))),
        Some(other) => Err(invalid_input(format!(
            "unsupported `bpmn` subcommand `{other}`"
        ))),
        None => Err(invalid_input("missing `bpmn` subcommand; expected `run`")),
    }
}

fn parse_bpmn_run_command(args: &[String]) -> io::Result<BpmnRunCliCommand> {
    let mut bpmn_path = None;
    let mut dmn_paths = Vec::new();
    let mut process_id = None;
    let mut instance_id = None;
    let mut context_json = None;
    let mut host_fixture_path = None;
    let mut event_fixture_path = None;
    let mut checkpoint_runtime = false;
    #[cfg(feature = "sqlite")]
    let mut checkpoint_sqlite = None;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--bpmn" => {
                bpmn_path = Some(PathBuf::from(parse_flag_value(args, &mut index, "--bpmn")?));
            }
            "--dmn" => {
                dmn_paths.push(PathBuf::from(parse_flag_value(args, &mut index, "--dmn")?));
            }
            "--process" => {
                process_id = Some(parse_flag_value(args, &mut index, "--process")?);
            }
            "--instance-id" => {
                instance_id = Some(parse_flag_value(args, &mut index, "--instance-id")?);
            }
            "--context-json" => {
                context_json = Some(parse_flag_value(args, &mut index, "--context-json")?);
            }
            "--host-fixture" => {
                host_fixture_path = Some(PathBuf::from(parse_flag_value(
                    args,
                    &mut index,
                    "--host-fixture",
                )?));
            }
            "--event-fixture" => {
                event_fixture_path = Some(PathBuf::from(parse_flag_value(
                    args,
                    &mut index,
                    "--event-fixture",
                )?));
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
                    "unsupported `bpmn run` option `{other}`"
                )));
            }
        }

        index += 1;
    }

    let checkpoint_backend = {
        #[cfg(feature = "sqlite")]
        {
            parse_bpmn_cli_checkpoint_backend(checkpoint_runtime, checkpoint_sqlite)?
        }
        #[cfg(not(feature = "sqlite"))]
        {
            parse_bpmn_cli_checkpoint_backend(checkpoint_runtime)
        }
    };

    if context_json.is_none() && checkpoint_backend.is_none() {
        return Err(invalid_input(
            "missing `--context-json <json>` for fresh `bpmn run` command",
        ));
    }

    Ok(BpmnRunCliCommand {
        bpmn_path: bpmn_path
            .ok_or_else(|| invalid_input("missing `--bpmn <path>` for `bpmn run` command"))?,
        dmn_paths,
        process_id: process_id
            .ok_or_else(|| invalid_input("missing `--process <id>` for `bpmn run` command"))?,
        instance_id: instance_id
            .ok_or_else(|| invalid_input("missing `--instance-id <id>` for `bpmn run` command"))?,
        context_json,
        checkpoint_backend,
        host_fixture_path,
        event_fixture_path,
    })
}

#[cfg(feature = "sqlite")]
fn parse_bpmn_cli_checkpoint_backend(
    checkpoint_runtime: bool,
    checkpoint_sqlite: Option<PathBuf>,
) -> io::Result<Option<BpmnCliCheckpointBackend>> {
    match (checkpoint_runtime, checkpoint_sqlite) {
        (false, None) => Ok(None),
        (true, None) => Ok(Some(BpmnCliCheckpointBackend::RuntimeValkey)),
        (false, Some(path)) => Ok(Some(BpmnCliCheckpointBackend::Sqlite(path))),
        (true, Some(_)) => Err(invalid_input(
            "`bpmn run` accepts at most one checkpoint backend option",
        )),
    }
}

#[cfg(not(feature = "sqlite"))]
fn parse_bpmn_cli_checkpoint_backend(checkpoint_runtime: bool) -> Option<BpmnCliCheckpointBackend> {
    if checkpoint_runtime {
        Some(BpmnCliCheckpointBackend::RuntimeValkey)
    } else {
        None
    }
}
