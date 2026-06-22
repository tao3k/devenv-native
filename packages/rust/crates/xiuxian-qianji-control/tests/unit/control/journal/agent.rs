use std::error::Error;
use std::io;

use xiuxian_qianji_control::{
    ActivityId, AgentDecision, AgentDecisionId, AgentDecisionJournalRecord, AgentDecisionOutcome,
    AgentJournalScope, AgentProposal, AgentProposalId, AgentProposalJournalRecord, ControlEvent,
    ControlEventKind, ControlLedger, DecisionReasonCode, InMemoryControlLedger, RunId, StepId,
    TokenId, record_agent_decision, record_agent_proposal,
};

use crate::control::support::artifact_ref;

#[test]
fn in_memory_ledger_replays_agent_proposal_and_decision_events() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-agent-journal")?;
    let step_id = StepId::new("stage-tool")?;
    let activity_id = ActivityId::new("activity-tool-web-fetch")?;
    let proposal_id = AgentProposalId::new("proposal-web-fetch")?;
    let decision_id = AgentDecisionId::new("decision-web-fetch")?;

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "record agent facts".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        2,
        ControlEventKind::StepCreated {
            title: "Tool stage".to_owned(),
            required_evidence: Vec::new(),
            budget: None,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        3,
        ControlEventKind::AgentProposalRecorded {
            proposal: AgentProposal::new(
                proposal_id.clone(),
                step_id.clone(),
                TokenId::new("token-tool")?,
                "call_tool",
            )
            .with_tool_name("web.fetch")
            .with_tool_input_ref(artifact_ref("artifact-web-fetch-input")?),
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        4,
        ControlEventKind::AgentDecisionRecorded {
            decision: AgentDecision::new(
                decision_id.clone(),
                proposal_id.clone(),
                AgentDecisionOutcome::Accepted,
                DecisionReasonCode::new("tool_authorized")?,
            )
            .with_scheduled_activity_id(activity_id.clone()),
        },
    ))?;

    let view = ledger.load_run_view(&run_id)?;
    let step = view
        .steps
        .get(&step_id)
        .ok_or_else(|| io::Error::other("missing replayed step"))?;
    let proposal = step
        .agent_proposals
        .get(&proposal_id)
        .ok_or_else(|| io::Error::other("missing replayed proposal"))?;
    let decision = step
        .agent_decisions
        .get(&decision_id)
        .ok_or_else(|| io::Error::other("missing replayed decision"))?;

    assert_eq!(proposal.tool_name.as_deref(), Some("web.fetch"));
    assert_eq!(decision.outcome, AgentDecisionOutcome::Accepted);
    assert_eq!(decision.scheduled_activity_id.as_ref(), Some(&activity_id));
    assert!(
        step.activities.is_empty(),
        "Agent decision recording must not schedule activity lifecycle state"
    );

    Ok(())
}

#[test]
fn helper_records_step_scoped_agent_proposal_and_decision_events() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-agent-helper")?;
    let step_id = StepId::new("stage-tool-helper")?;
    let activity_id = ActivityId::new("activity-tool-helper")?;
    let proposal_id = AgentProposalId::new("proposal-helper")?;
    let decision_id = AgentDecisionId::new("decision-helper")?;

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "record agent facts through helpers".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        2,
        ControlEventKind::StepCreated {
            title: "Tool helper stage".to_owned(),
            required_evidence: Vec::new(),
            budget: None,
        },
    ))?;

    record_agent_proposal(
        &ledger,
        AgentProposalJournalRecord::new(
            run_id.clone(),
            AgentJournalScope::step(step_id.clone()),
            3,
            AgentProposal::new(
                proposal_id.clone(),
                step_id.clone(),
                TokenId::new("token-helper")?,
                "call_tool",
            )
            .with_tool_name("web.fetch")
            .with_tool_input_ref(artifact_ref("artifact-helper-input")?),
        ),
    )?;
    record_agent_decision(
        &ledger,
        AgentDecisionJournalRecord::new(
            run_id.clone(),
            AgentJournalScope::step(step_id.clone()),
            4,
            AgentDecision::new(
                decision_id.clone(),
                proposal_id.clone(),
                AgentDecisionOutcome::Accepted,
                DecisionReasonCode::new("tool_authorized")?,
            )
            .with_scheduled_activity_id(activity_id.clone()),
        ),
    )?;

    let view = ledger.load_run_view(&run_id)?;
    let step = view
        .steps
        .get(&step_id)
        .ok_or_else(|| io::Error::other("missing replayed step"))?;

    assert!(step.agent_proposals.contains_key(&proposal_id));
    assert!(step.agent_decisions.contains_key(&decision_id));
    assert!(
        step.activities.is_empty(),
        "recording helper must not schedule activity lifecycle state"
    );

    Ok(())
}

#[test]
fn agent_journal_record_into_event_preserves_scope_and_payloads() -> Result<(), Box<dyn Error>> {
    let run_id = RunId::new("run-agent-builder")?;
    let step_id = StepId::new("stage-agent-builder")?;
    let activity_id = ActivityId::new("activity-agent-builder")?;
    let proposal_id = AgentProposalId::new("proposal-agent-builder")?;
    let decision_id = AgentDecisionId::new("decision-agent-builder")?;

    let proposal_event = AgentProposalJournalRecord::new(
        run_id.clone(),
        AgentJournalScope::step(step_id.clone()),
        10,
        AgentProposal::new(
            proposal_id.clone(),
            step_id.clone(),
            TokenId::new("token-agent-builder")?,
            "call_tool",
        )
        .with_tool_name("web.fetch")
        .with_tool_input_ref(artifact_ref("artifact-agent-builder-input")?),
    )
    .into_event();

    assert_eq!(proposal_event.run_id, run_id);
    assert_eq!(proposal_event.step_id.as_ref(), Some(&step_id));
    assert!(matches!(
        proposal_event.kind,
        ControlEventKind::AgentProposalRecorded { proposal }
            if proposal.proposal_id == proposal_id
    ));

    let decision_event = AgentDecisionJournalRecord::new(
        proposal_event.run_id,
        AgentJournalScope::run(),
        11,
        AgentDecision::new(
            decision_id.clone(),
            proposal_id,
            AgentDecisionOutcome::Accepted,
            DecisionReasonCode::new("tool_authorized")?,
        )
        .with_scheduled_activity_id(activity_id.clone()),
    )
    .into_event();

    assert_eq!(decision_event.step_id, None);
    assert!(matches!(
        decision_event.kind,
        ControlEventKind::AgentDecisionRecorded { decision }
            if decision.decision_id == decision_id
                && decision.scheduled_activity_id.as_ref() == Some(&activity_id)
    ));

    Ok(())
}

#[test]
fn helper_records_run_scoped_agent_decision_event() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-agent-helper-run-scope")?;
    let proposal_id = AgentProposalId::new("proposal-run-scope")?;
    let decision_id = AgentDecisionId::new("decision-run-scope")?;
    let activity_id = ActivityId::new("activity-run-scope")?;

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "record run-scoped agent decision".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    record_agent_decision(
        &ledger,
        AgentDecisionJournalRecord::new(
            run_id.clone(),
            AgentJournalScope::run(),
            2,
            AgentDecision::new(
                decision_id.clone(),
                proposal_id,
                AgentDecisionOutcome::Accepted,
                DecisionReasonCode::new("tool_authorized")?,
            )
            .with_scheduled_activity_id(activity_id),
        ),
    )?;

    let view = ledger.load_run_view(&run_id)?;

    assert!(view.agent_decisions.contains_key(&decision_id));
    assert!(
        view.activities.is_empty(),
        "run-scoped decision recording must not schedule activity lifecycle state"
    );

    Ok(())
}

#[test]
fn helper_rejects_mismatched_step_scoped_agent_proposal() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-agent-helper-mismatch")?;
    let proposal_id = AgentProposalId::new("proposal-mismatch")?;
    let proposal_step_id = StepId::new("proposal-step")?;
    let scope_step_id = StepId::new("scope-step")?;

    let Err(error) = record_agent_proposal(
        &ledger,
        AgentProposalJournalRecord::new(
            run_id.clone(),
            AgentJournalScope::step(scope_step_id),
            1,
            AgentProposal::new(
                proposal_id,
                proposal_step_id,
                TokenId::new("token-mismatch")?,
                "call_tool",
            ),
        ),
    ) else {
        return Err(io::Error::other("mismatched proposal scope should fail").into());
    };

    assert!(
        error.to_string().contains("does not match proposal step"),
        "unexpected error: {error}"
    );
    assert!(ledger.load_events(&run_id)?.is_empty());

    Ok(())
}
