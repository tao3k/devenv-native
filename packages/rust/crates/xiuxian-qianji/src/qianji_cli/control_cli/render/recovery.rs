use std::io;

use xiuxian_qianji_control::RunRecoverySnapshot;
#[cfg(all(feature = "duckdb", feature = "valkey"))]
use xiuxian_qianji_control::{
    RecoveryActionApplication, RecoveryLoopApplication, RecoveryPlanAction,
};

#[cfg(all(feature = "duckdb", feature = "valkey"))]
use super::push_fmt;

pub(crate) fn render_recovery_snapshot_text(snapshot: &RunRecoverySnapshot) -> String {
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

pub(crate) fn render_recovery_snapshot_json(snapshot: &RunRecoverySnapshot) -> io::Result<String> {
    serde_json::to_string_pretty(snapshot).map_err(io::Error::other)
}

#[cfg(all(feature = "duckdb", feature = "valkey"))]
pub(crate) fn render_recovery_loop_text(application: &RecoveryLoopApplication) -> String {
    let applied_count = application
        .action_results
        .iter()
        .filter(|result| recovery_application_applied(&result.result))
        .count();
    let skipped_count = application.action_results.len() - applied_count;
    let mut output = format!(
        concat!(
            "# Qianji Control Recovery Application\n\n",
            "- Attempt event sequence: `{}`\n",
            "- Actions: `{}`\n",
            "- Applied actions: `{}`\n",
            "- Skipped actions: `{}`\n"
        ),
        application.attempt_record.sequence,
        application.action_results.len(),
        applied_count,
        skipped_count
    );

    if !application.action_results.is_empty() {
        output.push_str("\n## Actions\n\n");
        for action in &application.action_results {
            push_fmt(
                &mut output,
                format_args!(
                    "- `{}` -> `{}`\n",
                    recovery_action_label(&action.action),
                    recovery_application_label(&action.result)
                ),
            );
        }
    }

    output
}

#[cfg(all(feature = "duckdb", feature = "valkey"))]
pub(crate) fn render_recovery_loop_json(
    application: &RecoveryLoopApplication,
) -> io::Result<String> {
    serde_json::to_string_pretty(application).map_err(io::Error::other)
}

#[cfg(all(feature = "duckdb", feature = "valkey"))]
fn recovery_application_applied(application: &RecoveryActionApplication) -> bool {
    !matches!(application, RecoveryActionApplication::NotApplicable { .. })
}

#[cfg(all(feature = "duckdb", feature = "valkey"))]
fn recovery_application_label(application: &RecoveryActionApplication) -> &'static str {
    match application {
        RecoveryActionApplication::AppliedStepRetry { .. } => "applied_step_retry",
        RecoveryActionApplication::AppliedActivityRetry { .. } => "applied_activity_retry",
        RecoveryActionApplication::AppliedTimerFire { .. } => "applied_timer_fire",
        RecoveryActionApplication::AppliedLeaseReclaim { .. } => "applied_lease_reclaim",
        RecoveryActionApplication::NotApplicable { .. } => "not_applicable",
    }
}

#[cfg(all(feature = "duckdb", feature = "valkey"))]
fn recovery_action_label(action: &RecoveryPlanAction) -> &'static str {
    match action {
        RecoveryPlanAction::ReclaimExpiredLease { .. } => "reclaim_expired_lease",
        RecoveryPlanAction::FireTimer { .. } => "fire_timer",
        RecoveryPlanAction::RetryActivity { .. } => "retry_activity",
        RecoveryPlanAction::ReviewRetryableActivity { .. } => "review_retryable_activity",
        RecoveryPlanAction::EscalateTerminalActivity { .. } => "escalate_terminal_activity",
        RecoveryPlanAction::ReconcileScheduledActivity { .. } => "reconcile_scheduled_activity",
        RecoveryPlanAction::InspectInFlightActivity { .. } => "inspect_in_flight_activity",
        RecoveryPlanAction::AwaitHumanApproval { .. } => "await_human_approval",
        RecoveryPlanAction::AwaitHumanInput { .. } => "await_human_input",
        RecoveryPlanAction::InspectBlockedStep { .. } => "inspect_blocked_step",
        RecoveryPlanAction::PreserveActiveLease { .. } => "preserve_active_lease",
        RecoveryPlanAction::AwaitTimer { .. } => "await_timer",
    }
}
