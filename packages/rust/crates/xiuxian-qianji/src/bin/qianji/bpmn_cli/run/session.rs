use std::io::{BufRead, Write};

use crate::bpmn_cli::deps::{
    BpmnAdvanceOutcome, QianjiBpmnCheckpointStore, QianjiBpmnPreparedWorkflowStart,
    QianjiBpmnSession, QianjiBpmnWorkflowCheckpointBackend, QianjiBpmnWorkflowControlError,
    QianjiBpmnWorkflowControlService, QianjiBpmnWorkflowResumeRequest,
    QianjiBpmnWorkflowTaskCompleteRequest, SchedulerAgentIdentity, invalid_input, io,
};
use crate::bpmn_cli::types::{
    BpmnCliHostBridgeContext, BpmnCliOutput, BpmnExecutionRenderContext, BpmnHostSessionCliCommand,
    BpmnRunCliCommand, BpmnTaskCompleteCliCommand, BpmnTaskCompleteCliKind,
};
use crate::bpmn_cli::{host, render};

use super::execution::{
    build_bpmn_workflow_start_request, build_bpmn_workflow_task_complete_request,
};
use super::shared::workflow_control_service;

const SESSION_RESULT_PREFIX: &str = "@@QIANJI_SESSION_RESULT ";

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

struct BpmnHostSessionRuntime {
    control_service: QianjiBpmnWorkflowControlService,
    prepared_source: QianjiBpmnPreparedWorkflowStart,
    host_context: BpmnCliHostBridgeContext,
    start_result: BpmnHostSessionStepResult,
}

impl BpmnHostSessionRuntime {
    async fn start(
        command: &BpmnHostSessionCliCommand,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let scheduler_identity = SchedulerAgentIdentity::from_env();
        let control_service = workflow_control_service(None, Some(&scheduler_identity));
        let start_request = build_bpmn_workflow_start_request(&command.start)?;
        let prepared = control_service.prepare_start_workflow(&start_request)?;
        let prepared_source = prepared.clone();
        let host_context = host::build_bpmn_cli_host_bridge(
            &prepared.package,
            command.start.process_id.as_str(),
            command.start.host_fixture_path.as_deref(),
            command.start.event_fixture_path.as_deref(),
        )?;
        let start_result =
            run_prepared_session_start(&command.start, &control_service, prepared, &host_context)
                .await?;

        Ok(Self {
            control_service,
            prepared_source,
            host_context,
            start_result,
        })
    }

    async fn complete_task(
        &self,
        session_command: &BpmnHostSessionCliCommand,
        request: BpmnHostSessionTaskCompleteRequest,
    ) -> Result<BpmnHostSessionStepResult, Box<dyn std::error::Error>> {
        let task_command = build_task_complete_command(session_command, request)?;
        let task_request = build_bpmn_workflow_task_complete_request(&task_command)?;
        let resume_request = QianjiBpmnWorkflowResumeRequest {
            bpmn_path: task_command.bpmn_path.clone(),
            dmn_paths: task_command.dmn_paths.clone(),
            instance_id: task_command.instance_id.clone(),
            checkpoint_backend: task_command.checkpoint_backend.clone(),
        };
        match self
            .control_service
            .prepare_resume_workflow_from_prepared_start(&resume_request, &self.prepared_source)
            .await
        {
            Ok(prepared) => {
                run_prepared_session_task_complete(
                    &task_command,
                    &task_request,
                    &self.control_service,
                    prepared,
                    &self.host_context,
                )
                .await
            }
            Err(QianjiBpmnWorkflowControlError::CheckpointMissing { .. }) => {
                Ok(BpmnHostSessionStepResult {
                    output: render::render_bpmn_task_complete_missing_output(&task_command),
                    summary: None,
                })
            }
            Err(error) => Err(error.into()),
        }
    }
}

#[derive(Clone)]
struct BpmnHostSessionStepResult {
    output: BpmnCliOutput,
    summary: Option<BpmnHostSessionResultSummary>,
}

#[derive(Clone)]
struct BpmnHostSessionResultSummary {
    outcome: String,
    checkpoint: BpmnHostSessionCheckpointSummary,
    pending_host_work: usize,
    variables: serde_json::Value,
}

#[derive(Clone)]
struct BpmnHostSessionCheckpointSummary {
    backend: String,
    source: String,
    saved: String,
    deleted: String,
    status: String,
}

async fn run_prepared_session_start(
    command: &BpmnRunCliCommand,
    control_service: &QianjiBpmnWorkflowControlService,
    prepared: QianjiBpmnPreparedWorkflowStart,
    host_context: &BpmnCliHostBridgeContext,
) -> Result<BpmnHostSessionStepResult, Box<dyn std::error::Error>> {
    let report = if command.host_fixture_path.is_some() {
        control_service
            .start_prepared_workflow_until_human_boundary(
                prepared,
                &host_context.host,
                false,
                |session, events| {
                    if command.trace_stream {
                        for line in
                            render::render_bpmn_execution_trace_stream_lines(session, events)
                        {
                            println!("{line}");
                        }
                    }
                },
            )
            .await?
    } else {
        control_service
            .start_prepared_workflow_until_host_boundary(
                prepared,
                &host_context.host,
                false,
                |session, events| {
                    if command.trace_stream {
                        for line in
                            render::render_bpmn_execution_trace_stream_lines(session, events)
                        {
                            println!("{line}");
                        }
                    }
                },
            )
            .await?
    };
    if command.trace_stream {
        for line in render::render_bpmn_pending_host_work_stream_lines(&report.execution.session) {
            println!("{line}");
        }
    }

    let render_context = BpmnExecutionRenderContext {
        resolved_bpmn_path: report.resolved_bpmn_path.as_path(),
        resolved_dmn_paths: &report.resolved_dmn_paths,
        checkpoint_store: report.checkpoint_store.as_ref(),
        resolved_host_fixture_path: host_context.resolved_host_fixture_path.as_deref(),
        resolved_event_fixture_path: host_context.resolved_event_fixture_path.as_deref(),
        resumed_from_checkpoint: report.execution.resumed_from_checkpoint,
        checkpoint_saved: report.execution.checkpoint_saved,
        checkpoint_deleted: report.execution.checkpoint_deleted,
    };

    let command_title = if command.start_at_node_id.is_some() {
        "qianji start-at"
    } else {
        "qianji run"
    };
    Ok(build_session_step_result(
        command_title,
        &report.execution.session,
        &report.execution.outcome,
        &render_context,
    ))
}

async fn run_prepared_session_task_complete(
    command: &BpmnTaskCompleteCliCommand,
    request: &QianjiBpmnWorkflowTaskCompleteRequest,
    control_service: &QianjiBpmnWorkflowControlService,
    prepared: QianjiBpmnPreparedWorkflowStart,
    host_context: &BpmnCliHostBridgeContext,
) -> Result<BpmnHostSessionStepResult, Box<dyn std::error::Error>> {
    let report = if command.host_fixture_path.is_some() {
        control_service
            .complete_prepared_workflow_task(prepared, request, &host_context.host)
            .await?
    } else {
        control_service
            .complete_prepared_workflow_task_until_host_boundary(
                prepared,
                request,
                &host_context.host,
            )
            .await?
    };
    if command.trace_stream {
        for line in render::render_bpmn_pending_host_work_stream_lines(&report.execution.session) {
            println!("{line}");
        }
    }

    let render_context = BpmnExecutionRenderContext {
        resolved_bpmn_path: report.resolved_bpmn_path.as_path(),
        resolved_dmn_paths: &report.resolved_dmn_paths,
        checkpoint_store: report.checkpoint_store.as_ref(),
        resolved_host_fixture_path: host_context.resolved_host_fixture_path.as_deref(),
        resolved_event_fixture_path: host_context.resolved_event_fixture_path.as_deref(),
        resumed_from_checkpoint: report.execution.resumed_from_checkpoint,
        checkpoint_saved: report.execution.checkpoint_saved,
        checkpoint_deleted: report.execution.checkpoint_deleted,
    };

    Ok(build_session_step_result(
        "qianji task complete",
        &report.execution.session,
        &report.execution.outcome,
        &render_context,
    ))
}

fn build_session_step_result(
    command_label: &str,
    session: &QianjiBpmnSession,
    outcome: &BpmnAdvanceOutcome,
    render_context: &BpmnExecutionRenderContext<'_>,
) -> BpmnHostSessionStepResult {
    let summary = BpmnHostSessionResultSummary {
        outcome: bpmn_session_outcome_label(outcome).to_string(),
        checkpoint: session_checkpoint_summary(render_context),
        pending_host_work: session.instance().pending_host_work.len(),
        variables: session.instance().variables.clone(),
    };
    let rendered = render_compact_session_status(command_label, &summary);
    BpmnHostSessionStepResult {
        output: BpmnCliOutput {
            rendered,
            exit_code: if matches!(outcome, BpmnAdvanceOutcome::Failed(_)) {
                2
            } else {
                0
            },
        },
        summary: Some(summary),
    }
}

fn render_compact_session_status(
    command_label: &str,
    summary: &BpmnHostSessionResultSummary,
) -> String {
    format!(
        "{command_label}: {} (checkpoint={}, source={}, saved={}, deleted={}, pending_host={})",
        summary.outcome,
        summary.checkpoint.backend,
        summary.checkpoint.source,
        summary.checkpoint.saved,
        summary.checkpoint.deleted,
        summary.pending_host_work,
    )
}

fn session_checkpoint_summary(
    render_context: &BpmnExecutionRenderContext<'_>,
) -> BpmnHostSessionCheckpointSummary {
    let backend = render_context
        .checkpoint_store
        .map_or("none", bpmn_session_checkpoint_backend_label);
    let source = if render_context.resumed_from_checkpoint {
        "resumed"
    } else {
        "fresh"
    };
    let saved = if render_context.checkpoint_saved {
        "yes"
    } else {
        "no"
    };
    let deleted = if render_context.checkpoint_deleted {
        "yes"
    } else {
        "no"
    };
    let status = if render_context.checkpoint_deleted {
        "deleted"
    } else if render_context.checkpoint_saved {
        "saved"
    } else {
        "unchanged"
    };
    BpmnHostSessionCheckpointSummary {
        backend: backend.to_string(),
        source: source.to_string(),
        saved: saved.to_string(),
        deleted: deleted.to_string(),
        status: status.to_string(),
    }
}

fn bpmn_session_checkpoint_backend_label(store: &QianjiBpmnCheckpointStore) -> &'static str {
    match store {
        QianjiBpmnCheckpointStore::Valkey { .. } => "runtime_valkey",
        #[cfg(feature = "duckdb")]
        QianjiBpmnCheckpointStore::DuckDb { .. } => "duckdb",
    }
}

fn bpmn_session_outcome_label(outcome: &BpmnAdvanceOutcome) -> &'static str {
    match outcome {
        BpmnAdvanceOutcome::Advanced => "advanced",
        BpmnAdvanceOutcome::BlockedOnHost(_) => "blocked_on_host",
        BpmnAdvanceOutcome::WaitingExternalEvent => "waiting_external_event",
        BpmnAdvanceOutcome::Suspended(_) => "suspended",
        BpmnAdvanceOutcome::Completed => "completed",
        BpmnAdvanceOutcome::Failed(_) => "failed",
    }
}

fn parse_session_request(raw: &str) -> Result<BpmnHostSessionRequest, Box<dyn std::error::Error>> {
    serde_json::from_str(raw)
        .map_err(|error| invalid_input(format!("invalid BPMN host-session JSONL request: {error}")))
        .map_err(Into::into)
}

fn build_task_complete_command(
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
        "send" => Ok(BpmnTaskCompleteCliKind::Send),
        "service" => Ok(BpmnTaskCompleteCliKind::Service),
        "script" => Ok(BpmnTaskCompleteCliKind::Script),
        "user" => Ok(BpmnTaskCompleteCliKind::User),
        "manual" => Ok(BpmnTaskCompleteCliKind::Manual),
        other => Err(invalid_input(format!(
            "unsupported BPMN host-session task kind `{other}`; expected `send`, `service`, `script`, `user`, or `manual`"
        ))),
    }
}

fn emit_session_result(result: &BpmnHostSessionStepResult, stderr: &str) -> io::Result<()> {
    let payload = serde_json::json!({
        "exitCode": result.output.exit_code,
        "stdout": result.output.rendered,
        "stderr": stderr,
        "outcome": result.summary.as_ref().map(|summary| summary.outcome.as_str()),
        "checkpoint": result.summary.as_ref().map(|summary| serde_json::json!({
            "backend": summary.checkpoint.backend,
            "source": summary.checkpoint.source,
            "saved": summary.checkpoint.saved,
            "deleted": summary.checkpoint.deleted,
            "status": summary.checkpoint.status,
        })),
        "pendingHostWork": result
            .summary
            .as_ref()
            .map(|summary| summary.pending_host_work),
        "variables": result
            .summary
            .as_ref()
            .map(|summary| summary.variables.clone()),
    });
    println!(
        "{SESSION_RESULT_PREFIX}{}",
        serde_json::to_string(&payload)?
    );
    io::stdout().flush()
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum BpmnHostSessionRequest {
    TaskComplete(BpmnHostSessionTaskCompleteRequest),
    Stop,
}

#[derive(Debug, serde::Deserialize)]
struct BpmnHostSessionTaskCompleteRequest {
    token_id: u64,
    process_id: String,
    activity_id: String,
    kind: String,
    data: serde_json::Value,
    claimant: Option<String>,
    continue_until_human_boundary: Option<bool>,
}
