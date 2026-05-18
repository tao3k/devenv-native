//! Side-effect-free recovery plan projection.

use crate::{
    ActivityId, ActivityRetryDecision, AgentDecisionId, LeaseId, RecoveryItemScope, RunId,
    RunRecoveryView, StepId, TimerId,
};

/// Deterministic recovery plan derived from replayed recovery facts.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RunRecoveryPlan {
    /// Run id.
    pub run_id: RunId,
    /// Observation time inherited from the recovery view.
    pub planned_at_ms: u64,
    /// Ordered recovery actions.
    #[serde(default)]
    pub actions: Vec<RecoveryPlanAction>,
}

impl RunRecoveryPlan {
    fn new(run_id: RunId, planned_at_ms: u64) -> Self {
        Self {
            run_id,
            planned_at_ms,
            actions: Vec::new(),
        }
    }

    /// Returns true when the plan contains at least one recovery action.
    #[must_use]
    pub fn has_actions(&self) -> bool {
        !self.actions.is_empty()
    }

    /// Returns compact operational counters over this recovery plan.
    #[must_use]
    pub fn summary(&self) -> RunRecoveryPlanSummary {
        RunRecoveryPlanSummary::from_actions(&self.actions)
    }
}

/// Compact management counters over a deterministic recovery plan.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct RunRecoveryPlanSummary {
    /// Total number of ordered actions in the plan.
    pub total_actions: usize,
    /// Expired leases that should be reclaimed.
    pub reclaim_expired_leases: usize,
    /// Timers ready to fire.
    pub fireable_timers: usize,
    /// Failed activities with a concrete retry decision.
    pub retry_activities: usize,
    /// Retryable failed activities that need policy review.
    pub review_retryable_activities: usize,
    /// Terminal failed activities that should escalate.
    pub terminal_activity_escalations: usize,
    /// Scheduled activities that should be reconciled after restart.
    pub reconcile_scheduled_activities: usize,
    /// In-flight activities that should be inspected after restart.
    pub inspect_in_flight_activities: usize,
    /// Agent decisions waiting for human approval.
    pub await_human_approvals: usize,
    /// Steps waiting for human input.
    pub await_human_inputs: usize,
    /// Blocked steps that need inspection.
    pub inspect_blocked_steps: usize,
    /// Active leases that should be preserved.
    pub preserve_active_leases: usize,
    /// Timers that are still pending.
    pub await_timers: usize,
}

impl RunRecoveryPlanSummary {
    fn from_actions(actions: &[RecoveryPlanAction]) -> Self {
        let mut summary = Self {
            total_actions: actions.len(),
            ..Self::default()
        };

        for action in actions {
            summary.record_action(action);
        }

        summary
    }

    fn record_action(&mut self, action: &RecoveryPlanAction) {
        match action {
            RecoveryPlanAction::ReclaimExpiredLease { .. } => {
                self.reclaim_expired_leases += 1;
            }
            RecoveryPlanAction::FireTimer { .. } => {
                self.fireable_timers += 1;
            }
            RecoveryPlanAction::RetryActivity { .. } => {
                self.retry_activities += 1;
            }
            RecoveryPlanAction::ReviewRetryableActivity { .. } => {
                self.review_retryable_activities += 1;
            }
            RecoveryPlanAction::EscalateTerminalActivity { .. } => {
                self.terminal_activity_escalations += 1;
            }
            RecoveryPlanAction::ReconcileScheduledActivity { .. } => {
                self.reconcile_scheduled_activities += 1;
            }
            RecoveryPlanAction::InspectInFlightActivity { .. } => {
                self.inspect_in_flight_activities += 1;
            }
            RecoveryPlanAction::AwaitHumanApproval { .. } => {
                self.await_human_approvals += 1;
            }
            RecoveryPlanAction::AwaitHumanInput { .. } => {
                self.await_human_inputs += 1;
            }
            RecoveryPlanAction::InspectBlockedStep { .. } => {
                self.inspect_blocked_steps += 1;
            }
            RecoveryPlanAction::PreserveActiveLease { .. } => {
                self.preserve_active_leases += 1;
            }
            RecoveryPlanAction::AwaitTimer { .. } => {
                self.await_timers += 1;
            }
        }
    }
}

/// One deterministic recovery action.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum RecoveryPlanAction {
    /// Reclaim an expired step lease before scheduling more work.
    ReclaimExpiredLease {
        /// Step whose lease expired.
        step_id: StepId,
        /// Expired lease id.
        lease_id: LeaseId,
    },
    /// Fire a timer whose scheduled fire time has elapsed.
    FireTimer {
        /// Run or step scope.
        scope: RecoveryItemScope,
        /// Timer id.
        timer_id: TimerId,
        /// Fire timestamp from the timer record when known.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fire_at_ms: Option<u64>,
    },
    /// Schedule the next retry attempt for a failed activity.
    RetryActivity {
        /// Run or step scope.
        scope: RecoveryItemScope,
        /// Activity id.
        activity_id: ActivityId,
        /// Deterministic retry decision.
        retry_decision: ActivityRetryDecision,
    },
    /// A failed activity was marked retryable but lacks a concrete retry
    /// decision.
    ReviewRetryableActivity {
        /// Run or step scope.
        scope: RecoveryItemScope,
        /// Activity id.
        activity_id: ActivityId,
    },
    /// Escalate a terminal failed activity to policy or human recovery.
    EscalateTerminalActivity {
        /// Run or step scope.
        scope: RecoveryItemScope,
        /// Activity id.
        activity_id: ActivityId,
        /// Stop decision when a retry policy supplied one.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        retry_decision: Option<ActivityRetryDecision>,
    },
    /// Reconcile scheduled activity state after restart.
    ReconcileScheduledActivity {
        /// Run or step scope.
        scope: RecoveryItemScope,
        /// Activity id.
        activity_id: ActivityId,
    },
    /// Inspect an activity that was in flight when the view was built.
    InspectInFlightActivity {
        /// Run or step scope.
        scope: RecoveryItemScope,
        /// Activity id.
        activity_id: ActivityId,
    },
    /// Wait for a human decision over an approval-required Agent decision.
    AwaitHumanApproval {
        /// Run or step scope.
        scope: RecoveryItemScope,
        /// Decision id.
        decision_id: AgentDecisionId,
    },
    /// Continue waiting for human input at a step boundary.
    AwaitHumanInput {
        /// Waiting step id.
        step_id: StepId,
    },
    /// Inspect a blocked step before attempting recovery.
    InspectBlockedStep {
        /// Blocked step id.
        step_id: StepId,
        /// Last block or error reason when known.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        last_error: Option<String>,
    },
    /// Preserve an active lease owned by a still-valid worker claim.
    PreserveActiveLease {
        /// Leased step id.
        step_id: StepId,
        /// Active lease id.
        lease_id: LeaseId,
    },
    /// Continue waiting for a timer that has not reached its fire time.
    AwaitTimer {
        /// Run or step scope.
        scope: RecoveryItemScope,
        /// Timer id.
        timer_id: TimerId,
        /// Fire timestamp from the timer record when known.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fire_at_ms: Option<u64>,
    },
}

impl RunRecoveryView {
    /// Projects this recovery view into a deterministic read-only recovery
    /// plan.
    ///
    /// The plan only describes what a later runtime loop should inspect or
    /// perform. It does not append events, enqueue work, mutate leases, fire
    /// timers, or execute retries.
    #[must_use]
    pub fn recovery_plan(&self) -> RunRecoveryPlan {
        let mut plan = RunRecoveryPlan::new(self.run_id.clone(), self.now_ms);

        project_expired_leases(self, &mut plan);
        project_fireable_timers(self, &mut plan);
        project_retryable_failures(self, &mut plan);
        project_terminal_failures(self, &mut plan);
        project_scheduled_activities(self, &mut plan);
        project_in_flight_activities(self, &mut plan);
        project_human_approval(self, &mut plan);
        project_step_waits(self, &mut plan);
        project_active_leases(self, &mut plan);
        project_pending_timers(self, &mut plan);

        plan
    }
}

fn project_expired_leases(recovery: &RunRecoveryView, plan: &mut RunRecoveryPlan) {
    for lease in &recovery.expired_leases {
        plan.actions.push(RecoveryPlanAction::ReclaimExpiredLease {
            step_id: lease.step_id.clone(),
            lease_id: lease.lease.lease_id.clone(),
        });
    }
}

fn project_fireable_timers(recovery: &RunRecoveryView, plan: &mut RunRecoveryPlan) {
    for timer in &recovery.fireable_timers {
        plan.actions.push(RecoveryPlanAction::FireTimer {
            scope: timer.scope.clone(),
            timer_id: timer.timer.timer_id.clone(),
            fire_at_ms: timer.timer.timer.as_ref().map(|record| record.fire_at_ms),
        });
    }
}

fn project_retryable_failures(recovery: &RunRecoveryView, plan: &mut RunRecoveryPlan) {
    for activity in &recovery.retryable_failed_activities {
        if let Some(retry_decision @ ActivityRetryDecision::Retry { .. }) = &activity.retry_decision
        {
            plan.actions.push(RecoveryPlanAction::RetryActivity {
                scope: activity.scope.clone(),
                activity_id: activity.activity.activity_id.clone(),
                retry_decision: retry_decision.clone(),
            });
        } else {
            plan.actions
                .push(RecoveryPlanAction::ReviewRetryableActivity {
                    scope: activity.scope.clone(),
                    activity_id: activity.activity.activity_id.clone(),
                });
        }
    }
}

fn project_terminal_failures(recovery: &RunRecoveryView, plan: &mut RunRecoveryPlan) {
    for activity in &recovery.terminal_failed_activities {
        plan.actions
            .push(RecoveryPlanAction::EscalateTerminalActivity {
                scope: activity.scope.clone(),
                activity_id: activity.activity.activity_id.clone(),
                retry_decision: activity.retry_decision.clone(),
            });
    }
}

fn project_scheduled_activities(recovery: &RunRecoveryView, plan: &mut RunRecoveryPlan) {
    for activity in &recovery.scheduled_activities {
        plan.actions
            .push(RecoveryPlanAction::ReconcileScheduledActivity {
                scope: activity.scope.clone(),
                activity_id: activity.activity.activity_id.clone(),
            });
    }
}

fn project_in_flight_activities(recovery: &RunRecoveryView, plan: &mut RunRecoveryPlan) {
    for activity in &recovery.in_flight_activities {
        plan.actions
            .push(RecoveryPlanAction::InspectInFlightActivity {
                scope: activity.scope.clone(),
                activity_id: activity.activity.activity_id.clone(),
            });
    }
}

fn project_human_approval(recovery: &RunRecoveryView, plan: &mut RunRecoveryPlan) {
    for decision in &recovery.pending_approval_decisions {
        plan.actions.push(RecoveryPlanAction::AwaitHumanApproval {
            scope: decision.scope.clone(),
            decision_id: decision.decision.decision_id.clone(),
        });
    }
}

fn project_step_waits(recovery: &RunRecoveryView, plan: &mut RunRecoveryPlan) {
    for step in &recovery.human_wait_steps {
        plan.actions.push(RecoveryPlanAction::AwaitHumanInput {
            step_id: step.step_id.clone(),
        });
    }
    for step in &recovery.blocked_steps {
        plan.actions.push(RecoveryPlanAction::InspectBlockedStep {
            step_id: step.step_id.clone(),
            last_error: step.last_error.clone(),
        });
    }
}

fn project_active_leases(recovery: &RunRecoveryView, plan: &mut RunRecoveryPlan) {
    for lease in &recovery.active_leases {
        plan.actions.push(RecoveryPlanAction::PreserveActiveLease {
            step_id: lease.step_id.clone(),
            lease_id: lease.lease.lease_id.clone(),
        });
    }
}

fn project_pending_timers(recovery: &RunRecoveryView, plan: &mut RunRecoveryPlan) {
    for timer in &recovery.pending_timers {
        plan.actions.push(RecoveryPlanAction::AwaitTimer {
            scope: timer.scope.clone(),
            timer_id: timer.timer.timer_id.clone(),
            fire_at_ms: timer.timer.timer.as_ref().map(|record| record.fire_at_ms),
        });
    }
}
