//! Event labels and operator-readable messages.

use serde_json::Value;
use xiuxian_qianji_control::ControlEventKind;

pub(super) fn event_message(kind: &ControlEventKind) -> String {
    match kind {
        ControlEventKind::RunCreated { intent, .. } => {
            format!("run created: {}", bounded_operator_text(intent))
        }
        ControlEventKind::RunAdmitted => "run admitted by qianji control".to_owned(),
        ControlEventKind::PlanRecorded { summary } => summary.clone(),
        ControlEventKind::StepCreated { title, .. } => title.clone(),
        ControlEventKind::ToolCallRecorded { tool_name, .. } => {
            format!("tool call recorded: {}", tool_name)
        }
        ControlEventKind::AgentProposalRecorded { proposal } => agent_proposal_message(
            proposal.proposed_action.as_str(),
            proposal.tool_name.as_deref(),
        ),
        ControlEventKind::AgentDecisionRecorded { decision } => agent_decision_message(
            format!("{:?}", decision.outcome).as_str(),
            decision.reason_code.as_str(),
            decision
                .scheduled_activity_id
                .as_ref()
                .map(|activity_id| activity_id.as_str()),
        ),
        ControlEventKind::ActivityScheduled { task } => {
            format!(
                "{} scheduled on {}",
                task.activity_type.as_str(),
                task.task_queue.as_str()
            )
        }
        ControlEventKind::ActivityStarted { activity_id, .. } => {
            format!("{} started", activity_id.as_str())
        }
        ControlEventKind::ActivityCompleted {
            activity_id,
            result,
            ..
        } => activity_completion_message(activity_id.as_str(), &result.metadata),
        ControlEventKind::ActivityFailed { failure, .. } => failure.message.clone(),
        ControlEventKind::StepWaiting { reason } => format!("{reason:?}"),
        ControlEventKind::StepFailed { message, .. } | ControlEventKind::RunFailed { message } => {
            message.clone()
        }
        ControlEventKind::StepBlocked { reason }
        | ControlEventKind::StepCancelled { reason }
        | ControlEventKind::RunBlocked { reason }
        | ControlEventKind::RunAborted { reason } => reason.clone(),
        ControlEventKind::SignalReceived { signal } => {
            format!("signal received: {}", signal.signal_name.as_str())
        }
        ControlEventKind::TimerScheduled { timer } => {
            format!("timer scheduled: {}", timer.timer_id.as_str())
        }
        ControlEventKind::TimerFired { timer_id } => {
            format!("timer fired: {}", timer_id.as_str())
        }
        ControlEventKind::VersionPinned { pin } => {
            format!(
                "version pinned: {}={}",
                pin.version_key.as_str(),
                pin.version
            )
        }
        ControlEventKind::ArtifactAttached { artifact } => format!(
            "artifact attached: {} {}",
            artifact.artifact_kind.as_str(),
            bounded_operator_text(artifact.uri.as_str())
        ),
        ControlEventKind::EvidenceAttached { evidence } => evidence_message(
            evidence
                .requirement_key
                .as_deref()
                .unwrap_or_else(|| evidence.evidence_id.as_str()),
            evidence.source.as_str(),
            evidence.summary.as_deref(),
        ),
        ControlEventKind::CostObserved { observation } => cost_message(
            observation.provider.as_str(),
            observation.model.as_deref(),
            observation.observed_total_tokens(),
            observation.cost_usd_micros,
            observation.latency_ms,
        ),
        ControlEventKind::GateEvaluated { result } => gate_message(
            result.gate_name.as_str(),
            result.passed,
            result.required_evidence_covered,
            result.selected_required_evidence.len(),
            result.missing_required_evidence.len(),
        ),
        ControlEventKind::RecoveryStarted { attempt } => {
            format!("recovery attempt {}: {}", attempt.attempt, attempt.reason)
        }
        ControlEventKind::WorkerHeartbeatObserved { heartbeat } => worker_heartbeat_message(
            heartbeat.worker_id.as_str(),
            text_field(&heartbeat.metadata, "phase"),
            text_field(&heartbeat.metadata, "activity_id"),
        ),
        ControlEventKind::StepQueued
        | ControlEventKind::StepLeaseAcquired { .. }
        | ControlEventKind::StepLeaseRenewed { .. }
        | ControlEventKind::StepLeaseReleased { .. }
        | ControlEventKind::StepStarted
        | ControlEventKind::StepSucceeded
        | ControlEventKind::RunCompleted => event_kind_name(kind).replace('_', " "),
    }
}

fn agent_proposal_message(action: &str, tool_name: Option<&str>) -> String {
    match tool_name {
        Some(tool_name) => format!("agent proposed {action} with {tool_name}"),
        None => format!("agent proposed {action}"),
    }
}

fn agent_decision_message(outcome: &str, reason_code: &str, activity_id: Option<&str>) -> String {
    let outcome = snake_case_debug(outcome);
    match activity_id {
        Some(activity_id) => {
            format!("agent decision {outcome}: {reason_code}; scheduled {activity_id}")
        }
        None => format!("agent decision {outcome}: {reason_code}"),
    }
}

fn evidence_message(requirement: &str, source: &str, summary: Option<&str>) -> String {
    match summary {
        Some(summary) => format!(
            "evidence attached for {requirement} from {source}: {}",
            bounded_operator_text(summary)
        ),
        None => format!("evidence attached for {requirement} from {source}"),
    }
}

fn cost_message(
    provider: &str,
    model: Option<&str>,
    total_tokens: u64,
    cost_usd_micros: u64,
    latency_ms: Option<u64>,
) -> String {
    let lane = model.map_or_else(
        || provider.to_owned(),
        |model| format!("{provider}/{model}"),
    );
    let latency = latency_ms.map_or_else(String::new, |latency_ms| format!(" · {latency_ms} ms"));
    format!("cost observed: {lane} · {total_tokens} tokens · {cost_usd_micros} usd_micros{latency}")
}

fn gate_message(
    gate_name: &str,
    passed: bool,
    required_evidence_covered: bool,
    selected_count: usize,
    missing_count: usize,
) -> String {
    let status = if passed { "passed" } else { "failed" };
    let coverage = if required_evidence_covered {
        "covered"
    } else {
        "missing"
    };
    format!(
        "gate {gate_name} {status}: required evidence {coverage} ({selected_count} selected, {missing_count} missing)"
    )
}

fn worker_heartbeat_message(
    worker_id: &str,
    phase: Option<&str>,
    activity_id: Option<&str>,
) -> String {
    match (phase, activity_id) {
        (Some(phase), Some(activity_id)) => {
            format!("worker {worker_id} {phase}: {activity_id}")
        }
        (Some(phase), None) => format!("worker {worker_id} {phase}"),
        (None, Some(activity_id)) => format!("worker {worker_id} heartbeat: {activity_id}"),
        (None, None) => format!("worker {worker_id} heartbeat"),
    }
}

fn activity_completion_message(activity_id: &str, metadata: &serde_json::Value) -> String {
    if let Some(preview) = text_field(metadata, "response_preview") {
        return preview.to_owned();
    }
    if let Some(summary) = host_work_completion_summary(metadata) {
        return summary;
    }
    format!("{activity_id} completed")
}

fn host_work_completion_summary(metadata: &Value) -> Option<String> {
    let data = metadata
        .get("qianji_bpmn_host_work_completion")?
        .get("data")?;
    let rendered = match data {
        Value::String(value) => value.clone(),
        _ => serde_json::to_string(data).ok()?,
    };
    Some(format!(
        "host work completed: {}",
        bounded_operator_text(rendered.as_str())
    ))
}

fn text_field<'a>(metadata: &'a Value, key: &str) -> Option<&'a str> {
    metadata
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
}

fn bounded_operator_text(value: &str) -> String {
    const MAX_CHARS: usize = 512;
    value.chars().take(MAX_CHARS).collect()
}

fn snake_case_debug(value: &str) -> String {
    let mut rendered = String::new();
    for (index, ch) in value.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if index > 0 {
                rendered.push('_');
            }
            rendered.push(ch.to_ascii_lowercase());
        } else {
            rendered.push(ch);
        }
    }
    rendered
}

pub(super) fn event_kind_name(kind: &ControlEventKind) -> &'static str {
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
