use std::io;

use xiuxian_qianji_control::{ControlEventKind, ControlEventRecord, RunId, RunRecoverySnapshot};

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
        output.push_str(&format!(
            "- #{} @{} [{}] `{}`\n",
            record.sequence,
            record.event.occurred_at_ms,
            scope,
            control_event_kind_label(&record.event.kind)
        ));
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
