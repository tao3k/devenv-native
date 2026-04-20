use std::fmt::Write as _;

use super::deps::{
    BpmnAdvanceOutcome, BpmnEventKind, BpmnInstanceState, BpmnPackage, BpmnProcessSpec,
    BpmnTimerKind, BpmnTimerSpec, InstanceLifecycle, Path, QianjiBpmnCheckpointStore,
    QianjiBpmnSession, SuspendReason, WaitKind,
};
use super::types::{BpmnCliOutput, BpmnRunCliCommand, BpmnRunRenderContext};

pub(super) fn render_bpmn_run_output(
    command: &BpmnRunCliCommand,
    session: &QianjiBpmnSession,
    outcome: &BpmnAdvanceOutcome,
    render_context: &BpmnRunRenderContext<'_>,
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
        "# BPMN Run\n\nSource: {}\nProcess: {}\nInstance: {}\nPackage: {}\nOutcome: {}\nLifecycle: {}\nCheckpoint backend: {}\nCheckpoint source: {}\nCheckpoint saved: {}\nCheckpoint deleted: {}\nHost fixture: {}\nEvent fixture: {}\nDMN sources: {}\nSequence: {}\nActive tokens: {}\nPending host work: {}\nWait registrations: {}\n",
        render_context.resolved_bpmn_path.display(),
        command.process_id,
        command.instance_id,
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

fn append_bpmn_wait_registrations(
    rendered: &mut String,
    package: &BpmnPackage,
    instance: &BpmnInstanceState,
) {
    if instance.waits.is_empty() {
        return;
    }

    let Some(process) = package.find_process(instance.process.process_id.as_ref()) else {
        return;
    };
    let mut wait_lines = instance
        .waits
        .iter()
        .map(|wait| {
            let wait_id = render_bpmn_wait_node_id(process, wait.node_index);
            let mut line = format!("- {wait_id} | kind={}", bpmn_wait_kind_label(&wait.kind));
            if let Some(event_kind) = wait.event_kind.as_ref() {
                let _ = write!(line, " | event={}", bpmn_event_kind_label(event_kind));
            }
            if let Some(reference) = wait.event_reference.as_ref() {
                let _ = write!(line, " | ref={reference}");
            }
            if let Some(name) = wait.event_name.as_ref() {
                let _ = write!(line, " | name={name}");
            }
            if let Some(timer) = wait.timer.as_ref() {
                let _ = write!(line, " | timer={}", bpmn_timer_spec_label(timer));
            }
            if let Some(blocking_node_index) = wait.blocking_node_index {
                let _ = write!(
                    line,
                    " | blocking={}",
                    render_bpmn_wait_node_id(process, blocking_node_index)
                );
            }
            if let Some(correlation_key) = wait.correlation_key.as_ref() {
                let _ = write!(line, " | correlation={correlation_key}");
            }
            (wait_id, line)
        })
        .collect::<Vec<_>>();
    wait_lines.sort_by(|left, right| left.0.cmp(&right.0));

    let _ = writeln!(rendered, "\n## Wait Registrations");

    if let Some(competition) = instance.event_competition.as_ref() {
        let _ = writeln!(
            rendered,
            "Competition gateway: {}",
            render_bpmn_wait_node_id(process, competition.gateway_node_index)
        );
    }

    let wait_key = wait_lines
        .iter()
        .map(|(wait_id, _)| wait_id.as_str())
        .collect::<Vec<_>>()
        .join("|");
    let _ = writeln!(rendered, "Event fixture key: {wait_key}");

    for (_, line) in wait_lines {
        let _ = writeln!(rendered, "{line}");
    }
}

fn render_bpmn_wait_node_id(process: &BpmnProcessSpec, node_index: u32) -> String {
    process.nodes.get(node_index as usize).map_or_else(
        || format!("node#{node_index}"),
        |node| node.bpmn_id.to_string(),
    )
}

fn bpmn_checkpoint_backend_label(store: &QianjiBpmnCheckpointStore) -> &'static str {
    match store {
        QianjiBpmnCheckpointStore::Valkey { .. } => "runtime_valkey",
        #[cfg(feature = "sqlite")]
        QianjiBpmnCheckpointStore::Sqlite { .. } => "sqlite",
    }
}

fn bpmn_outcome_label(outcome: &BpmnAdvanceOutcome) -> &'static str {
    match outcome {
        BpmnAdvanceOutcome::Advanced => "advanced",
        BpmnAdvanceOutcome::BlockedOnHost(_) => "blocked_on_host",
        BpmnAdvanceOutcome::WaitingExternalEvent => "waiting_external_event",
        BpmnAdvanceOutcome::Suspended(_) => "suspended",
        BpmnAdvanceOutcome::Completed => "completed",
        BpmnAdvanceOutcome::Failed(_) => "failed",
    }
}

fn bpmn_lifecycle_label(lifecycle: &InstanceLifecycle) -> &'static str {
    match lifecycle {
        InstanceLifecycle::Ready => "ready",
        InstanceLifecycle::Running => "running",
        InstanceLifecycle::Waiting => "waiting",
        InstanceLifecycle::Suspended => "suspended",
        InstanceLifecycle::Completed => "completed",
        InstanceLifecycle::Failed => "failed",
    }
}

fn bpmn_suspend_reason_label(reason: &SuspendReason) -> &'static str {
    match reason {
        SuspendReason::HostRequested => "host_requested",
        SuspendReason::ExternalWait => "external_wait",
        SuspendReason::DmnPlaceholder => "dmn_placeholder",
        SuspendReason::ScaffoldBoundary => "scaffold_boundary",
    }
}

fn bpmn_wait_kind_label(kind: &WaitKind) -> &'static str {
    match kind {
        WaitKind::ExternalEvent => "external_event",
        WaitKind::UserAction => "user_action",
        WaitKind::Timer => "timer",
    }
}

fn bpmn_event_kind_label(kind: &BpmnEventKind) -> &'static str {
    match kind {
        BpmnEventKind::Timer => "timer",
        BpmnEventKind::Message => "message",
        BpmnEventKind::Signal => "signal",
        BpmnEventKind::Conditional => "conditional",
    }
}

fn bpmn_timer_spec_label(timer: &BpmnTimerSpec) -> String {
    let kind = match timer.kind {
        BpmnTimerKind::Date => "date",
        BpmnTimerKind::Duration => "duration",
        BpmnTimerKind::Cycle => "cycle",
    };
    format!("{kind}:{}", timer.expression)
}
