use std::io::Write;

use crate::bpmn_cli::deps::io;
use crate::bpmn_cli::deps::{BpmnAdvanceOutcome, QianjiBpmnCheckpointStore, QianjiBpmnSession};
use crate::bpmn_cli::types::{BpmnCliOutput, BpmnExecutionRenderContext};

const SESSION_RESULT_PREFIX: &str = "@@QIANJI_SESSION_RESULT ";

#[derive(Clone)]
pub(super) struct BpmnHostSessionStepResult {
    pub(super) output: BpmnCliOutput,
    pub(super) summary: Option<BpmnHostSessionResultSummary>,
}

#[derive(Clone)]
pub(super) struct BpmnHostSessionResultSummary {
    outcome: String,
    checkpoint: BpmnHostSessionCheckpointSummary,
    pending_host_work: usize,
    variables: serde_json::Value,
}

#[derive(Clone)]
pub(super) struct BpmnHostSessionCheckpointSummary {
    backend: String,
    source: String,
    saved: String,
    deleted: String,
    status: String,
}

pub(super) fn emit_session_result(
    result: &BpmnHostSessionStepResult,
    stderr: &str,
) -> io::Result<()> {
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

pub(super) fn build_session_step_result(
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
