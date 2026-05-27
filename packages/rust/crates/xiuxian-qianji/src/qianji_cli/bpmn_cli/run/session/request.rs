#[cfg(feature = "duckdb")]
use crate::qianji_cli::bpmn_cli::deps::QianjiBpmnWorkflowCheckpointBackend;
use crate::qianji_cli::bpmn_cli::deps::{invalid_input, io};
use crate::qianji_cli::bpmn_cli::types::{
    BpmnHostSessionCliCommand, BpmnTaskCompleteCliCommand, BpmnTaskCompleteCliKind,
};

pub(super) fn parse_session_request(
    raw: &str,
) -> Result<BpmnHostSessionRequest, Box<dyn std::error::Error>> {
    serde_json::from_str(raw)
        .map_err(|error| invalid_input(format!("invalid BPMN host-session JSONL request: {error}")))
        .map_err(Into::into)
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(super) enum BpmnHostSessionRequest {
    TaskComplete(BpmnHostSessionTaskCompleteRequest),
    Stop,
}

#[derive(Debug, serde::Deserialize)]
pub(super) struct BpmnHostSessionTaskCompleteRequest {
    token_id: u64,
    process_id: String,
    activity_id: String,
    kind: String,
    data: serde_json::Value,
    claimant: Option<String>,
    continue_until_human_boundary: Option<bool>,
}

pub(super) fn build_task_complete_command(
    session: &BpmnHostSessionCliCommand,
    request: BpmnHostSessionTaskCompleteRequest,
) -> Result<BpmnTaskCompleteCliCommand, Box<dyn std::error::Error>> {
    let checkpoint_backend = match session.start.checkpoint_backend.clone() {
        Some(backend) => backend,
        None => {
            #[cfg(feature = "duckdb")]
            {
                QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb
            }
            #[cfg(not(feature = "duckdb"))]
            {
                return Err(invalid_input(
                    "missing checkpoint backend for `bpmn host-session`; use `--checkpoint-runtime` or enable local DuckDB",
                )
                .into());
            }
        }
    };

    Ok(BpmnTaskCompleteCliCommand {
        bpmn_path: session.start.bpmn_path.clone(),
        dmn_paths: session.start.dmn_paths.clone(),
        instance_id: session.start.instance_id.clone(),
        checkpoint_backend,
        token_id: request.token_id,
        process_id: request.process_id,
        activity_id: request.activity_id,
        kind: parse_session_task_complete_kind(request.kind.as_str())?,
        data_json: serde_json::to_string(&request.data)?,
        claimant: request.claimant,
        host_fixture_path: session.start.host_fixture_path.clone(),
        event_fixture_path: session.start.event_fixture_path.clone(),
        trace_stream: session.start.trace_stream,
        continue_until_human_boundary: session.start.host_fixture_path.is_some()
            && request.continue_until_human_boundary.unwrap_or(true),
    })
}

fn parse_session_task_complete_kind(raw: &str) -> io::Result<BpmnTaskCompleteCliKind> {
    match raw {
        "task" => Ok(BpmnTaskCompleteCliKind::Task),
        "send" => Ok(BpmnTaskCompleteCliKind::Send),
        "service" => Ok(BpmnTaskCompleteCliKind::Service),
        "script" => Ok(BpmnTaskCompleteCliKind::Script),
        "user" => Ok(BpmnTaskCompleteCliKind::User),
        "manual" => Ok(BpmnTaskCompleteCliKind::Manual),
        other => Err(invalid_input(format!(
            "unsupported BPMN host-session task kind `{other}`; expected `task`, `send`, `service`, `script`, `user`, or `manual`"
        ))),
    }
}
