//! Replay-derived recovery views.

use crate::{
    ActivityRetryDecision, ActivityStatus, ActivityView, AgentDecision, AgentDecisionOutcome,
    ControlResult, RunId, RunView, StepId, StepLease, StepStatus, TimerStatus, TimerView,
    WaitReason,
};

/// Scope for a recovery item.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "scope", rename_all = "snake_case")]
pub enum RecoveryItemScope {
    /// Run-scoped item.
    Run,
    /// Step-scoped item.
    Step {
        /// Owning step id.
        step_id: StepId,
    },
}

impl RecoveryItemScope {
    /// Returns run scope.
    #[must_use]
    pub const fn run() -> Self {
        Self::Run
    }

    /// Returns step scope for the supplied step id.
    #[must_use]
    pub fn step(step_id: StepId) -> Self {
        Self::Step { step_id }
    }
}

/// Activity recovery item.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActivityRecoveryItem {
    /// Run or step scope.
    pub scope: RecoveryItemScope,
    /// Replayed activity state.
    pub activity: ActivityView,
}

/// Failed activity recovery item.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FailedActivityRecoveryItem {
    /// Run or step scope.
    pub scope: RecoveryItemScope,
    /// Replayed failed activity state.
    pub activity: ActivityView,
    /// Deterministic retry decision when the task declared a retry policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_decision: Option<ActivityRetryDecision>,
}

/// Timer recovery item.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TimerRecoveryItem {
    /// Run or step scope.
    pub scope: RecoveryItemScope,
    /// Replayed timer state.
    pub timer: TimerView,
}

/// Agent decision recovery item.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AgentDecisionRecoveryItem {
    /// Run or step scope.
    pub scope: RecoveryItemScope,
    /// Approval-required Agent decision.
    pub decision: AgentDecision,
}

/// Step recovery item.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StepRecoveryItem {
    /// Step id.
    pub step_id: StepId,
    /// Replayed step status.
    pub status: StepStatus,
    /// Current wait reason when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wait_reason: Option<WaitReason>,
    /// Last error or block reason when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}

/// Lease recovery item.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LeaseRecoveryItem {
    /// Leased step id.
    pub step_id: StepId,
    /// Replayed lease.
    pub lease: StepLease,
}

/// Read-only recovery summary derived from a replayed run view.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RunRecoveryView {
    /// Run id.
    pub run_id: RunId,
    /// Caller-supplied observation time.
    pub now_ms: u64,
    /// Scheduled activities that have not started.
    #[serde(default)]
    pub scheduled_activities: Vec<ActivityRecoveryItem>,
    /// Activities that started and have not completed or failed.
    #[serde(default)]
    pub in_flight_activities: Vec<ActivityRecoveryItem>,
    /// Failed activities that may retry.
    #[serde(default)]
    pub retryable_failed_activities: Vec<FailedActivityRecoveryItem>,
    /// Failed activities that should not retry under current facts.
    #[serde(default)]
    pub terminal_failed_activities: Vec<FailedActivityRecoveryItem>,
    /// Scheduled timers that have not reached their fire time.
    #[serde(default)]
    pub pending_timers: Vec<TimerRecoveryItem>,
    /// Scheduled timers whose fire time has elapsed.
    #[serde(default)]
    pub fireable_timers: Vec<TimerRecoveryItem>,
    /// Agent decisions waiting for human approval.
    #[serde(default)]
    pub pending_approval_decisions: Vec<AgentDecisionRecoveryItem>,
    /// Steps waiting on human input.
    #[serde(default)]
    pub human_wait_steps: Vec<StepRecoveryItem>,
    /// Steps currently blocked.
    #[serde(default)]
    pub blocked_steps: Vec<StepRecoveryItem>,
    /// Step leases still active at `now_ms`.
    #[serde(default)]
    pub active_leases: Vec<LeaseRecoveryItem>,
    /// Step leases expired at `now_ms`.
    #[serde(default)]
    pub expired_leases: Vec<LeaseRecoveryItem>,
}

impl RunRecoveryView {
    fn new(run_id: RunId, now_ms: u64) -> Self {
        Self {
            run_id,
            now_ms,
            scheduled_activities: Vec::new(),
            in_flight_activities: Vec::new(),
            retryable_failed_activities: Vec::new(),
            terminal_failed_activities: Vec::new(),
            pending_timers: Vec::new(),
            fireable_timers: Vec::new(),
            pending_approval_decisions: Vec::new(),
            human_wait_steps: Vec::new(),
            blocked_steps: Vec::new(),
            active_leases: Vec::new(),
            expired_leases: Vec::new(),
        }
    }

    /// Returns true when the replayed run has recovery work to inspect.
    #[must_use]
    pub fn has_recovery_work(&self) -> bool {
        !self.scheduled_activities.is_empty()
            || !self.in_flight_activities.is_empty()
            || !self.retryable_failed_activities.is_empty()
            || !self.terminal_failed_activities.is_empty()
            || !self.pending_timers.is_empty()
            || !self.fireable_timers.is_empty()
            || !self.pending_approval_decisions.is_empty()
            || !self.human_wait_steps.is_empty()
            || !self.blocked_steps.is_empty()
            || !self.active_leases.is_empty()
            || !self.expired_leases.is_empty()
    }
}

impl RunView {
    /// Derives a read-only recovery summary from replayed durable history.
    ///
    /// # Errors
    ///
    /// Returns a control error when retry-policy evaluation fails for a
    /// replayed failed activity.
    pub fn recovery_view(&self, now_ms: u64) -> ControlResult<RunRecoveryView> {
        let mut recovery = RunRecoveryView::new(self.run_id.clone(), now_ms);

        collect_activities(&mut recovery, &RecoveryItemScope::run(), &self.activities)?;
        collect_timers(&mut recovery, &RecoveryItemScope::run(), &self.timers);
        collect_agent_decisions(
            &mut recovery,
            &RecoveryItemScope::run(),
            self.agent_decisions.values(),
        );

        for step in self.steps.values() {
            let scope = RecoveryItemScope::step(step.step_id.clone());
            collect_activities(&mut recovery, &scope, &step.activities)?;
            collect_timers(&mut recovery, &scope, &step.timers);
            collect_agent_decisions(&mut recovery, &scope, step.agent_decisions.values());
            collect_step_state(&mut recovery, step);
        }

        Ok(recovery)
    }
}

fn collect_activities(
    recovery: &mut RunRecoveryView,
    scope: &RecoveryItemScope,
    activities: &std::collections::BTreeMap<crate::ActivityId, ActivityView>,
) -> ControlResult<()> {
    for activity in activities.values() {
        match activity.status {
            ActivityStatus::Scheduled => {
                recovery.scheduled_activities.push(ActivityRecoveryItem {
                    scope: scope.clone(),
                    activity: activity.clone(),
                });
            }
            ActivityStatus::Started => {
                recovery.in_flight_activities.push(ActivityRecoveryItem {
                    scope: scope.clone(),
                    activity: activity.clone(),
                });
            }
            ActivityStatus::Failed => {
                collect_failed_activity(recovery, scope.clone(), activity)?;
            }
            ActivityStatus::Pending | ActivityStatus::Completed => {}
        }
    }
    Ok(())
}

fn collect_failed_activity(
    recovery: &mut RunRecoveryView,
    scope: RecoveryItemScope,
    activity: &ActivityView,
) -> ControlResult<()> {
    let retry_decision = activity
        .task
        .as_ref()
        .and_then(|task| task.retry_policy.as_ref())
        .zip(activity.failure.as_ref())
        .map(|(policy, failure)| policy.decide_after_failure(failure))
        .transpose()?;
    let item = FailedActivityRecoveryItem {
        scope,
        activity: activity.clone(),
        retry_decision,
    };

    if activity
        .failure
        .as_ref()
        .is_some_and(|failure| failure.retryable)
        && !matches!(
            item.retry_decision,
            Some(ActivityRetryDecision::DoNotRetry { .. })
        )
    {
        recovery.retryable_failed_activities.push(item);
    } else {
        recovery.terminal_failed_activities.push(item);
    }
    Ok(())
}

fn collect_timers(
    recovery: &mut RunRecoveryView,
    scope: &RecoveryItemScope,
    timers: &std::collections::BTreeMap<crate::TimerId, TimerView>,
) {
    for timer in timers.values() {
        if timer.status != TimerStatus::Scheduled {
            continue;
        }
        let item = TimerRecoveryItem {
            scope: scope.clone(),
            timer: timer.clone(),
        };
        if timer
            .timer
            .as_ref()
            .is_some_and(|record| record.fire_at_ms <= recovery.now_ms)
        {
            recovery.fireable_timers.push(item);
        } else {
            recovery.pending_timers.push(item);
        }
    }
}

fn collect_agent_decisions<'a>(
    recovery: &mut RunRecoveryView,
    scope: &RecoveryItemScope,
    decisions: impl Iterator<Item = &'a AgentDecision>,
) {
    recovery.pending_approval_decisions.extend(
        decisions
            .filter(|decision| decision.outcome == AgentDecisionOutcome::ApprovalRequired)
            .map(|decision| AgentDecisionRecoveryItem {
                scope: scope.clone(),
                decision: decision.clone(),
            }),
    );
}

fn collect_step_state(recovery: &mut RunRecoveryView, step: &crate::StepView) {
    if step.status == StepStatus::Waiting && step.wait_reason == Some(WaitReason::Human) {
        recovery.human_wait_steps.push(step_recovery_item(step));
    }
    if step.status == StepStatus::Blocked {
        recovery.blocked_steps.push(step_recovery_item(step));
    }
    if let Some(lease) = &step.active_lease {
        let item = LeaseRecoveryItem {
            step_id: step.step_id.clone(),
            lease: lease.clone(),
        };
        if lease.is_active_at(recovery.now_ms) {
            recovery.active_leases.push(item);
        } else {
            recovery.expired_leases.push(item);
        }
    }
}

fn step_recovery_item(step: &crate::StepView) -> StepRecoveryItem {
    StepRecoveryItem {
        step_id: step.step_id.clone(),
        status: step.status,
        wait_reason: step.wait_reason.clone(),
        last_error: step.last_error.clone(),
    }
}
