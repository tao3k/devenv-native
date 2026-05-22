//! Activity admission contracts.

use crate::{
    ActivityTask, AgentDecision, AgentDecisionId, AgentDecisionOutcome, AgentProposal,
    AgentProposalId, ApprovalRequestId, ControlError, ControlResult, DecisionReasonCode,
    LlmActivityTask, ToolAuthorizationDecision, ToolName,
};

/// Admitted LLM activity task before queueing or provider execution.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LlmActivityAdmission {
    /// LLM activity payload admitted by the deterministic controller.
    pub activity: LlmActivityTask,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl LlmActivityAdmission {
    /// Validates and records an LLM activity admission.
    ///
    /// # Errors
    ///
    /// Returns a control error when the LLM activity is invalid, when the
    /// generic activity input reference is missing, or when it does not match
    /// the request prompt reference. Provider adapters must not execute LLM
    /// calls without this claim-check binding.
    pub fn from_activity(activity: LlmActivityTask) -> ControlResult<Self> {
        activity.validate()?;
        if activity.task.input_ref.as_ref() != Some(&activity.request.prompt_ref) {
            return Err(invalid_admission_contract(
                "llm activity admission task input_ref must match request prompt_ref",
            ));
        }

        Ok(Self {
            activity,
            metadata: serde_json::Value::Null,
        })
    }

    /// Sets extension metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Validates this admission contract.
    ///
    /// # Errors
    ///
    /// Returns a control error when the admitted LLM activity is invalid or no
    /// longer preserves the prompt claim-check binding.
    pub fn validate(&self) -> ControlResult<()> {
        Self::from_activity(self.activity.clone()).map(|_| ())
    }

    /// Returns the validated generic activity task for schedule recording.
    #[must_use]
    pub const fn activity_task(&self) -> &ActivityTask {
        &self.activity.task
    }
}

/// Admitted tool activity task before queueing or execution.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ToolActivityAdmission {
    /// Agent proposal being admitted.
    pub proposal_id: AgentProposalId,
    /// Agent-visible tool name.
    pub tool_name: ToolName,
    /// Activity task that a later scheduler may enqueue.
    pub task: ActivityTask,
    /// Approval request that authorized this task, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_request_id: Option<ApprovalRequestId>,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl ToolActivityAdmission {
    /// Validates and records an authorized tool activity admission.
    ///
    /// # Errors
    ///
    /// Returns a control error when the proposal is invalid, the
    /// authorization is not authorized, or the task does not match the
    /// authorized activity type, queue, and proposal input reference.
    pub fn from_authorized_tool(
        proposal: &AgentProposal,
        authorization: &ToolAuthorizationDecision,
        task: ActivityTask,
    ) -> ControlResult<Self> {
        proposal.validate()?;
        task.validate()?;
        let Some(tool_name) = proposal.tool_name.as_deref() else {
            return Err(invalid_admission_contract(
                "tool activity admission requires proposal tool_name",
            ));
        };
        let ToolAuthorizationDecision::Authorized {
            activity_type,
            task_queue,
            approval_request_id,
        } = authorization
        else {
            return Err(invalid_admission_contract(
                "tool activity admission requires authorized tool decision",
            ));
        };
        if &task.activity_type != activity_type {
            return Err(invalid_admission_contract(
                "tool activity admission task activity_type does not match authorization",
            ));
        }
        if &task.task_queue != task_queue {
            return Err(invalid_admission_contract(
                "tool activity admission task task_queue does not match authorization",
            ));
        }
        if task.input_ref != proposal.tool_input_ref {
            return Err(invalid_admission_contract(
                "tool activity admission task input_ref does not match proposal tool_input_ref",
            ));
        }

        Ok(Self {
            proposal_id: proposal.proposal_id.clone(),
            tool_name: ToolName::new(tool_name.to_owned())?,
            task,
            approval_request_id: approval_request_id.clone(),
            metadata: serde_json::Value::Null,
        })
    }

    /// Sets extension metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Validates this admission contract.
    ///
    /// # Errors
    ///
    /// Returns a control error when the admitted task is invalid.
    pub fn validate(&self) -> ControlResult<()> {
        self.task.validate()
    }

    /// Creates an accepted Agent decision for this admitted activity.
    ///
    /// # Errors
    ///
    /// Returns a control error when the admission or produced decision is
    /// invalid.
    pub fn to_accepted_agent_decision(
        &self,
        decision_id: AgentDecisionId,
        reason_code: DecisionReasonCode,
    ) -> ControlResult<AgentDecision> {
        self.validate()?;
        let decision = AgentDecision::new(
            decision_id,
            self.proposal_id.clone(),
            AgentDecisionOutcome::Accepted,
            reason_code,
        )
        .with_scheduled_activity_id(self.task.activity_id.clone());
        decision.validate()?;
        Ok(decision)
    }
}

fn invalid_admission_contract(message: &str) -> ControlError {
    ControlError::InvalidEventSequence {
        message: message.to_owned(),
    }
}
