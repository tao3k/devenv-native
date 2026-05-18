use std::error::Error;

use xiuxian_qianji_control::{
    ActivityType, AgentProposal, AgentProposalId, ApprovalRequestId, ControlError,
    HumanApprovalDecision, HumanApprovalDecisionStatus, HumanApprovalResolution, PermissionScope,
    StepId, TaskQueue, TimerId, TokenId, ToolActivityContract, ToolAuthorizationDecision, ToolName,
    ToolPermissionDecision, ToolPermissionMode, ToolRiskLevel,
};

#[test]
fn tool_activity_contract_rejects_invalid_registry_entries() -> Result<(), Box<dyn Error>> {
    let missing_activity_namespace = tool_contract("web.fetch")?;
    let missing_activity_namespace = ToolActivityContract {
        activity_type: ActivityType::new("fetch")?,
        ..missing_activity_namespace
    };
    assert!(matches!(
        missing_activity_namespace.validate(),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    let blank_schema_hash = tool_contract("web.fetch")?.with_output_schema_hash(" ");
    assert!(matches!(
        blank_schema_hash.validate(),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    Ok(())
}

#[test]
fn tool_activity_contract_allows_automatic_permission_decision() -> Result<(), Box<dyn Error>> {
    let contract = tool_contract("web.fetch")?
        .with_input_schema_hash("sha256:input")
        .with_output_schema_hash("sha256:output");
    let proposal = tool_proposal("web.fetch")?.with_output_schema_hash("sha256:output");

    assert_eq!(
        contract.decide_for_proposal(&proposal)?,
        ToolPermissionDecision::Allowed {
            activity_type: ActivityType::new("tool.web_fetch")?,
            task_queue: TaskQueue::new("tool.web")?,
        }
    );

    Ok(())
}

#[test]
fn tool_activity_contract_reports_approval_and_denial_decisions() -> Result<(), Box<dyn Error>> {
    let approval_contract = ToolActivityContract::new(
        ToolName::new("github.pr.merge")?,
        ActivityType::new("tool.github_merge")?,
        TaskQueue::new("tool.github")?,
    )
    .with_risk_level(ToolRiskLevel::High)
    .with_permission_mode(ToolPermissionMode::HumanApprovalRequired)
    .with_permission_scope(PermissionScope::new("human.approval.github")?);
    let approval_proposal = tool_proposal("github.pr.merge")?;

    assert_eq!(
        approval_contract.decide_for_proposal(&approval_proposal)?,
        ToolPermissionDecision::ApprovalRequired {
            activity_type: ActivityType::new("tool.github_merge")?,
            task_queue: TaskQueue::new("tool.github")?,
            permission_scope: Some(PermissionScope::new("human.approval.github")?),
        }
    );

    let denied_contract = ToolActivityContract::new(
        ToolName::new("shell.rm_rf")?,
        ActivityType::new("tool.shell")?,
        TaskQueue::new("tool.shell")?,
    )
    .with_risk_level(ToolRiskLevel::Critical)
    .with_permission_mode(ToolPermissionMode::Denied);
    let denied_proposal = tool_proposal("shell.rm_rf")?;

    assert_eq!(
        denied_contract.decide_for_proposal(&denied_proposal)?,
        ToolPermissionDecision::Denied {
            risk_level: ToolRiskLevel::Critical,
        }
    );

    Ok(())
}

#[test]
fn tool_activity_contract_rejects_proposal_mismatches() -> Result<(), Box<dyn Error>> {
    let contract = tool_contract("web.fetch")?.with_output_schema_hash("sha256:output");

    let missing_tool = AgentProposal::new(
        AgentProposalId::new("proposal-missing-tool")?,
        StepId::new("stage-tool")?,
        TokenId::new("token-tool")?,
        "call_tool",
    );
    assert!(matches!(
        contract.decide_for_proposal(&missing_tool),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    let wrong_tool = tool_proposal("github.pr.merge")?;
    assert!(matches!(
        contract.decide_for_proposal(&wrong_tool),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    let wrong_schema = tool_proposal("web.fetch")?.with_output_schema_hash("sha256:other");
    assert!(matches!(
        contract.decide_for_proposal(&wrong_schema),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    Ok(())
}

#[test]
fn tool_authorization_decision_resolves_non_approval_permissions() -> Result<(), Box<dyn Error>> {
    let allowed = ToolPermissionDecision::Allowed {
        activity_type: ActivityType::new("tool.web_fetch")?,
        task_queue: TaskQueue::new("tool.web")?,
    };
    assert_eq!(
        ToolAuthorizationDecision::from_permission(&allowed)?,
        ToolAuthorizationDecision::Authorized {
            activity_type: ActivityType::new("tool.web_fetch")?,
            task_queue: TaskQueue::new("tool.web")?,
            approval_request_id: None,
        }
    );

    let approval_required = ToolPermissionDecision::ApprovalRequired {
        activity_type: ActivityType::new("tool.github_merge")?,
        task_queue: TaskQueue::new("tool.github")?,
        permission_scope: Some(PermissionScope::new("human.approval.github")?),
    };
    assert_eq!(
        ToolAuthorizationDecision::from_permission(&approval_required)?,
        ToolAuthorizationDecision::WaitingApproval {
            activity_type: ActivityType::new("tool.github_merge")?,
            task_queue: TaskQueue::new("tool.github")?,
            permission_scope: Some(PermissionScope::new("human.approval.github")?),
        }
    );

    let denied = ToolPermissionDecision::Denied {
        risk_level: ToolRiskLevel::Critical,
    };
    assert_eq!(
        ToolAuthorizationDecision::from_permission(&denied)?,
        ToolAuthorizationDecision::Denied {
            risk_level: ToolRiskLevel::Critical,
        }
    );

    Ok(())
}

#[test]
fn tool_authorization_decision_resolves_approval_decisions() -> Result<(), Box<dyn Error>> {
    let permission = approval_required_permission()?;
    let approval_request_id = ApprovalRequestId::new("approval-1")?;
    let approved = HumanApprovalDecision::new(
        approval_request_id.clone(),
        HumanApprovalDecisionStatus::Approved,
    );

    assert_eq!(
        ToolAuthorizationDecision::from_approval_decision(
            &permission,
            &approval_request_id,
            &approved,
        )?,
        ToolAuthorizationDecision::Authorized {
            activity_type: ActivityType::new("tool.github_merge")?,
            task_queue: TaskQueue::new("tool.github")?,
            approval_request_id: Some(ApprovalRequestId::new("approval-1")?),
        }
    );

    let rejected = HumanApprovalDecision::new(
        approval_request_id.clone(),
        HumanApprovalDecisionStatus::Rejected,
    );
    assert_eq!(
        ToolAuthorizationDecision::from_approval_decision(
            &permission,
            &approval_request_id,
            &rejected,
        )?,
        ToolAuthorizationDecision::Rejected {
            approval_request_id: ApprovalRequestId::new("approval-1")?,
        }
    );

    Ok(())
}

#[test]
fn tool_authorization_decision_resolves_approval_timeout() -> Result<(), Box<dyn Error>> {
    let permission = approval_required_permission()?;
    let approval_request_id = ApprovalRequestId::new("approval-1")?;
    let timeout = HumanApprovalResolution::TimedOut {
        approval_request_id: approval_request_id.clone(),
        timer_id: TimerId::new("approval-timeout")?,
    };

    assert_eq!(
        ToolAuthorizationDecision::from_approval_timeout(
            &permission,
            &approval_request_id,
            &timeout,
        )?,
        ToolAuthorizationDecision::TimedOut {
            approval_request_id: ApprovalRequestId::new("approval-1")?,
        }
    );

    Ok(())
}

#[test]
fn tool_authorization_decision_rejects_invalid_approval_resolution_inputs()
-> Result<(), Box<dyn Error>> {
    let permission = approval_required_permission()?;
    let expected_approval_request_id = ApprovalRequestId::new("approval-1")?;
    let wrong_approval = HumanApprovalDecision::new(
        ApprovalRequestId::new("approval-other")?,
        HumanApprovalDecisionStatus::Approved,
    );
    assert!(matches!(
        ToolAuthorizationDecision::from_approval_decision(
            &permission,
            &expected_approval_request_id,
            &wrong_approval,
        ),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    let allowed = ToolPermissionDecision::Allowed {
        activity_type: ActivityType::new("tool.web_fetch")?,
        task_queue: TaskQueue::new("tool.web")?,
    };
    let approval = HumanApprovalDecision::new(
        expected_approval_request_id.clone(),
        HumanApprovalDecisionStatus::Approved,
    );
    assert!(matches!(
        ToolAuthorizationDecision::from_approval_decision(
            &allowed,
            &expected_approval_request_id,
            &approval,
        ),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    let signal_matched = HumanApprovalResolution::SignalMatched {
        approval_request_id: expected_approval_request_id.clone(),
        payload_hash: None,
    };
    assert!(matches!(
        ToolAuthorizationDecision::from_approval_timeout(
            &permission,
            &expected_approval_request_id,
            &signal_matched,
        ),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    Ok(())
}

fn tool_contract(tool_name: &str) -> Result<ToolActivityContract, Box<dyn Error>> {
    Ok(ToolActivityContract::new(
        ToolName::new(tool_name)?,
        ActivityType::new("tool.web_fetch")?,
        TaskQueue::new("tool.web")?,
    ))
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

fn approval_required_permission() -> Result<ToolPermissionDecision, Box<dyn Error>> {
    Ok(ToolPermissionDecision::ApprovalRequired {
        activity_type: ActivityType::new("tool.github_merge")?,
        task_queue: TaskQueue::new("tool.github")?,
        permission_scope: Some(PermissionScope::new("human.approval.github")?),
    })
}
