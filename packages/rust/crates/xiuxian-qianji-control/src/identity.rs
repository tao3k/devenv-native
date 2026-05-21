//! Stable control-plane identity newtypes.

macro_rules! control_id {
    ($name:ident, $field:literal) => {
        #[doc = concat!("Typed control-plane value for `", $field, "`.")]
        #[derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            serde::Serialize,
            serde::Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Creates a new non-empty id.
            ///
            /// # Errors
            ///
            /// Returns [`crate::ControlError::BlankId`] when the id is empty.
            pub fn new(value: impl Into<String>) -> crate::ControlResult<Self> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(crate::ControlError::BlankId { field: $field });
                }
                Ok(Self(value))
            }

            /// Borrows the serialized id.
            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl TryFrom<String> for $name {
            type Error = crate::ControlError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = crate::ControlError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }
    };
}

control_id!(RunId, "run_id");
control_id!(StepId, "step_id");
control_id!(ActivityId, "activity_id");
control_id!(ActivityType, "activity_type");
control_id!(TaskQueue, "task_queue");
control_id!(IdempotencyKey, "idempotency_key");
control_id!(ErrorCode, "error_code");
control_id!(LlmModelId, "llm_model_id");
control_id!(ApprovalRequestId, "approval_request_id");
control_id!(ApproverId, "approver_id");
control_id!(AgentProposalId, "agent_proposal_id");
control_id!(AgentDecisionId, "agent_decision_id");
control_id!(DecisionReasonCode, "decision_reason_code");
control_id!(TokenId, "token_id");
control_id!(ToolName, "tool_name");
control_id!(PermissionScope, "permission_scope");
control_id!(SignalName, "signal_name");
control_id!(TimerId, "timer_id");
control_id!(VersionKey, "version_key");
control_id!(WorkerId, "worker_id");
control_id!(LeaseId, "lease_id");
control_id!(EvidenceId, "evidence_id");
control_id!(ArtifactId, "artifact_id");
control_id!(ArtifactKind, "artifact_kind");
control_id!(GateName, "gate_name");
