//! Tool activity registry and permission contracts.

use crate::{
    ActivityType, AgentProposal, ApprovalRequestId, ControlError, ControlResult,
    HumanApprovalDecision, HumanApprovalDecisionStatus, HumanApprovalResolution, PermissionScope,
    TaskQueue, ToolName,
};

/// Risk level declared for a tool activity contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolRiskLevel {
    /// Low-risk tool that can usually run automatically.
    Low,
    /// Medium-risk tool that may require policy-specific gates.
    Medium,
    /// High-risk tool that normally requires human or policy approval.
    High,
    /// Critical tool that should be denied or explicitly approved.
    Critical,
}

/// Permission mode for a tool activity contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolPermissionMode {
    /// Tool may run automatically after deterministic validation.
    Automatic,
    /// Tool requires approval before any activity is scheduled.
    HumanApprovalRequired,
    /// Tool is currently denied.
    Denied,
}

/// Registry entry mapping an agent-visible tool to an activity boundary.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ToolActivityContract {
    /// Agent-visible tool name.
    pub tool_name: ToolName,
    /// Activity type to schedule when allowed.
    pub activity_type: ActivityType,
    /// Task queue to use when allowed.
    pub task_queue: TaskQueue,
    /// Declared risk level.
    pub risk_level: ToolRiskLevel,
    /// Declared permission mode.
    pub permission_mode: ToolPermissionMode,
    /// Optional permission scope or approval lane.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission_scope: Option<PermissionScope>,
    /// Optional expected input schema hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_schema_hash: Option<String>,
    /// Optional expected output schema hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_schema_hash: Option<String>,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl ToolActivityContract {
    /// Creates a tool activity registry entry.
    #[must_use]
    pub fn new(tool_name: ToolName, activity_type: ActivityType, task_queue: TaskQueue) -> Self {
        Self {
            tool_name,
            activity_type,
            task_queue,
            risk_level: ToolRiskLevel::Low,
            permission_mode: ToolPermissionMode::Automatic,
            permission_scope: None,
            input_schema_hash: None,
            output_schema_hash: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Sets the declared risk level.
    #[must_use]
    pub const fn with_risk_level(mut self, risk_level: ToolRiskLevel) -> Self {
        self.risk_level = risk_level;
        self
    }

    /// Sets the permission mode.
    #[must_use]
    pub const fn with_permission_mode(mut self, permission_mode: ToolPermissionMode) -> Self {
        self.permission_mode = permission_mode;
        self
    }

    /// Sets the permission scope.
    #[must_use]
    pub fn with_permission_scope(mut self, permission_scope: PermissionScope) -> Self {
        self.permission_scope = Some(permission_scope);
        self
    }

    /// Sets the expected input schema hash.
    #[must_use]
    pub fn with_input_schema_hash(mut self, input_schema_hash: impl Into<String>) -> Self {
        self.input_schema_hash = Some(input_schema_hash.into());
        self
    }

    /// Sets the expected output schema hash.
    #[must_use]
    pub fn with_output_schema_hash(mut self, output_schema_hash: impl Into<String>) -> Self {
        self.output_schema_hash = Some(output_schema_hash.into());
        self
    }

    /// Validates this registry entry.
    ///
    /// # Errors
    ///
    /// Returns a control error when route or schema fields are invalid.
    pub fn validate(&self) -> ControlResult<()> {
        validate_route_name("tool activity_type", self.activity_type.as_str())?;
        validate_route_name("tool task_queue", self.task_queue.as_str())?;
        validate_optional_hash("tool input_schema_hash", self.input_schema_hash.as_ref())?;
        validate_optional_hash("tool output_schema_hash", self.output_schema_hash.as_ref())?;
        Ok(())
    }

    /// Produces a deterministic permission decision for one proposal.
    ///
    /// # Errors
    ///
    /// Returns a control error when the registry entry or proposal is invalid,
    /// or when the proposal does not match this registry entry.
    pub fn decide_for_proposal(
        &self,
        proposal: &AgentProposal,
    ) -> ControlResult<ToolPermissionDecision> {
        self.validate()?;
        proposal.validate()?;

        let Some(proposed_tool_name) = proposal.tool_name.as_deref() else {
            return Err(invalid_tool_contract(
                "tool proposal requires tool_name before registry binding",
            ));
        };
        if proposed_tool_name != self.tool_name.as_str() {
            return Err(invalid_tool_contract(
                "tool proposal tool_name does not match registry entry",
            ));
        }
        if let Some(expected_output_hash) = &self.output_schema_hash
            && proposal.output_schema_hash.as_deref() != Some(expected_output_hash.as_str())
        {
            return Err(invalid_tool_contract(
                "tool proposal output_schema_hash does not match registry entry",
            ));
        }

        let decision = match self.permission_mode {
            ToolPermissionMode::Automatic => ToolPermissionDecision::Allowed {
                activity_type: self.activity_type.clone(),
                task_queue: self.task_queue.clone(),
            },
            ToolPermissionMode::HumanApprovalRequired => ToolPermissionDecision::ApprovalRequired {
                activity_type: self.activity_type.clone(),
                task_queue: self.task_queue.clone(),
                permission_scope: self.permission_scope.clone(),
            },
            ToolPermissionMode::Denied => ToolPermissionDecision::Denied {
                risk_level: self.risk_level,
            },
        };
        Ok(decision)
    }
}

/// Deterministic tool permission decision.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolPermissionDecision {
    /// Tool may be scheduled as the declared activity.
    Allowed {
        /// Activity type to schedule.
        activity_type: ActivityType,
        /// Task queue to route to.
        task_queue: TaskQueue,
    },
    /// Tool requires approval before scheduling.
    ApprovalRequired {
        /// Activity type that would be scheduled after approval.
        activity_type: ActivityType,
        /// Task queue that would be used after approval.
        task_queue: TaskQueue,
        /// Optional approval scope.
        permission_scope: Option<PermissionScope>,
    },
    /// Tool is denied by registry policy.
    Denied {
        /// Risk level that informed denial.
        risk_level: ToolRiskLevel,
    },
}

/// Deterministic authorization outcome for a proposed tool.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolAuthorizationDecision {
    /// Tool may be scheduled by a later reducer.
    Authorized {
        /// Activity type a later reducer may schedule.
        activity_type: ActivityType,
        /// Task queue a later reducer may route to.
        task_queue: TaskQueue,
        /// Approval request that authorized the tool, when applicable.
        approval_request_id: Option<ApprovalRequestId>,
    },
    /// Tool requires human approval before it can be authorized.
    WaitingApproval {
        /// Activity type that may be scheduled after approval.
        activity_type: ActivityType,
        /// Task queue that may be used after approval.
        task_queue: TaskQueue,
        /// Optional approval scope.
        permission_scope: Option<PermissionScope>,
    },
    /// Tool approval was rejected.
    Rejected {
        /// Approval request that rejected the tool.
        approval_request_id: ApprovalRequestId,
    },
    /// Tool is denied by registry policy.
    Denied {
        /// Risk level that informed denial.
        risk_level: ToolRiskLevel,
    },
    /// Tool approval timed out.
    TimedOut {
        /// Approval request that timed out.
        approval_request_id: ApprovalRequestId,
    },
}

impl ToolAuthorizationDecision {
    /// Resolves a tool permission that does not yet have an approval result.
    ///
    /// # Errors
    ///
    /// Returns a control error when the permission fields are invalid.
    pub fn from_permission(permission: &ToolPermissionDecision) -> ControlResult<Self> {
        match permission {
            ToolPermissionDecision::Allowed {
                activity_type,
                task_queue,
            } => {
                validate_route_name("tool activity_type", activity_type.as_str())?;
                validate_route_name("tool task_queue", task_queue.as_str())?;
                Ok(Self::Authorized {
                    activity_type: activity_type.clone(),
                    task_queue: task_queue.clone(),
                    approval_request_id: None,
                })
            }
            ToolPermissionDecision::ApprovalRequired {
                activity_type,
                task_queue,
                permission_scope,
            } => {
                validate_route_name("tool activity_type", activity_type.as_str())?;
                validate_route_name("tool task_queue", task_queue.as_str())?;
                Ok(Self::WaitingApproval {
                    activity_type: activity_type.clone(),
                    task_queue: task_queue.clone(),
                    permission_scope: permission_scope.clone(),
                })
            }
            ToolPermissionDecision::Denied { risk_level } => Ok(Self::Denied {
                risk_level: *risk_level,
            }),
        }
    }

    /// Resolves an approval-required tool permission with a human approval
    /// decision.
    ///
    /// # Errors
    ///
    /// Returns a control error when the permission does not require approval
    /// or when request ids do not match.
    pub fn from_approval_decision(
        permission: &ToolPermissionDecision,
        expected_approval_request_id: &ApprovalRequestId,
        approval_decision: &HumanApprovalDecision,
    ) -> ControlResult<Self> {
        let ToolPermissionDecision::ApprovalRequired {
            activity_type,
            task_queue,
            ..
        } = permission
        else {
            return Err(invalid_tool_contract(
                "tool authorization with approval decision requires approval-required permission",
            ));
        };
        validate_approval_request_id(
            expected_approval_request_id,
            &approval_decision.approval_request_id,
        )?;
        validate_route_name("tool activity_type", activity_type.as_str())?;
        validate_route_name("tool task_queue", task_queue.as_str())?;

        match approval_decision.status {
            HumanApprovalDecisionStatus::Approved => Ok(Self::Authorized {
                activity_type: activity_type.clone(),
                task_queue: task_queue.clone(),
                approval_request_id: Some(approval_decision.approval_request_id.clone()),
            }),
            HumanApprovalDecisionStatus::Rejected => Ok(Self::Rejected {
                approval_request_id: approval_decision.approval_request_id.clone(),
            }),
        }
    }

    /// Resolves an approval-required tool permission with a timeout
    /// resolution.
    ///
    /// # Errors
    ///
    /// Returns a control error when the permission does not require approval
    /// or when the resolution is not a matching timeout.
    pub fn from_approval_timeout(
        permission: &ToolPermissionDecision,
        expected_approval_request_id: &ApprovalRequestId,
        approval_resolution: &HumanApprovalResolution,
    ) -> ControlResult<Self> {
        if !matches!(permission, ToolPermissionDecision::ApprovalRequired { .. }) {
            return Err(invalid_tool_contract(
                "tool authorization timeout requires approval-required permission",
            ));
        }
        let HumanApprovalResolution::TimedOut {
            approval_request_id,
            ..
        } = approval_resolution
        else {
            return Err(invalid_tool_contract(
                "tool authorization timeout requires a timed-out approval resolution",
            ));
        };
        validate_approval_request_id(expected_approval_request_id, approval_request_id)?;
        Ok(Self::TimedOut {
            approval_request_id: approval_request_id.clone(),
        })
    }
}

fn validate_approval_request_id(
    expected: &ApprovalRequestId,
    actual: &ApprovalRequestId,
) -> ControlResult<()> {
    if expected != actual {
        return Err(invalid_tool_contract(
            "tool authorization approval_request_id does not match request",
        ));
    }
    Ok(())
}

fn validate_route_name(field: &'static str, value: &str) -> ControlResult<()> {
    if !value.contains('.') {
        return Err(invalid_tool_contract(match field {
            "tool activity_type" => "tool activity_type must contain a namespace separator",
            "tool task_queue" => "tool task_queue must contain a namespace separator",
            _ => "tool route must contain a namespace separator",
        }));
    }
    Ok(())
}

fn validate_optional_hash(field: &'static str, hash: Option<&String>) -> ControlResult<()> {
    if hash.is_some_and(|value| value.trim().is_empty()) {
        return Err(invalid_tool_contract(match field {
            "tool input_schema_hash" => "tool input_schema_hash must not be blank when supplied",
            "tool output_schema_hash" => "tool output_schema_hash must not be blank when supplied",
            _ => "tool schema hash must not be blank when supplied",
        }));
    }
    Ok(())
}

fn invalid_tool_contract(message: &str) -> ControlError {
    ControlError::InvalidEventSequence {
        message: message.to_owned(),
    }
}
