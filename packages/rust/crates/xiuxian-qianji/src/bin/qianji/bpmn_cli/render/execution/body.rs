use std::fmt::Write as _;

use crate::bpmn_cli::deps::{BpmnAdvanceOutcome, Path, QianjiBpmnSession};
use crate::bpmn_cli::types::{BpmnCliOutput, BpmnExecutionRenderContext};

use crate::bpmn_cli::render::support::{
    append_bpmn_human_task_lifecycle_event_summary, append_bpmn_wait_registrations,
    bpmn_checkpoint_backend_label, bpmn_lifecycle_label, bpmn_outcome_label,
    bpmn_suspend_reason_label,
};

use super::host_work::append_bpmn_pending_host_work;
use super::trace::render_bpmn_execution_trace;

pub(super) fn render_bpmn_execution_output(
    title: &str,
    process_id: &str,
    instance_id: &str,
    session: &QianjiBpmnSession,
    outcome: &BpmnAdvanceOutcome,
    render_context: &BpmnExecutionRenderContext<'_>,
) -> BpmnCliOutput {
    let checkpoint_backend = render_context
        .checkpoint_store
        .map_or("none", bpmn_checkpoint_backend_label);
    let checkpoint_source = if render_context.resumed_from_checkpoint {
        "resumed"
    } else {
        "fresh"
    };
    let checkpoint_saved_label = if render_context.checkpoint_saved {
        "yes"
    } else {
        "no"
    };
    let checkpoint_deleted_label = if render_context.checkpoint_deleted {
        "yes"
    } else {
        "no"
    };
    let host_fixture = render_context.resolved_host_fixture_path.map_or_else(
        || "none".to_string(),
        |path: &Path| path.display().to_string(),
    );
    let event_fixture = render_context.resolved_event_fixture_path.map_or_else(
        || "none".to_string(),
        |path: &Path| path.display().to_string(),
    );
    let variables = serde_json::to_string_pretty(&session.instance().variables)
        .unwrap_or_else(|error| format!("{{\"serialization_error\":\"{error}\"}}"));
    let mut rendered = format!(
        "# {title}\n\nSource: {}\nProcess: {}\nInstance: {}\nPackage: {}\nOutcome: {}\nLifecycle: {}\nCheckpoint backend: {}\nCheckpoint source: {}\nCheckpoint saved: {}\nCheckpoint deleted: {}\nHost fixture: {}\nEvent fixture: {}\nDMN sources: {}\nSequence: {}\nActive tokens: {}\nPending host work: {}\nWait registrations: {}\n",
        render_context.resolved_bpmn_path.display(),
        process_id,
        instance_id,
        session.package().package_id,
        bpmn_outcome_label(outcome),
        bpmn_lifecycle_label(&session.instance().lifecycle),
        checkpoint_backend,
        checkpoint_source,
        checkpoint_saved_label,
        checkpoint_deleted_label,
        host_fixture,
        event_fixture,
        render_context.resolved_dmn_paths.len(),
        session.instance().sequence,
        session.instance().active_tokens.len(),
        session.instance().pending_host_work.len(),
        session.instance().waits.len(),
    );

    append_dmn_sources(&mut rendered, render_context);
    append_failure_state(&mut rendered, session, outcome);
    append_bpmn_human_task_lifecycle_event_summary(
        &mut rendered,
        &session.instance().human_task_events,
    );
    append_bpmn_wait_registrations(&mut rendered, session.package(), session.instance());
    append_bpmn_pending_host_work(&mut rendered, session);
    append_trace(&mut rendered, session);
    append_variables(&mut rendered, variables.as_str());

    BpmnCliOutput {
        rendered,
        exit_code: if matches!(outcome, BpmnAdvanceOutcome::Failed(_)) {
            2
        } else {
            0
        },
    }
}

fn append_dmn_sources(rendered: &mut String, render_context: &BpmnExecutionRenderContext<'_>) {
    if render_context.resolved_dmn_paths.is_empty() {
        return;
    }

    let _ = writeln!(rendered, "\n## DMN Sources");
    for path in render_context.resolved_dmn_paths {
        let _ = writeln!(rendered, "- {}", path.display());
    }
}

fn append_failure_state(
    rendered: &mut String,
    session: &QianjiBpmnSession,
    outcome: &BpmnAdvanceOutcome,
) {
    if let Some(reason) = session.instance().suspend_reason.as_ref() {
        let _ = writeln!(
            rendered,
            "\nSuspend reason: {}",
            bpmn_suspend_reason_label(reason)
        );
    }

    if let BpmnAdvanceOutcome::Failed(message) = outcome {
        let _ = writeln!(rendered, "\nFailure: {message}");
    }
}

fn append_trace(rendered: &mut String, session: &QianjiBpmnSession) {
    let trace = render_bpmn_execution_trace(session);
    let _ = writeln!(rendered, "\n## Trace");
    let _ = writeln!(rendered, "```json");
    let _ = writeln!(rendered, "{trace}");
    let _ = writeln!(rendered, "```");
}

fn append_variables(rendered: &mut String, variables: &str) {
    let _ = writeln!(rendered, "\n## Variables");
    let _ = writeln!(rendered, "```json");
    let _ = writeln!(rendered, "{variables}");
    let _ = writeln!(rendered, "```");
}
