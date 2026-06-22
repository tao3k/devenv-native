use std::error::Error;

use xiuxian_qianji_control::{
    ActivityId, ActivityTask, ActivityType, AgentDecisionId, AgentDecisionOutcome, AgentProposal,
    AgentProposalId, ApprovalRequestId, ArtifactId, ArtifactKind, ArtifactRef, ControlError,
    DecisionReasonCode, IdempotencyKey, LlmActivityAdmission, LlmActivityRequest, LlmActivityTask,
    LlmModelId, PermissionScope, StepId, TaskQueue, TokenId, ToolActivityAdmission,
    ToolAuthorizationDecision, ToolName, ToolRiskLevel,
};

#[test]
fn llm_activity_admission_accepts_claim_check_task() -> Result<(), Box<dyn Error>> {
    let prompt_ref = input_ref("input-llm-prompt")?;
    let activity = LlmActivityTask::new(
        activity_task("llm.plan", "llm.openai")?.with_input_ref(prompt_ref.clone()),
        LlmActivityRequest::new(LlmModelId::new("openai/gpt-5.2")?, prompt_ref)
            .with_context_ref(input_ref("input-llm-context")?)
            .with_tool_schema_hash("sha256:tool-schema")
            .with_response_schema_ref(input_ref("input-response-schema")?),
    );

    let admission = LlmActivityAdmission::from_activity(activity)?;

    admission.validate()?;
    assert_eq!(admission.activity.task.activity_type.as_str(), "llm.plan");
    assert_eq!(admission.activity.task.task_queue.as_str(), "llm.openai");

    Ok(())
}

#[test]
fn llm_activity_admission_rejects_missing_or_mismatched_prompt_input_ref()
-> Result<(), Box<dyn Error>> {
    let prompt_ref = input_ref("input-llm-prompt")?;
    let missing_input = LlmActivityTask::new(
        activity_task("llm.plan", "llm.openai")?,
        LlmActivityRequest::new(LlmModelId::new("openai/gpt-5.2")?, prompt_ref.clone()),
    );
    assert!(matches!(
        LlmActivityAdmission::from_activity(missing_input),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    let mismatched_input = LlmActivityTask::new(
        activity_task("llm.plan", "llm.openai")?.with_input_ref(input_ref("input-other")?),
        LlmActivityRequest::new(LlmModelId::new("openai/gpt-5.2")?, prompt_ref),
    );
    assert!(matches!(
        LlmActivityAdmission::from_activity(mismatched_input),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    Ok(())
}

#[test]
fn llm_activity_admission_exposes_generic_activity_task() -> Result<(), Box<dyn Error>> {
    let prompt_ref = input_ref("input-llm-prompt")?;
    let admission = LlmActivityAdmission::from_activity(LlmActivityTask::new(
        activity_task("llm.plan", "llm.openai")?.with_input_ref(prompt_ref.clone()),
        LlmActivityRequest::new(LlmModelId::new("openai/gpt-5.2")?, prompt_ref),
    ))?;

    assert_eq!(
        admission.activity_task().activity_id,
        ActivityId::new("activity-llm-plan")?
    );
    assert_eq!(admission.activity_task().activity_type.as_str(), "llm.plan");

    Ok(())
}

#[test]
fn tool_activity_admission_accepts_authorized_tool_task() -> Result<(), Box<dyn Error>> {
    let proposal = tool_proposal("web.fetch")?.with_tool_input_ref(input_ref("input-web")?);
    let authorization = ToolAuthorizationDecision::Authorized {
        activity_type: ActivityType::new("tool.web_fetch")?,
        task_queue: TaskQueue::new("tool.web")?,
        approval_request_id: None,
    };
    let task = activity_task("tool.web_fetch", "tool.web")?.with_input_ref(input_ref("input-web")?);

    let admission = ToolActivityAdmission::from_authorized_tool(&proposal, &authorization, task)?;

    assert_eq!(
        admission.proposal_id,
        AgentProposalId::new("proposal-web-fetch")?
    );
    assert_eq!(admission.tool_name.as_str(), "web.fetch");
    assert_eq!(admission.approval_request_id, None);
    admission.validate()?;

    Ok(())
}

#[test]
fn tool_activity_admission_preserves_approval_provenance() -> Result<(), Box<dyn Error>> {
    let proposal = tool_proposal("github.pr.merge")?;
    let authorization = ToolAuthorizationDecision::Authorized {
        activity_type: ActivityType::new("tool.github_merge")?,
        task_queue: TaskQueue::new("tool.github")?,
        approval_request_id: Some(ApprovalRequestId::new("approval-1")?),
    };
    let task = activity_task("tool.github_merge", "tool.github")?;

    let admission = ToolActivityAdmission::from_authorized_tool(&proposal, &authorization, task)?;

    assert_eq!(
        admission.approval_request_id,
        Some(ApprovalRequestId::new("approval-1")?)
    );

    Ok(())
}

#[test]
fn tool_activity_admission_rejects_non_authorized_outcomes() -> Result<(), Box<dyn Error>> {
    let proposal = tool_proposal("github.pr.merge")?;
    let task = activity_task("tool.github_merge", "tool.github")?;

    for authorization in [
        ToolAuthorizationDecision::WaitingApproval {
            activity_type: ActivityType::new("tool.github_merge")?,
            task_queue: TaskQueue::new("tool.github")?,
            permission_scope: Some(PermissionScope::new("human.approval.github")?),
        },
        ToolAuthorizationDecision::Rejected {
            approval_request_id: ApprovalRequestId::new("approval-1")?,
        },
        ToolAuthorizationDecision::Denied {
            risk_level: ToolRiskLevel::Critical,
        },
        ToolAuthorizationDecision::TimedOut {
            approval_request_id: ApprovalRequestId::new("approval-1")?,
        },
    ] {
        assert!(matches!(
            ToolActivityAdmission::from_authorized_tool(&proposal, &authorization, task.clone(),),
            Err(ControlError::InvalidEventSequence { .. })
        ));
    }

    Ok(())
}

#[test]
fn tool_activity_admission_rejects_mismatched_task_or_proposal() -> Result<(), Box<dyn Error>> {
    let authorization = ToolAuthorizationDecision::Authorized {
        activity_type: ActivityType::new("tool.web_fetch")?,
        task_queue: TaskQueue::new("tool.web")?,
        approval_request_id: None,
    };

    let missing_tool = AgentProposal::new(
        AgentProposalId::new("proposal-missing-tool")?,
        StepId::new("stage-tool")?,
        TokenId::new("token-tool")?,
        "call_tool",
    );
    assert!(matches!(
        ToolActivityAdmission::from_authorized_tool(
            &missing_tool,
            &authorization,
            activity_task("tool.web_fetch", "tool.web")?,
        ),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    let proposal = tool_proposal("web.fetch")?.with_tool_input_ref(input_ref("input-web")?);
    assert!(matches!(
        ToolActivityAdmission::from_authorized_tool(
            &proposal,
            &authorization,
            activity_task("tool.other", "tool.web")?.with_input_ref(input_ref("input-web")?),
        ),
        Err(ControlError::InvalidEventSequence { .. })
    ));
    assert!(matches!(
        ToolActivityAdmission::from_authorized_tool(
            &proposal,
            &authorization,
            activity_task("tool.web_fetch", "tool.other")?.with_input_ref(input_ref("input-web")?),
        ),
        Err(ControlError::InvalidEventSequence { .. })
    ));
    assert!(matches!(
        ToolActivityAdmission::from_authorized_tool(
            &proposal,
            &authorization,
            activity_task("tool.web_fetch", "tool.web")?.with_input_ref(input_ref("input-other")?),
        ),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    Ok(())
}

#[test]
fn tool_activity_admission_builds_accepted_agent_decision() -> Result<(), Box<dyn Error>> {
    let proposal = tool_proposal("web.fetch")?;
    let authorization = ToolAuthorizationDecision::Authorized {
        activity_type: ActivityType::new("tool.web_fetch")?,
        task_queue: TaskQueue::new("tool.web")?,
        approval_request_id: None,
    };
    let task = activity_task("tool.web_fetch", "tool.web")?;
    let admission = ToolActivityAdmission::from_authorized_tool(&proposal, &authorization, task)?;

    let decision = admission.to_accepted_agent_decision(
        AgentDecisionId::new("decision-web-fetch")?,
        DecisionReasonCode::new("tool_authorized")?,
    )?;

    assert_eq!(
        decision.proposal_id,
        AgentProposalId::new("proposal-web-fetch")?
    );
    assert_eq!(decision.outcome, AgentDecisionOutcome::Accepted);
    assert_eq!(
        decision.scheduled_activity_id,
        Some(ActivityId::new("activity-tool-web_fetch")?)
    );
    decision.validate()?;

    Ok(())
}

#[test]
fn tool_activity_admission_rejects_accepted_decision_from_invalid_admission()
-> Result<(), Box<dyn Error>> {
    let invalid_admission = ToolActivityAdmission {
        proposal_id: AgentProposalId::new("proposal-web-fetch")?,
        tool_name: ToolName::new("web.fetch")?,
        task: activity_task("tool.web_fetch", "tool.web")?.with_timeout_ms(0),
        approval_request_id: None,
        metadata: serde_json::Value::Null,
    };

    assert!(matches!(
        invalid_admission.to_accepted_agent_decision(
            AgentDecisionId::new("decision-invalid")?,
            DecisionReasonCode::new("tool_authorized")?,
        ),
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

fn input_ref(artifact_id: &str) -> Result<ArtifactRef, Box<dyn Error>> {
    Ok(ArtifactRef {
        artifact_id: ArtifactId::new(artifact_id)?,
        artifact_kind: ArtifactKind::new("tool_input")?,
        uri: format!("wendao://tool/input/{artifact_id}"),
        content_digest: Some(format!("sha256:{artifact_id}")),
        metadata: serde_json::Value::Null,
    })
}
