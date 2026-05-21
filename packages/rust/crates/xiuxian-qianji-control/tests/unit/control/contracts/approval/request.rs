use std::error::Error;

use xiuxian_qianji_control::{
    ActivityType, AgentProposalId, ApprovalRequestId, ArtifactId, ArtifactKind, ArtifactRef,
    ControlError, HumanApprovalRequest, HumanApprovalResolution, PermissionScope, SignalName,
    SignalRecord, TaskQueue, TimerId, TimerRecord, ToolPermissionDecision, ToolRiskLevel,
};

#[test]
fn human_approval_request_builds_from_approval_required_tool_decision() -> Result<(), Box<dyn Error>>
{
    let decision = ToolPermissionDecision::ApprovalRequired {
        activity_type: ActivityType::new("tool.github_merge")?,
        task_queue: TaskQueue::new("tool.github")?,
        permission_scope: Some(PermissionScope::new("human.approval.github")?),
    };

    let request = HumanApprovalRequest::from_tool_permission(
        ApprovalRequestId::new("approval-1")?,
        AgentProposalId::new("proposal-1")?,
        SignalName::new("human.approval")?,
        &decision,
    )?;

    assert_eq!(
        request.permission_scope,
        Some(PermissionScope::new("human.approval.github")?)
    );
    assert!(request.validate().is_ok());

    let allowed = ToolPermissionDecision::Allowed {
        activity_type: ActivityType::new("tool.web_fetch")?,
        task_queue: TaskQueue::new("tool.web")?,
    };
    assert!(matches!(
        HumanApprovalRequest::from_tool_permission(
            ApprovalRequestId::new("approval-allowed")?,
            AgentProposalId::new("proposal-allowed")?,
            SignalName::new("human.approval")?,
            &allowed,
        ),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    let denied = ToolPermissionDecision::Denied {
        risk_level: ToolRiskLevel::Critical,
    };
    assert!(matches!(
        HumanApprovalRequest::from_tool_permission(
            ApprovalRequestId::new("approval-denied")?,
            AgentProposalId::new("proposal-denied")?,
            SignalName::new("human.approval")?,
            &denied,
        ),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    Ok(())
}

#[test]
fn human_approval_request_matches_signal_and_expected_payload_hash() -> Result<(), Box<dyn Error>> {
    let request = approval_request()?.with_expected_payload_hash("sha256:approval");
    let approval_signal = make_signal("human.approval", Some("sha256:approval"))?;

    assert_eq!(
        request.match_signal(&approval_signal)?,
        HumanApprovalResolution::SignalMatched {
            approval_request_id: ApprovalRequestId::new("approval-1")?,
            payload_hash: Some("sha256:approval".to_owned()),
        }
    );

    let wrong_signal = make_signal("human.reject", Some("sha256:approval"))?;
    assert!(matches!(
        request.match_signal(&wrong_signal),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    let wrong_hash = make_signal("human.approval", Some("sha256:other"))?;
    assert!(matches!(
        request.match_signal(&wrong_hash),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    Ok(())
}

#[test]
fn human_approval_request_matches_timeout_timer() -> Result<(), Box<dyn Error>> {
    let request = approval_request()?.with_timeout_timer_id(TimerId::new("approval-timeout")?);
    let approval_timer = make_timer("approval-timeout")?;

    assert_eq!(
        request.match_timer(&approval_timer)?,
        HumanApprovalResolution::TimedOut {
            approval_request_id: ApprovalRequestId::new("approval-1")?,
            timer_id: TimerId::new("approval-timeout")?,
        }
    );

    let wrong_timer = make_timer("other-timeout")?;
    assert!(matches!(
        request.match_timer(&wrong_timer),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    let no_timer_request = approval_request()?;
    assert!(matches!(
        no_timer_request.match_timer(&approval_timer),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    Ok(())
}

#[test]
fn human_approval_request_rejects_blank_expected_payload_hash() -> Result<(), Box<dyn Error>> {
    let request = approval_request()?.with_expected_payload_hash(" ");

    assert!(matches!(
        request.validate(),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    Ok(())
}

fn approval_request() -> Result<HumanApprovalRequest, Box<dyn Error>> {
    Ok(HumanApprovalRequest::new(
        ApprovalRequestId::new("approval-1")?,
        AgentProposalId::new("proposal-1")?,
        SignalName::new("human.approval")?,
    )
    .with_permission_scope(PermissionScope::new("human.approval.github")?)
    .with_payload_ref(ArtifactRef {
        artifact_id: ArtifactId::new("approval-context")?,
        artifact_kind: ArtifactKind::new("approval_payload")?,
        uri: "wendao://approval/context".to_owned(),
        content_digest: Some("sha256:approval-context".to_owned()),
        metadata: serde_json::Value::Null,
    }))
}

fn make_signal(name: &str, payload_hash: Option<&str>) -> Result<SignalRecord, Box<dyn Error>> {
    Ok(SignalRecord {
        signal_name: SignalName::new(name)?,
        payload_ref: None,
        payload_hash: payload_hash.map(str::to_owned),
        metadata: serde_json::Value::Null,
    })
}

fn make_timer(timer_id: &str) -> Result<TimerRecord, Box<dyn Error>> {
    Ok(TimerRecord {
        timer_id: TimerId::new(timer_id)?,
        fire_at_ms: 1_780_000_000_000,
        metadata: serde_json::Value::Null,
    })
}
