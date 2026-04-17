use crate::agent::SessionRecallFeedbackDirection;
use crate::agent::{
    DownstreamAdmissionRuntimeSnapshot, MemoryRecallMetricsSnapshot, MemoryRuntimeStatusSnapshot,
    SessionContextBudgetSnapshot, SessionContextSnapshotInfo, SessionContextWindowInfo,
    SessionMemoryRecallSnapshot,
};
use crate::channels::managed_runtime::replies as shared_replies;
use crate::jobs::{JobMetricsSnapshot, JobStatusSnapshot};

const PERMISSION_HINTS: shared_replies::PermissionHints<'static> =
    shared_replies::PermissionHints {
        control_command_hint: "Ask an identity allowed by `telegram.control_command_allow_from` (or matching `telegram.admin_command_rules` / `telegram.admin_users`) to run this command.",
        slash_command_hint: "Ask an admin to grant this command via telegram slash command allowlist settings.",
    };

pub(in super::super) fn format_job_status(snapshot: &JobStatusSnapshot) -> String {
    shared_replies::format_job_status(snapshot)
}

pub(in super::super) fn format_job_metrics(metrics: &JobMetricsSnapshot) -> String {
    shared_replies::format_job_metrics(metrics)
}

pub(in super::super) fn format_job_not_found(job_id: &str) -> String {
    shared_replies::format_job_not_found(job_id)
}

pub(in super::super) fn format_job_status_json(snapshot: &JobStatusSnapshot) -> String {
    shared_replies::format_job_status_json(snapshot)
}

pub(in super::super) fn format_job_metrics_json(metrics: &JobMetricsSnapshot) -> String {
    shared_replies::format_job_metrics_json(metrics)
}

pub(in super::super) fn format_job_not_found_json(job_id: &str) -> String {
    shared_replies::format_job_not_found_json(job_id)
}

pub(in super::super) fn format_session_feedback(
    direction: SessionRecallFeedbackDirection,
    previous_bias: f32,
    updated_bias: f32,
) -> String {
    shared_replies::format_session_feedback(direction, previous_bias, updated_bias)
}

pub(in super::super) fn format_session_feedback_json(
    direction: SessionRecallFeedbackDirection,
    previous_bias: f32,
    updated_bias: f32,
) -> String {
    shared_replies::format_session_feedback_json(direction, previous_bias, updated_bias)
}

pub(in super::super) fn format_session_feedback_unavailable_json() -> String {
    shared_replies::format_session_feedback_unavailable_json()
}

pub(in super::super) fn format_control_command_admin_required(
    command: &str,
    sender: &str,
) -> String {
    shared_replies::format_control_command_admin_required(command, sender, PERMISSION_HINTS)
}

pub(in super::super) fn format_slash_command_permission_required(
    command: &str,
    sender: &str,
) -> String {
    shared_replies::format_slash_command_permission_required(command, sender, PERMISSION_HINTS)
}

pub(in super::super) fn format_slash_help() -> String {
    shared_replies::format_slash_help()
}

pub(in super::super) fn format_slash_help_json() -> String {
    shared_replies::format_slash_help_json()
}

pub(in super::super) fn format_command_error_json(command: &str, error: &str) -> String {
    shared_replies::format_command_error_json(command, error)
}

pub(in super::super) fn format_session_context_snapshot(
    session_id: &str,
    partition_key: &str,
    partition_mode: &str,
    active: SessionContextWindowInfo,
    snapshot: Option<SessionContextSnapshotInfo>,
    admission: DownstreamAdmissionRuntimeSnapshot,
) -> String {
    shared_replies::format_session_context_snapshot(
        session_id,
        partition_key,
        partition_mode,
        active,
        snapshot,
        admission,
    )
}

pub(in super::super) fn format_session_context_snapshot_json(
    session_id: &str,
    partition_key: &str,
    partition_mode: &str,
    active: SessionContextWindowInfo,
    snapshot: Option<SessionContextSnapshotInfo>,
    admission: DownstreamAdmissionRuntimeSnapshot,
) -> String {
    shared_replies::format_session_context_snapshot_json(
        session_id,
        partition_key,
        partition_mode,
        active,
        snapshot,
        admission,
    )
}

pub(in super::super) fn format_context_budget_snapshot(
    snapshot: &SessionContextBudgetSnapshot,
) -> String {
    shared_replies::format_context_budget_snapshot(snapshot)
}

pub(in super::super) fn format_context_budget_snapshot_json(
    snapshot: &SessionContextBudgetSnapshot,
) -> String {
    shared_replies::format_context_budget_snapshot_json(snapshot)
}

pub(in super::super) fn format_context_budget_not_found_json() -> String {
    shared_replies::format_context_budget_not_found_json()
}

pub(in super::super) fn format_memory_recall_snapshot(
    snapshot: SessionMemoryRecallSnapshot,
    metrics: MemoryRecallMetricsSnapshot,
    runtime_status: MemoryRuntimeStatusSnapshot,
    admission_status: DownstreamAdmissionRuntimeSnapshot,
    session_scope: &str,
) -> String {
    shared_replies::format_memory_recall_snapshot(
        snapshot,
        metrics,
        runtime_status,
        admission_status,
        session_scope,
    )
}

pub(in super::super) fn format_memory_recall_snapshot_json(
    snapshot: SessionMemoryRecallSnapshot,
    metrics: MemoryRecallMetricsSnapshot,
    runtime_status: &MemoryRuntimeStatusSnapshot,
    admission_status: DownstreamAdmissionRuntimeSnapshot,
    session_scope: &str,
) -> String {
    shared_replies::format_memory_recall_snapshot_json(
        snapshot,
        metrics,
        runtime_status,
        admission_status,
        session_scope,
    )
}

pub(in super::super) fn format_memory_recall_not_found(
    runtime_status: MemoryRuntimeStatusSnapshot,
    admission_status: DownstreamAdmissionRuntimeSnapshot,
    session_scope: &str,
) -> String {
    shared_replies::format_memory_recall_not_found(runtime_status, admission_status, session_scope)
}

pub(in super::super) fn format_memory_recall_not_found_json(
    metrics: MemoryRecallMetricsSnapshot,
    runtime_status: &MemoryRuntimeStatusSnapshot,
    admission_status: DownstreamAdmissionRuntimeSnapshot,
    session_scope: &str,
) -> String {
    shared_replies::format_memory_recall_not_found_json(
        metrics,
        runtime_status,
        admission_status,
        session_scope,
    )
}

pub(in super::super) fn format_memory_recall_compact_snapshot(
    snapshot: SessionMemoryRecallSnapshot,
    runtime_status: &MemoryRuntimeStatusSnapshot,
    admission_status: DownstreamAdmissionRuntimeSnapshot,
    session_scope: &str,
) -> String {
    shared_replies::format_memory_recall_compact_snapshot(
        snapshot,
        runtime_status,
        admission_status,
        session_scope,
    )
}

pub(in super::super) fn format_memory_recall_compact_not_found(
    runtime_status: &MemoryRuntimeStatusSnapshot,
    admission_status: DownstreamAdmissionRuntimeSnapshot,
    session_scope: &str,
) -> String {
    shared_replies::format_memory_recall_compact_not_found(
        runtime_status,
        admission_status,
        session_scope,
    )
}
