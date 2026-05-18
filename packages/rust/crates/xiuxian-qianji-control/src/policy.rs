//! Deterministic Agent policy reducers.

use crate::{
    ActivityTask, AgentDecision, AgentDecisionId, AgentDecisionOutcome, AgentProposal,
    ControlError, ControlResult, DecisionReasonCode, GateResult, ToolActivityAdmission,
    ToolAuthorizationDecision,
};

/// Stable reason codes emitted by the built-in Agent policy reducer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentPolicyReason {
    /// A deterministic gate failed before any work could be scheduled.
    GateFailed,
    /// The tool authorization allowed scheduling.
    ToolAuthorized,
    /// The tool requires human approval before scheduling.
    ToolApprovalRequired,
    /// Human approval rejected the tool.
    ToolApprovalRejected,
    /// Registry policy denied the tool.
    ToolDenied,
    /// Human approval timed out.
    ToolApprovalTimedOut,
}

impl AgentPolicyReason {
    /// Returns the stable serialized reason code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GateFailed => "gate_failed",
            Self::ToolAuthorized => "tool_authorized",
            Self::ToolApprovalRequired => "tool_approval_required",
            Self::ToolApprovalRejected => "tool_approval_rejected",
            Self::ToolDenied => "tool_denied",
            Self::ToolApprovalTimedOut => "tool_approval_timed_out",
        }
    }

    fn into_reason_code(self) -> ControlResult<DecisionReasonCode> {
        DecisionReasonCode::new(self.as_str())
    }
}

/// Tool-policy input to the deterministic Agent reducer.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ToolPolicyReductionRequest {
    /// Qianji-assigned decision id for the reduction output.
    pub decision_id: AgentDecisionId,
    /// LLM-authored proposal being reduced.
    pub proposal: AgentProposal,
    /// Deterministic authorization fact for the proposal's tool.
    pub authorization: ToolAuthorizationDecision,
    /// Candidate activity task. Required only when authorization is
    /// `Authorized`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<ActivityTask>,
    /// Optional gate result that must pass before scheduling work.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate_result: Option<GateResult>,
    /// Optional checkpoint sequence produced by the caller's reducer boundary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint_seq: Option<u64>,
}

impl ToolPolicyReductionRequest {
    /// Creates a tool-policy reduction request.
    #[must_use]
    pub const fn new(
        decision_id: AgentDecisionId,
        proposal: AgentProposal,
        authorization: ToolAuthorizationDecision,
    ) -> Self {
        Self {
            decision_id,
            proposal,
            authorization,
            task: None,
            gate_result: None,
            checkpoint_seq: None,
        }
    }

    /// Sets the candidate activity task.
    #[must_use]
    pub fn with_task(mut self, task: ActivityTask) -> Self {
        self.task = Some(task);
        self
    }

    /// Sets the gate result used by the reduction.
    #[must_use]
    pub fn with_gate_result(mut self, gate_result: GateResult) -> Self {
        self.gate_result = Some(gate_result);
        self
    }

    /// Sets the checkpoint sequence used by the reduction.
    #[must_use]
    pub const fn with_checkpoint_seq(mut self, checkpoint_seq: u64) -> Self {
        self.checkpoint_seq = Some(checkpoint_seq);
        self
    }

    /// Reduces the request into a deterministic Agent decision.
    ///
    /// # Errors
    ///
    /// Returns a control error when the proposal is invalid, the candidate
    /// task does not match the authorization, the request tries to attach an
    /// activity to a non-authorized outcome, or the produced decision is
    /// internally inconsistent.
    pub fn reduce(self) -> ControlResult<ToolPolicyReduction> {
        self.proposal.validate()?;

        if let Some(gate_result) = self.gate_result.as_ref()
            && !gate_result.passed
        {
            let decision = self.decision(
                AgentDecisionOutcome::Rejected,
                AgentPolicyReason::GateFailed,
            )?;
            return Ok(ToolPolicyReduction {
                decision,
                admission: None,
            });
        }

        match &self.authorization {
            ToolAuthorizationDecision::Authorized { .. } => self.reduce_authorized(),
            ToolAuthorizationDecision::WaitingApproval { .. } => {
                self.reject_task_for_non_authorized()?;
                let decision = self.decision(
                    AgentDecisionOutcome::ApprovalRequired,
                    AgentPolicyReason::ToolApprovalRequired,
                )?;
                Ok(ToolPolicyReduction {
                    decision,
                    admission: None,
                })
            }
            ToolAuthorizationDecision::Rejected { .. } => {
                self.reject_task_for_non_authorized()?;
                let decision = self.decision(
                    AgentDecisionOutcome::Rejected,
                    AgentPolicyReason::ToolApprovalRejected,
                )?;
                Ok(ToolPolicyReduction {
                    decision,
                    admission: None,
                })
            }
            ToolAuthorizationDecision::Denied { .. } => {
                self.reject_task_for_non_authorized()?;
                let decision = self.decision(
                    AgentDecisionOutcome::Rejected,
                    AgentPolicyReason::ToolDenied,
                )?;
                Ok(ToolPolicyReduction {
                    decision,
                    admission: None,
                })
            }
            ToolAuthorizationDecision::TimedOut { .. } => {
                self.reject_task_for_non_authorized()?;
                let decision = self.decision(
                    AgentDecisionOutcome::Rejected,
                    AgentPolicyReason::ToolApprovalTimedOut,
                )?;
                Ok(ToolPolicyReduction {
                    decision,
                    admission: None,
                })
            }
        }
    }

    fn reduce_authorized(self) -> ControlResult<ToolPolicyReduction> {
        let Some(task) = self.task.clone() else {
            return Err(invalid_policy_reduction(
                "authorized tool policy reduction requires an activity task",
            ));
        };
        let admission =
            ToolActivityAdmission::from_authorized_tool(&self.proposal, &self.authorization, task)?;
        let mut decision = admission.to_accepted_agent_decision(
            self.decision_id.clone(),
            AgentPolicyReason::ToolAuthorized.into_reason_code()?,
        )?;
        decision = self.attach_optional_decision_fields(decision);
        decision.validate()?;

        Ok(ToolPolicyReduction {
            decision,
            admission: Some(admission),
        })
    }

    fn reject_task_for_non_authorized(&self) -> ControlResult<()> {
        if self.task.is_some() {
            return Err(invalid_policy_reduction(
                "non-authorized tool policy reduction cannot carry an activity task",
            ));
        }
        Ok(())
    }

    fn decision(
        &self,
        outcome: AgentDecisionOutcome,
        reason: AgentPolicyReason,
    ) -> ControlResult<AgentDecision> {
        let decision = AgentDecision::new(
            self.decision_id.clone(),
            self.proposal.proposal_id.clone(),
            outcome,
            reason.into_reason_code()?,
        );
        let decision = self.attach_optional_decision_fields(decision);
        decision.validate()?;
        Ok(decision)
    }

    fn attach_optional_decision_fields(&self, mut decision: AgentDecision) -> AgentDecision {
        if let Some(gate_result) = self.gate_result.clone() {
            decision = decision.with_gate_result(gate_result);
        }
        if let Some(checkpoint_seq) = self.checkpoint_seq {
            decision = decision.with_checkpoint_seq(checkpoint_seq);
        }
        decision
    }
}

/// Output of one deterministic tool-policy reduction.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ToolPolicyReduction {
    /// Deterministic Agent decision produced by the reducer.
    pub decision: AgentDecision,
    /// Admitted tool activity, present only for accepted decisions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admission: Option<ToolActivityAdmission>,
}

impl ToolPolicyReduction {
    /// Validates the reduction output.
    ///
    /// # Errors
    ///
    /// Returns a control error when the decision or admission pairing is
    /// inconsistent.
    pub fn validate(&self) -> ControlResult<()> {
        self.decision.validate()?;
        match (self.decision.outcome, self.admission.is_some()) {
            (AgentDecisionOutcome::Accepted, false) => Err(invalid_policy_reduction(
                "accepted policy reduction requires an admitted activity",
            )),
            (AgentDecisionOutcome::Rejected | AgentDecisionOutcome::ApprovalRequired, true) => {
                Err(invalid_policy_reduction(
                    "non-accepted policy reduction cannot carry an admitted activity",
                ))
            }
            _ => Ok(()),
        }
    }
}

fn invalid_policy_reduction(message: &str) -> ControlError {
    ControlError::InvalidEventSequence {
        message: message.to_owned(),
    }
}
