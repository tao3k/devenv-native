use std::io::BufRead;

use crate::bpmn_cli::deps::io;
use crate::bpmn_cli::types::{BpmnCliOutput, BpmnHostSessionCliCommand};

use super::request::{BpmnHostSessionRequest, parse_session_request};
use super::result::{BpmnHostSessionStepResult, emit_session_result};
use super::runtime::BpmnHostSessionRuntime;

pub(crate) async fn run_bpmn_host_session_command(
    command: &BpmnHostSessionCliCommand,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    let runtime = BpmnHostSessionRuntime::start(command).await?;
    let start_result = runtime.start_result.clone();
    emit_session_result(&start_result, "")?;
    if start_result.output.exit_code != 0 {
        return Ok(BpmnCliOutput {
            rendered: String::new(),
            exit_code: 0,
        });
    }

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        match parse_session_request(line.trim()) {
            Ok(BpmnHostSessionRequest::TaskComplete(request)) => {
                let result = runtime.complete_task(command, request).await?;
                emit_session_result(&result, "")?;
            }
            Ok(BpmnHostSessionRequest::Stop) => break,
            Err(error) => {
                emit_session_result(
                    &BpmnHostSessionStepResult {
                        output: BpmnCliOutput {
                            rendered: String::new(),
                            exit_code: 2,
                        },
                        summary: None,
                    },
                    &error.to_string(),
                )?;
            }
        }
    }

    Ok(BpmnCliOutput {
        rendered: String::new(),
        exit_code: 0,
    })
}
