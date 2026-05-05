use std::{fs, io, path::PathBuf};

use crate::{WorkflowPlan, emit_workflow_plan_bpmn, render_workflow_plan_validation_report};

use super::invalid_input;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EmitCliCommand {
    Bpmn { path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EmitCliOutput {
    pub(crate) rendered: String,
}

pub(super) fn handle_emit_command(command: &EmitCliCommand) -> io::Result<()> {
    let output = run_emit_command(command)?;
    println!("{}", output.rendered);
    Ok(())
}

pub(crate) fn run_emit_command(command: &EmitCliCommand) -> io::Result<EmitCliOutput> {
    let rendered = match command {
        EmitCliCommand::Bpmn { path } => {
            let plan = read_workflow_plan(path)?;
            emit_workflow_plan_bpmn(&plan).map_err(|error| {
                invalid_input(render_workflow_plan_validation_report(&error.validation))
            })?
        }
    };
    Ok(EmitCliOutput { rendered })
}

pub(crate) fn parse_emit_command(args: &[String]) -> io::Result<Option<EmitCliCommand>> {
    let Some(command_name) = args.get(1).map(String::as_str) else {
        return Ok(None);
    };
    if command_name != "emit" {
        return Ok(None);
    }

    let Some(path) = args.get(2) else {
        return Err(invalid_input("missing input path for `emit`"));
    };
    parse_emit_flags(args, 3)?;
    Ok(Some(EmitCliCommand::Bpmn {
        path: PathBuf::from(path),
    }))
}

fn parse_emit_flags(args: &[String], start: usize) -> io::Result<()> {
    let mut bpmn = false;
    for value in &args[start..] {
        match value.as_str() {
            "--bpmn" => bpmn = true,
            other => {
                return Err(invalid_input(format!(
                    "`emit <path> --bpmn` does not accept argument `{other}`"
                )));
            }
        }
    }
    if bpmn {
        Ok(())
    } else {
        Err(invalid_input("missing `--bpmn` for `emit <path>`"))
    }
}

fn read_workflow_plan(path: &PathBuf) -> io::Result<WorkflowPlan> {
    let content = fs::read_to_string(path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("failed to read WorkflowPlan at {}: {error}", path.display()),
        )
    })?;
    serde_json::from_str(&content).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "failed to parse WorkflowPlan JSON at {}: {error}",
                path.display()
            ),
        )
    })
}
