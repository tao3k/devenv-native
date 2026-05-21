use std::io;

use xiuxian_qianji_control::AgentDecision;

use super::serde_status;

pub(crate) fn render_agent_decision_text(decision: &AgentDecision) -> String {
    let scheduled_activity = decision
        .scheduled_activity_id
        .as_ref()
        .map_or("<none>", |activity_id| activity_id.as_str());
    let checkpoint_seq = decision
        .checkpoint_seq
        .map_or_else(|| "<none>".to_string(), |sequence| sequence.to_string());
    let gate_name = decision
        .gate_result
        .as_ref()
        .map_or("<none>", |gate| gate.gate_name.as_str());
    let gate_passed = decision
        .gate_result
        .as_ref()
        .map_or("<none>".to_string(), |gate| gate.passed.to_string());
    let gate_evidence_covered = decision
        .gate_result
        .as_ref()
        .map_or("<none>".to_string(), |gate| {
            gate.required_evidence_covered.to_string()
        });
    let gate_selected_count = decision
        .gate_result
        .as_ref()
        .map_or(0, |gate| gate.selected_required_evidence.len());
    let gate_missing_count = decision
        .gate_result
        .as_ref()
        .map_or(0, |gate| gate.missing_required_evidence.len());

    format!(
        concat!(
            "# Qianji Control Decision\n\n",
            "- Decision: `{}`\n",
            "- Proposal: `{}`\n",
            "- Outcome: `{}`\n",
            "- Reason code: `{}`\n",
            "- Scheduled activity: `{}`\n",
            "- Checkpoint seq: `{}`\n",
            "- Gate: `{}`\n",
            "- Gate passed: `{}`\n",
            "- Gate evidence covered: `{}`\n",
            "- Gate selected evidence: `{}`\n",
            "- Gate missing evidence: `{}`\n"
        ),
        decision.decision_id.as_str(),
        decision.proposal_id.as_str(),
        serde_status(&decision.outcome),
        decision.reason_code.as_str(),
        scheduled_activity,
        checkpoint_seq,
        gate_name,
        gate_passed,
        gate_evidence_covered,
        gate_selected_count,
        gate_missing_count
    )
}

pub(crate) fn render_agent_decision_json(decision: &AgentDecision) -> io::Result<String> {
    serde_json::to_string_pretty(decision).map_err(io::Error::other)
}
