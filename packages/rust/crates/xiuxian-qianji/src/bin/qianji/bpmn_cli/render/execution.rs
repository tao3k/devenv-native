use std::fmt::Write as _;

use crate::bpmn_cli::deps::{BpmnAdvanceOutcome, Path, QianjiBpmnSession};
use crate::bpmn_cli::types::{
    BpmnCliOutput, BpmnEventPollCliCommand, BpmnExecutionRenderContext, BpmnResumeCliCommand,
    BpmnRunCliCommand, BpmnStartCliCommand, BpmnTaskCompleteCliCommand,
};

use super::support::{
    append_bpmn_wait_registrations, bpmn_checkpoint_backend_label,
    bpmn_checkpoint_backend_selection_label, bpmn_lifecycle_label, bpmn_outcome_label,
    bpmn_suspend_reason_label,
};

pub(crate) fn render_bpmn_start_output(
    command: &BpmnStartCliCommand,
    session: &QianjiBpmnSession,
    outcome: &BpmnAdvanceOutcome,
    render_context: &BpmnExecutionRenderContext<'_>,
) -> BpmnCliOutput {
    render_bpmn_execution_output(
        "BPMN Start",
        command.process_id.as_str(),
        command.instance_id.as_str(),
        session,
        outcome,
        render_context,
    )
}

pub(crate) fn render_bpmn_run_output(
    command: &BpmnRunCliCommand,
    session: &QianjiBpmnSession,
    outcome: &BpmnAdvanceOutcome,
    render_context: &BpmnExecutionRenderContext<'_>,
) -> BpmnCliOutput {
    render_bpmn_execution_output(
        "BPMN Run",
        command.process_id.as_str(),
        command.instance_id.as_str(),
        session,
        outcome,
        render_context,
    )
}

pub(crate) fn render_bpmn_resume_output(
    command: &BpmnResumeCliCommand,
    session: &QianjiBpmnSession,
    outcome: &BpmnAdvanceOutcome,
    render_context: &BpmnExecutionRenderContext<'_>,
) -> BpmnCliOutput {
    render_bpmn_execution_output(
        "BPMN Resume",
        session.instance().process.process_id.as_ref(),
        command.instance_id.as_str(),
        session,
        outcome,
        render_context,
    )
}

pub(crate) fn render_bpmn_event_poll_output(
    command: &BpmnEventPollCliCommand,
    session: &QianjiBpmnSession,
    outcome: &BpmnAdvanceOutcome,
    render_context: &BpmnExecutionRenderContext<'_>,
) -> BpmnCliOutput {
    render_bpmn_execution_output(
        "BPMN Event Poll",
        session.instance().process.process_id.as_ref(),
        command.instance_id.as_str(),
        session,
        outcome,
        render_context,
    )
}

pub(crate) fn render_bpmn_task_complete_output(
    command: &BpmnTaskCompleteCliCommand,
    session: &QianjiBpmnSession,
    outcome: &BpmnAdvanceOutcome,
    render_context: &BpmnExecutionRenderContext<'_>,
) -> BpmnCliOutput {
    render_bpmn_execution_output(
        "BPMN Task Complete",
        session.instance().process.process_id.as_ref(),
        command.instance_id.as_str(),
        session,
        outcome,
        render_context,
    )
}

pub(crate) fn render_bpmn_resume_missing_output(command: &BpmnResumeCliCommand) -> BpmnCliOutput {
    BpmnCliOutput {
        rendered: format!(
            "# BPMN Resume\n\nSource: {}\nInstance: {}\nCheckpoint backend: {}\nCheckpoint status: missing\n",
            command.bpmn_path.display(),
            command.instance_id,
            bpmn_checkpoint_backend_selection_label(&command.checkpoint_backend),
        ),
        exit_code: 1,
    }
}

pub(crate) fn render_bpmn_event_poll_missing_output(
    command: &BpmnEventPollCliCommand,
) -> BpmnCliOutput {
    BpmnCliOutput {
        rendered: format!(
            "# BPMN Event Poll\n\nSource: {}\nInstance: {}\nCheckpoint backend: {}\nCheckpoint status: missing\n",
            command.bpmn_path.display(),
            command.instance_id,
            bpmn_checkpoint_backend_selection_label(&command.checkpoint_backend),
        ),
        exit_code: 1,
    }
}

pub(crate) fn render_bpmn_task_complete_missing_output(
    command: &BpmnTaskCompleteCliCommand,
) -> BpmnCliOutput {
    BpmnCliOutput {
        rendered: format!(
            "# BPMN Task Complete\n\nSource: {}\nInstance: {}\nCheckpoint backend: {}\nCheckpoint status: missing\n",
            command.bpmn_path.display(),
            command.instance_id,
            bpmn_checkpoint_backend_selection_label(&command.checkpoint_backend),
        ),
        exit_code: 1,
    }
}

fn render_bpmn_execution_output(
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

    if !render_context.resolved_dmn_paths.is_empty() {
        let _ = writeln!(rendered, "\n## DMN Sources");
        for path in render_context.resolved_dmn_paths {
            let _ = writeln!(rendered, "- {}", path.display());
        }
    }

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

    append_bpmn_wait_registrations(&mut rendered, session.package(), session.instance());

    let _ = writeln!(rendered, "\n## Variables");
    let _ = writeln!(rendered, "```json");
    let _ = writeln!(rendered, "{variables}");
    let _ = writeln!(rendered, "```");

    BpmnCliOutput {
        rendered,
        exit_code: if matches!(outcome, BpmnAdvanceOutcome::Failed(_)) {
            2
        } else {
            0
        },
    }
}
