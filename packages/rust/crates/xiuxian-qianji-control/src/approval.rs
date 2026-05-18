//! Human approval signal and timer contracts.

use crate::{
    AgentProposalId, ApprovalRequestId, ApproverId, ArtifactRef, ControlError, ControlResult,
    PermissionScope, SignalName, SignalRecord, TimerId, TimerRecord, ToolPermissionDecision,
};

/// Human approval wait contract derived from an approval-required decision.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HumanApprovalRequest {
    /// Stable approval request id.
    pub approval_request_id: ApprovalRequestId,
    /// Agent proposal that requires approval.
    pub proposal_id: AgentProposalId,
    /// Signal name expected to resolve this approval request.
    pub signal_name: SignalName,
    /// Optional approval or permission scope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission_scope: Option<PermissionScope>,
    /// Optional claim-check payload reference for approval context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_ref: Option<ArtifactRef>,
    /// Optional expected signal payload hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_payload_hash: Option<String>,
    /// Optional timer id that resolves the request as timed out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_timer_id: Option<TimerId>,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl HumanApprovalRequest {
    /// Creates a human approval request.
    #[must_use]
    pub fn new(
        approval_request_id: ApprovalRequestId,
        proposal_id: AgentProposalId,
        signal_name: SignalName,
    ) -> Self {
        Self {
            approval_request_id,
            proposal_id,
            signal_name,
            permission_scope: None,
            payload_ref: None,
            expected_payload_hash: None,
            timeout_timer_id: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Creates an approval request from an approval-required tool decision.
    ///
    /// # Errors
    ///
    /// Returns a control error when the tool decision does not require human
    /// approval.
    pub fn from_tool_permission(
        approval_request_id: ApprovalRequestId,
        proposal_id: AgentProposalId,
        signal_name: SignalName,
        decision: &ToolPermissionDecision,
    ) -> ControlResult<Self> {
        let ToolPermissionDecision::ApprovalRequired {
            permission_scope, ..
        } = decision
        else {
            return Err(invalid_approval_request(
                "human approval request requires an approval-required tool decision",
            ));
        };
        Ok(Self::new(approval_request_id, proposal_id, signal_name)
            .with_optional_permission_scope(permission_scope.clone()))
    }

    /// Sets the permission scope.
    #[must_use]
    pub fn with_permission_scope(mut self, permission_scope: PermissionScope) -> Self {
        self.permission_scope = Some(permission_scope);
        self
    }

    /// Sets the approval payload claim-check reference.
    #[must_use]
    pub fn with_payload_ref(mut self, payload_ref: ArtifactRef) -> Self {
        self.payload_ref = Some(payload_ref);
        self
    }

    /// Sets the expected signal payload hash.
    #[must_use]
    pub fn with_expected_payload_hash(mut self, expected_payload_hash: impl Into<String>) -> Self {
        self.expected_payload_hash = Some(expected_payload_hash.into());
        self
    }

    /// Sets the timeout timer id.
    #[must_use]
    pub fn with_timeout_timer_id(mut self, timeout_timer_id: TimerId) -> Self {
        self.timeout_timer_id = Some(timeout_timer_id);
        self
    }

    /// Validates this approval request.
    ///
    /// # Errors
    ///
    /// Returns a control error when optional hash fields are invalid.
    pub fn validate(&self) -> ControlResult<()> {
        if self
            .expected_payload_hash
            .as_deref()
            .is_some_and(|hash| hash.trim().is_empty())
        {
            return Err(invalid_approval_request(
                "human approval expected_payload_hash must not be blank when supplied",
            ));
        }
        Ok(())
    }

    /// Matches a signal journal record against this approval request.
    ///
    /// # Errors
    ///
    /// Returns a control error when the signal name or expected payload hash
    /// does not match.
    pub fn match_signal(&self, signal: &SignalRecord) -> ControlResult<HumanApprovalResolution> {
        self.validate()?;
        if signal.signal_name != self.signal_name {
            return Err(invalid_approval_request(
                "human approval signal_name does not match request",
            ));
        }
        if let Some(expected_payload_hash) = &self.expected_payload_hash
            && signal.payload_hash.as_deref() != Some(expected_payload_hash.as_str())
        {
            return Err(invalid_approval_request(
                "human approval signal payload_hash does not match request",
            ));
        }
        Ok(HumanApprovalResolution::SignalMatched {
            approval_request_id: self.approval_request_id.clone(),
            payload_hash: signal.payload_hash.clone(),
        })
    }

    /// Matches a timer journal record against this approval request.
    ///
    /// # Errors
    ///
    /// Returns a control error when no timeout timer is declared or the timer
    /// id does not match.
    pub fn match_timer(&self, timer: &TimerRecord) -> ControlResult<HumanApprovalResolution> {
        self.validate()?;
        let Some(timeout_timer_id) = &self.timeout_timer_id else {
            return Err(invalid_approval_request(
                "human approval request has no timeout_timer_id",
            ));
        };
        if timeout_timer_id != &timer.timer_id {
            return Err(invalid_approval_request(
                "human approval timeout timer_id does not match request",
            ));
        }
        Ok(HumanApprovalResolution::TimedOut {
            approval_request_id: self.approval_request_id.clone(),
            timer_id: timer.timer_id.clone(),
        })
    }

    fn with_optional_permission_scope(mut self, permission_scope: Option<PermissionScope>) -> Self {
        self.permission_scope = permission_scope;
        self
    }
}

/// Deterministic result of matching approval wait inputs.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HumanApprovalResolution {
    /// The expected approval signal was observed.
    SignalMatched {
        /// Resolved approval request.
        approval_request_id: ApprovalRequestId,
        /// Observed payload hash, when present.
        payload_hash: Option<String>,
    },
    /// The declared timeout timer fired.
    TimedOut {
        /// Resolved approval request.
        approval_request_id: ApprovalRequestId,
        /// Matching timer id.
        timer_id: TimerId,
    },
}

/// Decision encoded by a human approval signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HumanApprovalDecisionStatus {
    /// Approval was granted.
    Approved,
    /// Approval was rejected.
    Rejected,
}

/// Deterministic approval decision parsed from a matched signal.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HumanApprovalDecision {
    /// Resolved approval request.
    pub approval_request_id: ApprovalRequestId,
    /// Approved or rejected outcome.
    pub status: HumanApprovalDecisionStatus,
    /// Optional human or policy actor id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decided_by: Option<ApproverId>,
    /// Optional human-readable reason.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Optional claim-check payload reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_ref: Option<ArtifactRef>,
    /// Optional payload hash observed on the signal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_hash: Option<String>,
    /// Original compact decision metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl HumanApprovalDecision {
    /// Creates an approval decision.
    #[must_use]
    pub fn new(
        approval_request_id: ApprovalRequestId,
        status: HumanApprovalDecisionStatus,
    ) -> Self {
        Self {
            approval_request_id,
            status,
            decided_by: None,
            reason: None,
            payload_ref: None,
            payload_hash: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Parses a typed approval decision from a matching signal record.
    ///
    /// # Errors
    ///
    /// Returns a control error when the signal does not match the request or
    /// when the compact signal metadata does not contain a valid decision.
    pub fn from_signal(
        request: &HumanApprovalRequest,
        signal: &SignalRecord,
    ) -> ControlResult<Self> {
        let HumanApprovalResolution::SignalMatched {
            approval_request_id,
            ..
        } = request.match_signal(signal)?
        else {
            return Err(invalid_approval_request(
                "human approval decision requires a matched signal",
            ));
        };
        let status =
            parse_decision_status(required_metadata_string(&signal.metadata, "decision")?)?;
        let decision = Self::new(approval_request_id, status)
            .with_optional_decided_by(optional_approver_id(&signal.metadata, "decided_by")?)
            .with_optional_reason(optional_metadata_string(&signal.metadata, "reason")?)
            .with_optional_payload_ref(signal.payload_ref.clone())
            .with_optional_payload_hash(signal.payload_hash.clone())
            .with_metadata(signal.metadata.clone());
        decision.validate()?;
        Ok(decision)
    }

    /// Sets the approver id.
    #[must_use]
    pub fn with_decided_by(mut self, decided_by: ApproverId) -> Self {
        self.decided_by = Some(decided_by);
        self
    }

    /// Sets the decision reason.
    #[must_use]
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Sets the decision payload claim-check reference.
    #[must_use]
    pub fn with_payload_ref(mut self, payload_ref: ArtifactRef) -> Self {
        self.payload_ref = Some(payload_ref);
        self
    }

    /// Sets the observed payload hash.
    #[must_use]
    pub fn with_payload_hash(mut self, payload_hash: impl Into<String>) -> Self {
        self.payload_hash = Some(payload_hash.into());
        self
    }

    /// Sets extension metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Validates this approval decision.
    ///
    /// # Errors
    ///
    /// Returns a control error when optional reason or hash fields are blank.
    pub fn validate(&self) -> ControlResult<()> {
        if self
            .reason
            .as_deref()
            .is_some_and(|reason| reason.trim().is_empty())
        {
            return Err(invalid_approval_request(
                "human approval decision reason must not be blank when supplied",
            ));
        }
        if self
            .payload_hash
            .as_deref()
            .is_some_and(|payload_hash| payload_hash.trim().is_empty())
        {
            return Err(invalid_approval_request(
                "human approval decision payload_hash must not be blank when supplied",
            ));
        }
        Ok(())
    }

    fn with_optional_decided_by(mut self, decided_by: Option<ApproverId>) -> Self {
        self.decided_by = decided_by;
        self
    }

    fn with_optional_reason(mut self, reason: Option<String>) -> Self {
        self.reason = reason;
        self
    }

    fn with_optional_payload_ref(mut self, payload_ref: Option<ArtifactRef>) -> Self {
        self.payload_ref = payload_ref;
        self
    }

    fn with_optional_payload_hash(mut self, payload_hash: Option<String>) -> Self {
        self.payload_hash = payload_hash;
        self
    }
}

fn parse_decision_status(value: &str) -> ControlResult<HumanApprovalDecisionStatus> {
    match value {
        "approved" => Ok(HumanApprovalDecisionStatus::Approved),
        "rejected" => Ok(HumanApprovalDecisionStatus::Rejected),
        _ => Err(invalid_approval_request(
            "human approval decision must be either approved or rejected",
        )),
    }
}

fn required_metadata_string<'a>(
    metadata: &'a serde_json::Value,
    field: &'static str,
) -> ControlResult<&'a str> {
    let value = metadata.get(field).ok_or_else(|| {
        invalid_approval_request("human approval signal metadata requires decision")
    })?;
    let Some(value) = value.as_str() else {
        return Err(invalid_approval_request(
            "human approval signal decision metadata must be a string",
        ));
    };
    if value.trim().is_empty() {
        return Err(invalid_approval_request(
            "human approval signal decision metadata must not be blank",
        ));
    }
    Ok(value)
}

fn optional_metadata_string(
    metadata: &serde_json::Value,
    field: &'static str,
) -> ControlResult<Option<String>> {
    let Some(value) = metadata.get(field) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let Some(value) = value.as_str() else {
        return Err(invalid_approval_request(match field {
            "reason" => "human approval signal reason metadata must be a string",
            "decided_by" => "human approval signal decided_by metadata must be a string",
            _ => "human approval signal metadata field must be a string",
        }));
    };
    if value.trim().is_empty() {
        return Err(invalid_approval_request(match field {
            "reason" => "human approval signal reason metadata must not be blank",
            "decided_by" => "human approval signal decided_by metadata must not be blank",
            _ => "human approval signal metadata field must not be blank",
        }));
    }
    Ok(Some(value.to_owned()))
}

fn optional_approver_id(
    metadata: &serde_json::Value,
    field: &'static str,
) -> ControlResult<Option<ApproverId>> {
    optional_metadata_string(metadata, field)?
        .map(ApproverId::new)
        .transpose()
}

fn invalid_approval_request(message: &str) -> ControlError {
    ControlError::InvalidEventSequence {
        message: message.to_owned(),
    }
}
