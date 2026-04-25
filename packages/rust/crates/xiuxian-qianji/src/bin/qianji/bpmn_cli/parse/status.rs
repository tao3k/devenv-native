use crate::bpmn_cli::deps::QianjiBpmnWorkflowCheckpointBackend;
use crate::bpmn_cli::deps::{PathBuf, invalid_input, io, parse_flag_value};
use crate::bpmn_cli::types::{BpmnCancelCliCommand, BpmnInstancesCliCommand, BpmnStatusCliCommand};

pub(super) fn parse_bpmn_status_command(args: &[String]) -> io::Result<BpmnStatusCliCommand> {
    let mut instance_id = None;
    let mut bpmn_path = None;
    let mut dmn_paths = Vec::new();
    let mut checkpoint_runtime = false;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--instance-id" => {
                instance_id = Some(parse_flag_value(args, &mut index, "--instance-id")?);
            }
            "--bpmn" => {
                bpmn_path = Some(PathBuf::from(parse_flag_value(args, &mut index, "--bpmn")?));
            }
            "--dmn" => {
                dmn_paths.push(PathBuf::from(parse_flag_value(args, &mut index, "--dmn")?));
            }
            "--checkpoint-runtime" => {
                checkpoint_runtime = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "unsupported `bpmn status` option `{other}`"
                )));
            }
        }

        index += 1;
    }

    let checkpoint_backend = optional_bpmn_checkpoint_backend(checkpoint_runtime).ok_or_else(|| {
        invalid_input("missing checkpoint backend for `bpmn status`; use `--checkpoint-runtime` or enable local DuckDB")
    })?;

    Ok(BpmnStatusCliCommand {
        instance_id: instance_id.ok_or_else(|| {
            invalid_input("missing `--instance-id <id>` for `bpmn status` command")
        })?,
        checkpoint_backend,
        bpmn_path,
        dmn_paths,
    })
}

pub(super) fn parse_bpmn_instances_command(args: &[String]) -> io::Result<BpmnInstancesCliCommand> {
    let mut checkpoint_runtime = false;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--checkpoint-runtime" => {
                checkpoint_runtime = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "unsupported `bpmn instances` option `{other}`"
                )));
            }
        }

        index += 1;
    }

    let checkpoint_backend = optional_bpmn_checkpoint_backend(checkpoint_runtime).ok_or_else(|| {
        invalid_input(
            "missing checkpoint backend for `bpmn instances`; use `--checkpoint-runtime` or enable local DuckDB",
        )
    })?;

    Ok(BpmnInstancesCliCommand { checkpoint_backend })
}

pub(super) fn parse_bpmn_cancel_command(args: &[String]) -> io::Result<BpmnCancelCliCommand> {
    let mut instance_id = None;
    let mut checkpoint_runtime = false;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--instance-id" => {
                instance_id = Some(parse_flag_value(args, &mut index, "--instance-id")?);
            }
            "--checkpoint-runtime" => {
                checkpoint_runtime = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "unsupported `bpmn cancel` option `{other}`"
                )));
            }
        }

        index += 1;
    }

    let checkpoint_backend = optional_bpmn_checkpoint_backend(checkpoint_runtime).ok_or_else(|| {
        invalid_input("missing checkpoint backend for `bpmn cancel`; use `--checkpoint-runtime` or enable local DuckDB")
    })?;

    Ok(BpmnCancelCliCommand {
        instance_id: instance_id.ok_or_else(|| {
            invalid_input("missing `--instance-id <id>` for `bpmn cancel` command")
        })?,
        checkpoint_backend,
    })
}

fn optional_bpmn_checkpoint_backend(
    checkpoint_runtime: bool,
) -> Option<QianjiBpmnWorkflowCheckpointBackend> {
    checkpoint_runtime
        .then_some(QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey)
        .or_else(local_bpmn_checkpoint_backend)
}

#[cfg(feature = "duckdb")]
fn local_bpmn_checkpoint_backend() -> Option<QianjiBpmnWorkflowCheckpointBackend> {
    [QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb]
        .into_iter()
        .next()
}

#[cfg(not(feature = "duckdb"))]
fn local_bpmn_checkpoint_backend() -> Option<QianjiBpmnWorkflowCheckpointBackend> {
    None
}
