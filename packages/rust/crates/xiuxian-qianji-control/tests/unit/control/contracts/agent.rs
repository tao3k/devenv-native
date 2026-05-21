use std::error::Error;

use xiuxian_qianji_control::{
    ActivityId, AgentDecision, AgentDecisionId, AgentDecisionOutcome, AgentProposal,
    AgentProposalId, ControlError, DecisionReasonCode, StepId, TokenId,
};

use crate::control::support::artifact_ref;

#[test]
fn agent_proposal_contract_rejects_invalid_payloads() -> Result<(), Box<dyn Error>> {
    let valid_proposal = AgentProposal::new(
        AgentProposalId::new("proposal-valid")?,
        StepId::new("stage-plan")?,
        TokenId::new("token-a")?,
        "schedule_llm_plan",
    )
    .with_tool_name("llm.plan")
    .with_tool_input_ref(artifact_ref("artifact-agent-input")?)
    .with_confidence_millis(875)
    .with_rationale_ref(artifact_ref("artifact-agent-rationale")?)
    .with_output_schema_hash("sha256:agent-output-schema");
    valid_proposal.validate()?;

    let blank_action = AgentProposal::new(
        AgentProposalId::new("proposal-blank-action")?,
        StepId::new("stage-plan")?,
        TokenId::new("token-a")?,
        " ",
    );
    assert!(matches!(
        blank_action.validate(),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    let blank_tool = AgentProposal::new(
        AgentProposalId::new("proposal-blank-tool")?,
        StepId::new("stage-plan")?,
        TokenId::new("token-a")?,
        "schedule_llm_plan",
    )
    .with_tool_name(" ");
    assert!(matches!(
        blank_tool.validate(),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    let invalid_confidence = AgentProposal::new(
        AgentProposalId::new("proposal-invalid-confidence")?,
        StepId::new("stage-plan")?,
        TokenId::new("token-a")?,
        "schedule_llm_plan",
    )
    .with_confidence_millis(1_001);
    assert!(matches!(
        invalid_confidence.validate(),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    let blank_schema_hash = AgentProposal::new(
        AgentProposalId::new("proposal-blank-schema")?,
        StepId::new("stage-plan")?,
        TokenId::new("token-a")?,
        "schedule_llm_plan",
    )
    .with_output_schema_hash(" ");
    assert!(matches!(
        blank_schema_hash.validate(),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    Ok(())
}

#[test]
fn agent_decision_contract_validates_outcome_activity_consistency() -> Result<(), Box<dyn Error>> {
    let proposal_id = AgentProposalId::new("proposal-decision")?;

    let accepted = AgentDecision::new(
        AgentDecisionId::new("decision-accepted")?,
        proposal_id.clone(),
        AgentDecisionOutcome::Accepted,
        DecisionReasonCode::new("policy_allowed")?,
    )
    .with_scheduled_activity_id(ActivityId::new("activity-accepted")?)
    .with_checkpoint_seq(1);
    accepted.validate()?;

    let accepted_without_activity = AgentDecision::new(
        AgentDecisionId::new("decision-missing-activity")?,
        proposal_id.clone(),
        AgentDecisionOutcome::Accepted,
        DecisionReasonCode::new("policy_allowed")?,
    );
    assert!(matches!(
        accepted_without_activity.validate(),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    let rejected_with_activity = AgentDecision::new(
        AgentDecisionId::new("decision-rejected-activity")?,
        proposal_id.clone(),
        AgentDecisionOutcome::Rejected,
        DecisionReasonCode::new("policy_denied")?,
    )
    .with_scheduled_activity_id(ActivityId::new("activity-rejected")?);
    assert!(matches!(
        rejected_with_activity.validate(),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    let approval_with_activity = AgentDecision::new(
        AgentDecisionId::new("decision-approval-activity")?,
        proposal_id.clone(),
        AgentDecisionOutcome::ApprovalRequired,
        DecisionReasonCode::new("approval_required")?,
    )
    .with_scheduled_activity_id(ActivityId::new("activity-approval")?);
    assert!(matches!(
        approval_with_activity.validate(),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    let zero_checkpoint = AgentDecision::new(
        AgentDecisionId::new("decision-zero-checkpoint")?,
        proposal_id,
        AgentDecisionOutcome::Rejected,
        DecisionReasonCode::new("policy_denied")?,
    )
    .with_checkpoint_seq(0);
    assert!(matches!(
        zero_checkpoint.validate(),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    Ok(())
}
