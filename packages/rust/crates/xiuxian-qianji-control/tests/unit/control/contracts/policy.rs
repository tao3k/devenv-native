use std::error::Error;

use xiuxian_qianji_control::{
    ActivityId, ActivityTask, ActivityType, AgentDecisionId, AgentDecisionOutcome,
    AgentPolicyReason, AgentProposal, AgentProposalId, ApprovalRequestId, ControlError,
    DecisionReasonCode, GateName, GateResult, IdempotencyKey, PermissionScope, StepId, TaskQueue,
    TokenId, ToolAuthorizationDecision, ToolPolicyReductionRequest, ToolRiskLevel,
};

use crate::control::support::artifact_ref;

#[test]
fn tool_policy_reducer_accepts_authorized_tool_activity() -> Result<(), Box<dyn Error>> {
    let reduction = ToolPolicyReductionRequest::new(
        AgentDecisionId::new("decision-web-fetch")?,
        tool_proposal("web.fetch")?.with_tool_input_ref(artifact_ref("artifact-web-input")?),
        ToolAuthorizationDecision::Authorized {
            activity_type: ActivityType::new("tool.web_fetch")?,
            task_queue: TaskQueue::new("tool.web")?,
            approval_request_id: None,
        },
    )
    .with_task(
        activity_task("tool.web_fetch", "tool.web")?
            .with_input_ref(artifact_ref("artifact-web-input")?),
    )
    .with_gate_result(passing_gate()?)
    .with_checkpoint_seq(7)
    .reduce()?;

    reduction.validate()?;
    assert_eq!(reduction.decision.outcome, AgentDecisionOutcome::Accepted);
    assert_eq!(
        reduction.decision.reason_code,
        DecisionReasonCode::new(AgentPolicyReason::ToolAuthorized.as_str())?
    );
    assert_eq!(
        reduction.decision.scheduled_activity_id,
        Some(ActivityId::new("activity-tool-web_fetch")?)
    );
    assert_eq!(reduction.decision.checkpoint_seq, Some(7));
    assert_eq!(reduction.decision.gate_result, Some(passing_gate()?));
    assert!(reduction.admission.is_some());

    Ok(())
}

#[test]
fn tool_policy_reducer_requires_task_for_authorized_tool() -> Result<(), Box<dyn Error>> {
    let result = ToolPolicyReductionRequest::new(
        AgentDecisionId::new("decision-missing-task")?,
        tool_proposal("web.fetch")?,
        ToolAuthorizationDecision::Authorized {
            activity_type: ActivityType::new("tool.web_fetch")?,
            task_queue: TaskQueue::new("tool.web")?,
            approval_request_id: None,
        },
    )
    .reduce();

    assert!(matches!(
        result,
        Err(ControlError::InvalidEventSequence { .. })
    ));

    Ok(())
}

#[test]
fn tool_policy_reducer_waits_for_approval_without_activity() -> Result<(), Box<dyn Error>> {
    let reduction = ToolPolicyReductionRequest::new(
        AgentDecisionId::new("decision-approval")?,
        tool_proposal("github.pr.merge")?,
        ToolAuthorizationDecision::WaitingApproval {
            activity_type: ActivityType::new("tool.github_merge")?,
            task_queue: TaskQueue::new("tool.github")?,
            permission_scope: Some(PermissionScope::new("human.approval.github")?),
        },
    )
    .reduce()?;

    reduction.validate()?;
    assert_eq!(
        reduction.decision.outcome,
        AgentDecisionOutcome::ApprovalRequired
    );
    assert_eq!(
        reduction.decision.reason_code,
        DecisionReasonCode::new(AgentPolicyReason::ToolApprovalRequired.as_str())?
    );
    assert_eq!(reduction.decision.scheduled_activity_id, None);
    assert!(reduction.admission.is_none());

    Ok(())
}

#[test]
fn tool_policy_reducer_rejects_denied_rejected_and_timed_out_tools() -> Result<(), Box<dyn Error>> {
    let cases = [
        (
            ToolAuthorizationDecision::Denied {
                risk_level: ToolRiskLevel::Critical,
            },
            AgentPolicyReason::ToolDenied,
        ),
        (
            ToolAuthorizationDecision::Rejected {
                approval_request_id: ApprovalRequestId::new("approval-rejected")?,
            },
            AgentPolicyReason::ToolApprovalRejected,
        ),
        (
            ToolAuthorizationDecision::TimedOut {
                approval_request_id: ApprovalRequestId::new("approval-timeout")?,
            },
            AgentPolicyReason::ToolApprovalTimedOut,
        ),
    ];

    for (index, (authorization, expected_reason)) in cases.into_iter().enumerate() {
        let reduction = ToolPolicyReductionRequest::new(
            AgentDecisionId::new(format!("decision-rejected-{index}"))?,
            tool_proposal("github.pr.merge")?,
            authorization,
        )
        .reduce()?;

        reduction.validate()?;
        assert_eq!(reduction.decision.outcome, AgentDecisionOutcome::Rejected);
        assert_eq!(
            reduction.decision.reason_code,
            DecisionReasonCode::new(expected_reason.as_str())?
        );
        assert_eq!(reduction.decision.scheduled_activity_id, None);
        assert!(reduction.admission.is_none());
    }

    Ok(())
}

#[test]
fn tool_policy_reducer_rejects_failed_gate_before_scheduling() -> Result<(), Box<dyn Error>> {
    let reduction = ToolPolicyReductionRequest::new(
        AgentDecisionId::new("decision-gate-failed")?,
        tool_proposal("web.fetch")?,
        ToolAuthorizationDecision::Authorized {
            activity_type: ActivityType::new("tool.web_fetch")?,
            task_queue: TaskQueue::new("tool.web")?,
            approval_request_id: None,
        },
    )
    .with_task(activity_task("tool.web_fetch", "tool.web")?)
    .with_gate_result(failing_gate()?)
    .reduce()?;

    reduction.validate()?;
    assert_eq!(reduction.decision.outcome, AgentDecisionOutcome::Rejected);
    assert_eq!(
        reduction.decision.reason_code,
        DecisionReasonCode::new(AgentPolicyReason::GateFailed.as_str())?
    );
    assert_eq!(reduction.decision.scheduled_activity_id, None);
    assert_eq!(reduction.decision.gate_result, Some(failing_gate()?));
    assert!(reduction.admission.is_none());

    Ok(())
}

#[test]
fn tool_policy_reducer_rejects_task_on_non_authorized_outcome() -> Result<(), Box<dyn Error>> {
    let result = ToolPolicyReductionRequest::new(
        AgentDecisionId::new("decision-task-smuggle")?,
        tool_proposal("github.pr.merge")?,
        ToolAuthorizationDecision::WaitingApproval {
            activity_type: ActivityType::new("tool.github_merge")?,
            task_queue: TaskQueue::new("tool.github")?,
            permission_scope: None,
        },
    )
    .with_task(activity_task("tool.github_merge", "tool.github")?)
    .reduce();

    assert!(matches!(
        result,
        Err(ControlError::InvalidEventSequence { .. })
    ));

    Ok(())
}

fn tool_proposal(tool_name: &str) -> Result<AgentProposal, Box<dyn Error>> {
    Ok(AgentProposal::new(
        AgentProposalId::new(format!("proposal-{}", tool_name.replace('.', "-")))?,
        StepId::new("stage-tool")?,
        TokenId::new("token-tool")?,
        "call_tool",
    )
    .with_tool_name(tool_name))
}

fn activity_task(activity_type: &str, task_queue: &str) -> Result<ActivityTask, Box<dyn Error>> {
    Ok(ActivityTask::new(
        ActivityId::new(format!("activity-{}", activity_type.replace('.', "-")))?,
        ActivityType::new(activity_type)?,
        TaskQueue::new(task_queue)?,
        IdempotencyKey::new(format!("run/{activity_type}/{task_queue}"))?,
    ))
}

fn passing_gate() -> Result<GateResult, Box<dyn Error>> {
    Ok(GateResult {
        gate_name: GateName::new("required_evidence")?,
        passed: true,
        required_evidence_covered: true,
        selected_required_evidence: vec!["ownership_boundary".to_owned()],
        missing_required_evidence: Vec::new(),
        reasons: Vec::new(),
        metadata: serde_json::Value::Null,
    })
}

fn failing_gate() -> Result<GateResult, Box<dyn Error>> {
    Ok(GateResult {
        gate_name: GateName::new("required_evidence")?,
        passed: false,
        required_evidence_covered: false,
        selected_required_evidence: Vec::new(),
        missing_required_evidence: vec!["validation_path".to_owned()],
        reasons: vec!["missing required evidence: validation_path".to_owned()],
        metadata: serde_json::Value::Null,
    })
}
