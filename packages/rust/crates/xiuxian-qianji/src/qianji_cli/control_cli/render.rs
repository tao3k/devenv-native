use std::io;

use xiuxian_qianji_control::{
    ActivityView, AgentDecision, ControlEventKind, ControlEventRecord, RunId, RunRecoverySnapshot,
    RunView, StepView, TimerView,
};

pub(super) fn render_control_history_text(
    run_id: &RunId,
    records: &[ControlEventRecord],
) -> String {
    let mut output = format!(
        concat!(
            "# Qianji Control History\n\n",
            "- Run: `{}`\n",
            "- Events: `{}`\n\n"
        ),
        run_id.as_str(),
        records.len()
    );

    for record in records {
        let scope = record
            .event
            .step_id
            .as_ref()
            .map_or_else(|| "run".to_string(), |step_id| format!("step:{step_id}"));
        output.push_str("- #");
        output.push_str(&record.sequence.to_string());
        output.push_str(" @");
        output.push_str(&record.event.occurred_at_ms.to_string());
        output.push_str(" [");
        output.push_str(&scope);
        output.push_str("] `");
        output.push_str(control_event_kind_label(&record.event.kind));
        output.push_str("`\n");
    }

    output
}

pub(super) fn render_control_history_json(records: &[ControlEventRecord]) -> io::Result<String> {
    serde_json::to_string_pretty(records).map_err(io::Error::other)
}

pub(super) fn render_recovery_snapshot_text(snapshot: &RunRecoverySnapshot) -> String {
    let summary = &snapshot.summary;
    format!(
        concat!(
            "# Qianji Control Recovery Snapshot\n\n",
            "- Run: `{}`\n",
            "- Observed at ms: `{}`\n",
            "- Total actions: `{}`\n",
            "- Retry activities: `{}`\n",
            "- Terminal escalations: `{}`\n",
            "- Human approvals: `{}`\n",
            "- Blocked steps: `{}`\n",
            "- Fireable timers: `{}`\n",
            "- Expired leases: `{}`\n"
        ),
        snapshot.run_id.as_str(),
        snapshot.observed_at_ms,
        summary.total_actions,
        summary.retry_activities,
        summary.terminal_activity_escalations,
        summary.await_human_approvals,
        summary.inspect_blocked_steps,
        summary.fireable_timers,
        summary.reclaim_expired_leases
    )
}

pub(super) fn render_recovery_snapshot_json(snapshot: &RunRecoverySnapshot) -> io::Result<String> {
    serde_json::to_string_pretty(snapshot).map_err(io::Error::other)
}

pub(super) fn render_signal_append_text(record: &ControlEventRecord) -> String {
    let (signal_name, metadata) = match &record.event.kind {
        ControlEventKind::SignalReceived { signal } => (
            signal.signal_name.as_str(),
            serde_json::to_string(&signal.metadata).unwrap_or_else(|_| "null".to_string()),
        ),
        _ => ("<unknown>", "null".to_string()),
    };
    let scope = record
        .event
        .step_id
        .as_ref()
        .map_or("run", |step_id| step_id.as_str());

    format!(
        concat!(
            "# Qianji Control Signal\n\n",
            "- Event sequence: `{}`\n",
            "- Run: `{}`\n",
            "- Scope: `{}`\n",
            "- Signal: `{}`\n",
            "- Received at ms: `{}`\n",
            "- Payload metadata: `{}`\n"
        ),
        record.sequence,
        record.event.run_id.as_str(),
        scope,
        signal_name,
        record.event.occurred_at_ms,
        metadata
    )
}

pub(super) fn render_signal_append_json(record: &ControlEventRecord) -> io::Result<String> {
    serde_json::to_string_pretty(record).map_err(io::Error::other)
}

pub(super) fn render_activity_view_text(activity: &ActivityView) -> String {
    let task_type = activity
        .task
        .as_ref()
        .map_or("<unknown>", |task| task.activity_type.as_str());
    let task_queue = activity
        .task
        .as_ref()
        .map_or("<unknown>", |task| task.task_queue.as_str());
    let idempotency_key = activity
        .task
        .as_ref()
        .map_or("<unknown>", |task| task.idempotency_key.as_str());
    let timeout_ms = activity
        .task
        .as_ref()
        .and_then(|task| task.timeout_ms)
        .map_or_else(|| "<none>".to_string(), |timeout| timeout.to_string());
    let worker = activity
        .worker_id
        .as_ref()
        .map_or("<none>", |worker| worker.as_str());
    let result_hash = activity
        .result
        .as_ref()
        .and_then(|result| result.output_hash.as_deref())
        .unwrap_or("<none>");
    let failure_code = activity
        .failure
        .as_ref()
        .map_or("<none>", |failure| failure.error_code.as_str());
    let failure_retryable = activity
        .failure
        .as_ref()
        .map_or("<none>".to_string(), |failure| {
            failure.retryable.to_string()
        });

    format!(
        concat!(
            "# Qianji Control Activity\n\n",
            "- Activity: `{}`\n",
            "- Status: `{}`\n",
            "- Updated at ms: `{}`\n",
            "- Activity type: `{}`\n",
            "- Task queue: `{}`\n",
            "- Idempotency key: `{}`\n",
            "- Timeout ms: `{}`\n",
            "- Attempt: `{}`\n",
            "- Worker: `{}`\n",
            "- Result hash: `{}`\n",
            "- Failure code: `{}`\n",
            "- Failure retryable: `{}`\n"
        ),
        activity.activity_id.as_str(),
        serde_status(&activity.status),
        activity.updated_at_ms,
        task_type,
        task_queue,
        idempotency_key,
        timeout_ms,
        activity.attempt,
        worker,
        result_hash,
        failure_code,
        failure_retryable
    )
}

pub(super) fn render_activity_view_json(activity: &ActivityView) -> io::Result<String> {
    serde_json::to_string_pretty(activity).map_err(io::Error::other)
}

pub(super) fn render_agent_decision_text(decision: &AgentDecision) -> String {
    let scheduled_activity = decision
        .scheduled_activity_id
        .as_ref()
        .map_or("<none>", |activity_id| activity_id.as_str());
    let checkpoint_seq = decision
        .checkpoint_seq
        .map_or_else(|| "<none>".to_string(), |sequence| sequence.to_string());
    let gate_name = decision
        .gate_result
        .as_ref()
        .map_or("<none>", |gate| gate.gate_name.as_str());
    let gate_passed = decision
        .gate_result
        .as_ref()
        .map_or("<none>".to_string(), |gate| gate.passed.to_string());
    let gate_evidence_covered = decision
        .gate_result
        .as_ref()
        .map_or("<none>".to_string(), |gate| {
            gate.required_evidence_covered.to_string()
        });
    let gate_selected_count = decision
        .gate_result
        .as_ref()
        .map_or(0, |gate| gate.selected_required_evidence.len());
    let gate_missing_count = decision
        .gate_result
        .as_ref()
        .map_or(0, |gate| gate.missing_required_evidence.len());

    format!(
        concat!(
            "# Qianji Control Decision\n\n",
            "- Decision: `{}`\n",
            "- Proposal: `{}`\n",
            "- Outcome: `{}`\n",
            "- Reason code: `{}`\n",
            "- Scheduled activity: `{}`\n",
            "- Checkpoint seq: `{}`\n",
            "- Gate: `{}`\n",
            "- Gate passed: `{}`\n",
            "- Gate evidence covered: `{}`\n",
            "- Gate selected evidence: `{}`\n",
            "- Gate missing evidence: `{}`\n"
        ),
        decision.decision_id.as_str(),
        decision.proposal_id.as_str(),
        serde_status(&decision.outcome),
        decision.reason_code.as_str(),
        scheduled_activity,
        checkpoint_seq,
        gate_name,
        gate_passed,
        gate_evidence_covered,
        gate_selected_count,
        gate_missing_count
    )
}

pub(super) fn render_agent_decision_json(decision: &AgentDecision) -> io::Result<String> {
    serde_json::to_string_pretty(decision).map_err(io::Error::other)
}

pub(super) fn render_run_view_text(view: &RunView) -> String {
    let step_activity_count = view
        .steps
        .values()
        .map(|step| step.activities.len())
        .sum::<usize>();
    let step_timer_count = view
        .steps
        .values()
        .map(|step| step.timers.len())
        .sum::<usize>();
    let mut output = format!(
        concat!(
            "# Qianji Control View\n\n",
            "- Run: `{}`\n",
            "- Status: `{}`\n",
            "- Updated at ms: `{}`\n",
            "- Steps: `{}`\n",
            "- Run activities: `{}`\n",
            "- Step activities: `{}`\n",
            "- Run timers: `{}`\n",
            "- Step timers: `{}`\n",
            "- Signals: `{}`\n",
            "- Total cost usd micros: `{}`\n"
        ),
        view.run_id.as_str(),
        serde_status(&view.status),
        view.updated_at_ms,
        view.steps.len(),
        view.activities.len(),
        step_activity_count,
        view.timers.len(),
        step_timer_count,
        view.signals.len(),
        view.total_cost_usd_micros()
    );

    if !view.steps.is_empty() {
        output.push_str("\n## Steps\n\n");
        for step in view.steps.values() {
            render_step_summary(&mut output, step);
        }
    }

    output
}

pub(super) fn render_run_view_json(view: &RunView) -> io::Result<String> {
    serde_json::to_string_pretty(view).map_err(io::Error::other)
}

pub(super) fn render_step_view_text(step: &StepView) -> String {
    let title = step.title.as_deref().unwrap_or("<untitled>");
    format!(
        concat!(
            "# Qianji Control Step\n\n",
            "- Step: `{}`\n",
            "- Status: `{}`\n",
            "- Title: `{}`\n",
            "- Updated at ms: `{}`\n",
            "- Required evidence: `{}`\n",
            "- Covered evidence: `{}`\n",
            "- Evidence refs: `{}`\n",
            "- Artifacts: `{}`\n",
            "- Activities: `{}`\n",
            "- Agent proposals: `{}`\n",
            "- Agent decisions: `{}`\n",
            "- Gates: `{}`\n",
            "- Timers: `{}`\n",
            "- Signals: `{}`\n",
            "- Version pins: `{}`\n",
            "- Recovery attempts: `{}`\n",
            "- Total cost usd micros: `{}`\n"
        ),
        step.step_id.as_str(),
        serde_status(&step.status),
        title,
        step.updated_at_ms,
        step.required_evidence.len(),
        step.covered_required_evidence().len(),
        step.evidence.len(),
        step.artifacts.len(),
        step.activities.len(),
        step.agent_proposals.len(),
        step.agent_decisions.len(),
        step.gate_results.len(),
        step.timers.len(),
        step.signals.len(),
        step.version_pins.len(),
        step.recovery_attempts.len(),
        step.total_cost_usd_micros()
    )
}

pub(super) fn render_step_view_json(step: &StepView) -> io::Result<String> {
    serde_json::to_string_pretty(step).map_err(io::Error::other)
}

pub(super) fn render_timer_view_text(timer: &TimerView) -> String {
    let scheduled_ms = timer
        .timer
        .as_ref()
        .map(|record| record.fire_at_ms.to_string());
    let completed_ms = timer.fired_at_ms.map(|fired_at| fired_at.to_string());

    format!(
        concat!(
            "# Qianji Control Timer\n\n",
            "- Timer: `{}`\n",
            "- Status: `{}`\n",
            "- Fire at ms: `{}`\n",
            "- Fired at ms: `{}`\n",
            "- Updated at ms: `{}`\n"
        ),
        timer.timer_id.as_str(),
        serde_status(&timer.status),
        scheduled_ms.as_deref().unwrap_or("<none>"),
        completed_ms.as_deref().unwrap_or("<none>"),
        timer.updated_at_ms
    )
}

pub(super) fn render_timer_view_json(timer: &TimerView) -> io::Result<String> {
    serde_json::to_string_pretty(timer).map_err(io::Error::other)
}

fn render_step_summary(output: &mut String, step: &StepView) {
    let title = step.title.as_deref().unwrap_or("<untitled>");
    output.push_str("- `");
    output.push_str(step.step_id.as_str());
    output.push_str("` [");
    output.push_str(&serde_status(&step.status));
    output.push_str("] ");
    output.push_str(title);
    output.push_str(" evidence `");
    output.push_str(&step.covered_required_evidence().len().to_string());
    output.push('/');
    output.push_str(&step.required_evidence.len().to_string());
    output.push_str("` activities `");
    output.push_str(&step.activities.len().to_string());
    output.push_str("` gates `");
    output.push_str(&step.gate_results.len().to_string());
    output.push_str("` updated `");
    output.push_str(&step.updated_at_ms.to_string());
    output.push_str("`\n");
}

fn serde_status<T>(status: &T) -> String
where
    T: serde::Serialize,
{
    serde_json::to_string(status)
        .unwrap_or_else(|_| "\"unknown\"".to_string())
        .trim_matches('"')
        .to_string()
}

fn control_event_kind_label(kind: &ControlEventKind) -> &'static str {
    match kind {
        ControlEventKind::RunCreated { .. } => "run_created",
        ControlEventKind::RunAdmitted => "run_admitted",
        ControlEventKind::PlanRecorded { .. } => "plan_recorded",
        ControlEventKind::StepCreated { .. } => "step_created",
        ControlEventKind::StepQueued => "step_queued",
        ControlEventKind::StepLeaseAcquired { .. } => "step_lease_acquired",
        ControlEventKind::StepLeaseRenewed { .. } => "step_lease_renewed",
        ControlEventKind::StepLeaseReleased { .. } => "step_lease_released",
        ControlEventKind::StepStarted => "step_started",
        ControlEventKind::StepWaiting { .. } => "step_waiting",
        ControlEventKind::ToolCallRecorded { .. } => "tool_call_recorded",
        ControlEventKind::AgentProposalRecorded { .. } => "agent_proposal_recorded",
        ControlEventKind::AgentDecisionRecorded { .. } => "agent_decision_recorded",
        ControlEventKind::ActivityScheduled { .. } => "activity_scheduled",
        ControlEventKind::ActivityStarted { .. } => "activity_started",
        ControlEventKind::ActivityCompleted { .. } => "activity_completed",
        ControlEventKind::ActivityFailed { .. } => "activity_failed",
        ControlEventKind::SignalReceived { .. } => "signal_received",
        ControlEventKind::TimerScheduled { .. } => "timer_scheduled",
        ControlEventKind::TimerFired { .. } => "timer_fired",
        ControlEventKind::VersionPinned { .. } => "version_pinned",
        ControlEventKind::ArtifactAttached { .. } => "artifact_attached",
        ControlEventKind::EvidenceAttached { .. } => "evidence_attached",
        ControlEventKind::CostObserved { .. } => "cost_observed",
        ControlEventKind::GateEvaluated { .. } => "gate_evaluated",
        ControlEventKind::RecoveryStarted { .. } => "recovery_started",
        ControlEventKind::WorkerHeartbeatObserved { .. } => "worker_heartbeat_observed",
        ControlEventKind::StepSucceeded => "step_succeeded",
        ControlEventKind::StepFailed { .. } => "step_failed",
        ControlEventKind::StepBlocked { .. } => "step_blocked",
        ControlEventKind::StepCancelled { .. } => "step_cancelled",
        ControlEventKind::RunCompleted => "run_completed",
        ControlEventKind::RunFailed { .. } => "run_failed",
        ControlEventKind::RunBlocked { .. } => "run_blocked",
        ControlEventKind::RunAborted { .. } => "run_aborted",
    }
}
