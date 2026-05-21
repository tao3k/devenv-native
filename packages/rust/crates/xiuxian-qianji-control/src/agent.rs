//! Agent proposal and deterministic decision contracts.

use crate::{
    ActivityId, AgentDecisionId, AgentProposalId, ArtifactRef, ControlError, ControlResult,
    DecisionReasonCode, GateResult, StepId, TokenId,
};

/// LLM-authored proposal waiting for deterministic control reduction.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AgentProposal {
    /// Stable proposal id.
    pub proposal_id: AgentProposalId,
    /// Owning workflow step.
    pub step_id: StepId,
    /// Active token that produced this proposal.
    pub token_id: TokenId,
    /// Proposed action name.
    pub proposed_action: String,
    /// Optional tool name proposed by the agent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    /// Optional claim-check input reference for the proposed tool or activity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_input_ref: Option<ArtifactRef>,
    /// Optional confidence encoded in thousandths.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence_millis: Option<u32>,
    /// Optional claim-check rationale reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale_ref: Option<ArtifactRef>,
    /// Optional hash of the expected output schema.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_schema_hash: Option<String>,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl AgentProposal {
    /// Creates an agent proposal with deterministic defaults.
    #[must_use]
    pub fn new(
        proposal_id: AgentProposalId,
        step_id: StepId,
        token_id: TokenId,
        proposed_action: impl Into<String>,
    ) -> Self {
        Self {
            proposal_id,
            step_id,
            token_id,
            proposed_action: proposed_action.into(),
            tool_name: None,
            tool_input_ref: None,
            confidence_millis: None,
            rationale_ref: None,
            output_schema_hash: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Sets the proposed tool name.
    #[must_use]
    pub fn with_tool_name(mut self, tool_name: impl Into<String>) -> Self {
        self.tool_name = Some(tool_name.into());
        self
    }

    /// Sets the proposed tool input reference.
    #[must_use]
    pub fn with_tool_input_ref(mut self, tool_input_ref: ArtifactRef) -> Self {
        self.tool_input_ref = Some(tool_input_ref);
        self
    }

    /// Sets the proposal confidence in thousandths.
    #[must_use]
    pub const fn with_confidence_millis(mut self, confidence_millis: u32) -> Self {
        self.confidence_millis = Some(confidence_millis);
        self
    }

    /// Sets the rationale reference.
    #[must_use]
    pub fn with_rationale_ref(mut self, rationale_ref: ArtifactRef) -> Self {
        self.rationale_ref = Some(rationale_ref);
        self
    }

    /// Sets the output schema hash.
    #[must_use]
    pub fn with_output_schema_hash(mut self, output_schema_hash: impl Into<String>) -> Self {
        self.output_schema_hash = Some(output_schema_hash.into());
        self
    }

    /// Validates the proposal payload.
    ///
    /// # Errors
    ///
    /// Returns a control error when proposal action, tool name, confidence, or
    /// schema hash fields are invalid.
    pub fn validate(&self) -> ControlResult<()> {
        if self.proposed_action.trim().is_empty() {
            return Err(invalid_agent_contract(
                "agent proposal requires non-blank proposed_action",
            ));
        }
        if self
            .tool_name
            .as_ref()
            .is_some_and(|tool_name| tool_name.trim().is_empty())
        {
            return Err(invalid_agent_contract(
                "agent proposal tool_name must not be blank when supplied",
            ));
        }
        if self
            .confidence_millis
            .is_some_and(|confidence_millis| confidence_millis > 1_000)
        {
            return Err(invalid_agent_contract(
                "agent proposal confidence_millis cannot exceed 1000",
            ));
        }
        if self
            .output_schema_hash
            .as_ref()
            .is_some_and(|hash| hash.trim().is_empty())
        {
            return Err(invalid_agent_contract(
                "agent proposal output_schema_hash must not be blank when supplied",
            ));
        }
        Ok(())
    }
}

/// Deterministic reducer outcome for one agent proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentDecisionOutcome {
    /// Proposal was accepted and activity work may be scheduled.
    Accepted,
    /// Proposal was rejected.
    Rejected,
    /// Proposal requires human approval before scheduling work.
    ApprovalRequired,
}

/// Qianji-authored decision over an agent proposal.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AgentDecision {
    /// Stable decision id.
    pub decision_id: AgentDecisionId,
    /// Proposal being reduced.
    pub proposal_id: AgentProposalId,
    /// Deterministic outcome.
    pub outcome: AgentDecisionOutcome,
    /// Stable reason code.
    pub reason_code: DecisionReasonCode,
    /// Activity scheduled by an accepted decision.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheduled_activity_id: Option<ActivityId>,
    /// Optional gate result used by the reducer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate_result: Option<GateResult>,
    /// Optional checkpoint sequence produced by the reducer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint_seq: Option<u64>,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl AgentDecision {
    /// Creates an agent decision with deterministic defaults.
    #[must_use]
    pub fn new(
        decision_id: AgentDecisionId,
        proposal_id: AgentProposalId,
        outcome: AgentDecisionOutcome,
        reason_code: DecisionReasonCode,
    ) -> Self {
        Self {
            decision_id,
            proposal_id,
            outcome,
            reason_code,
            scheduled_activity_id: None,
            gate_result: None,
            checkpoint_seq: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Sets the scheduled activity id.
    #[must_use]
    pub fn with_scheduled_activity_id(mut self, scheduled_activity_id: ActivityId) -> Self {
        self.scheduled_activity_id = Some(scheduled_activity_id);
        self
    }

    /// Sets the gate result used by the decision.
    #[must_use]
    pub fn with_gate_result(mut self, gate_result: GateResult) -> Self {
        self.gate_result = Some(gate_result);
        self
    }

    /// Sets the checkpoint sequence.
    #[must_use]
    pub const fn with_checkpoint_seq(mut self, checkpoint_seq: u64) -> Self {
        self.checkpoint_seq = Some(checkpoint_seq);
        self
    }

    /// Validates the deterministic decision payload.
    ///
    /// # Errors
    ///
    /// Returns a control error when outcome/activity/checkpoint fields are
    /// inconsistent.
    pub fn validate(&self) -> ControlResult<()> {
        match (self.outcome, self.scheduled_activity_id.is_some()) {
            (AgentDecisionOutcome::Accepted, false) => {
                return Err(invalid_agent_contract(
                    "accepted agent decision requires scheduled_activity_id",
                ));
            }
            (AgentDecisionOutcome::Rejected | AgentDecisionOutcome::ApprovalRequired, true) => {
                return Err(invalid_agent_contract(
                    "non-accepted agent decision cannot carry scheduled_activity_id",
                ));
            }
            _ => {}
        }

        if matches!(self.checkpoint_seq, Some(0)) {
            return Err(invalid_agent_contract(
                "agent decision checkpoint_seq must be non-zero when supplied",
            ));
        }

        Ok(())
    }
}

fn invalid_agent_contract(message: &str) -> ControlError {
    ControlError::InvalidEventSequence {
        message: message.to_owned(),
    }
}
