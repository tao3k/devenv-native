use std::error::Error;

use serde_json::json;
use xiuxian_qianji_control::{
    AgentProposalId, ApprovalRequestId, ApproverId, ArtifactId, ArtifactKind, ArtifactRef,
    ControlError, HumanApprovalDecision, HumanApprovalDecisionStatus, HumanApprovalRequest,
    SignalName, SignalRecord,
};

#[test]
fn human_approval_decision_parses_approved_signal_metadata() -> Result<(), Box<dyn Error>> {
    let request = approval_request()?.with_expected_payload_hash("sha256:approval");
    let signal = approval_signal(json!({
        "decision": "approved",
        "decided_by": "user.alice",
        "reason": "reviewed risk and approved",
    }))?;

    let decision = HumanApprovalDecision::from_signal(&request, &signal)?;

    assert_eq!(decision.status, HumanApprovalDecisionStatus::Approved);
    assert_eq!(decision.decided_by, Some(ApproverId::new("user.alice")?));
    assert_eq!(
        decision.reason.as_deref(),
        Some("reviewed risk and approved")
    );
    assert_eq!(decision.payload_hash.as_deref(), Some("sha256:approval"));

    Ok(())
}

#[test]
fn human_approval_decision_parses_rejected_signal_metadata() -> Result<(), Box<dyn Error>> {
    let request = approval_request()?.with_expected_payload_hash("sha256:approval");
    let signal = approval_signal(json!({
        "decision": "rejected",
        "decided_by": "user.alice",
        "reason": "missing evidence",
    }))?;

    let decision = HumanApprovalDecision::from_signal(&request, &signal)?;

    assert_eq!(decision.status, HumanApprovalDecisionStatus::Rejected);
    assert_eq!(decision.decided_by, Some(ApproverId::new("user.alice")?));
    assert_eq!(decision.reason.as_deref(), Some("missing evidence"));

    Ok(())
}

#[test]
fn human_approval_decision_rejects_missing_or_invalid_metadata() -> Result<(), Box<dyn Error>> {
    let request = approval_request()?.with_expected_payload_hash("sha256:approval");

    for metadata in [
        json!({}),
        json!({ "decision": "maybe" }),
        json!({ "decision": "" }),
        json!({ "decision": 1 }),
        json!({ "decision": "approved", "decided_by": "" }),
        json!({ "decision": "approved", "reason": 1 }),
    ] {
        let signal = approval_signal(metadata)?;
        assert!(matches!(
            HumanApprovalDecision::from_signal(&request, &signal),
            Err(ControlError::InvalidEventSequence { .. })
        ));
    }

    let wrong_signal = SignalRecord {
        signal_name: SignalName::new("human.other")?,
        payload_ref: None,
        payload_hash: Some("sha256:approval".to_owned()),
        metadata: json!({ "decision": "approved" }),
    };
    assert!(matches!(
        HumanApprovalDecision::from_signal(&request, &wrong_signal),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    Ok(())
}

#[test]
fn human_approval_decision_rejects_blank_optional_fields() -> Result<(), Box<dyn Error>> {
    let blank_reason = HumanApprovalDecision::new(
        ApprovalRequestId::new("approval-1")?,
        HumanApprovalDecisionStatus::Approved,
    )
    .with_reason(" ");
    assert!(matches!(
        blank_reason.validate(),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    let blank_hash = HumanApprovalDecision::new(
        ApprovalRequestId::new("approval-1")?,
        HumanApprovalDecisionStatus::Approved,
    )
    .with_payload_hash(" ");
    assert!(matches!(
        blank_hash.validate(),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    Ok(())
}

fn approval_request() -> Result<HumanApprovalRequest, Box<dyn Error>> {
    Ok(HumanApprovalRequest::new(
        ApprovalRequestId::new("approval-1")?,
        AgentProposalId::new("proposal-1")?,
        SignalName::new("human.approval")?,
    ))
}

fn approval_signal(metadata: serde_json::Value) -> Result<SignalRecord, Box<dyn Error>> {
    Ok(SignalRecord {
        signal_name: SignalName::new("human.approval")?,
        payload_ref: Some(ArtifactRef {
            artifact_id: ArtifactId::new("approval-payload")?,
            artifact_kind: ArtifactKind::new("approval_signal")?,
            uri: "wendao://approval/signal".to_owned(),
            content_digest: Some("sha256:approval".to_owned()),
            metadata: serde_json::Value::Null,
        }),
        payload_hash: Some("sha256:approval".to_owned()),
        metadata,
    })
}
