use super::support::{activity_task, record, run_id, step_id};
use crate::QianjiControlRunStreamSource;
use crate::bpmn::qianji_control_run_stream_rows;
use xiuxian_qianji_control::{
    ActivityId, ActivityResult, AgentDecision, AgentDecisionId, AgentDecisionOutcome,
    AgentProposal, AgentProposalId, ControlEvent, ControlEventKind, CostObservation,
    DecisionReasonCode, EvidenceId, EvidenceRef, GateName, GateResult, TokenId,
};

#[test]
fn qianji_control_run_stream_rows_use_structured_llm_routes() {
    let run_id = run_id();
    let events = vec![record(
        1,
        ControlEvent::run(
            run_id.clone(),
            10,
            ControlEventKind::ActivityScheduled {
                task: activity_task("bpmn-llm-plan", "tool.run", "default"),
            },
        ),
    )];

    let rows = qianji_control_run_stream_rows(&run_id, &events);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].source, QianjiControlRunStreamSource::Llm);
}

#[test]
fn qianji_control_run_stream_rows_use_structured_agent_routes() {
    let run_id = run_id();
    let events = vec![record(
        1,
        ControlEvent::run(
            run_id.clone(),
            10,
            ControlEventKind::ActivityScheduled {
                task: activity_task("agent-plan", "agent.plan", "default"),
            },
        ),
    )];

    let rows = qianji_control_run_stream_rows(&run_id, &events);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].source, QianjiControlRunStreamSource::Subagent);
}

#[test]
fn qianji_control_run_stream_rows_do_not_scan_metadata_text_for_agent_lane() {
    let run_id = run_id();
    let mut task = activity_task("tool-note", "tool.run", "default");
    task.metadata = serde_json::json!({
        "not_subagent_key": "contains subagent as unstructured text",
        "nested": { "note": "pi-subagents appears in prose only" },
    });
    let events = vec![record(
        1,
        ControlEvent::run(
            run_id.clone(),
            10,
            ControlEventKind::ActivityScheduled { task },
        ),
    )];

    let rows = qianji_control_run_stream_rows(&run_id, &events);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].source, QianjiControlRunStreamSource::Tool);
}

#[test]
fn qianji_control_run_stream_rows_render_activity_result_summaries() {
    let run_id = run_id();
    let activity_id = ActivityId::new("bpmn.instance.process.resolve_project.1")
        .unwrap_or_else(|error| panic!("activity id should be valid: {error}"));
    let events = vec![record(
        1,
        ControlEvent::step(
            run_id.clone(),
            step_id("resolve_project"),
            10,
            ControlEventKind::ActivityCompleted {
                activity_id,
                result: ActivityResult {
                    output_ref: None,
                    output_hash: None,
                    metadata: serde_json::json!({
                        "qianji_bpmn_host_work_completion": {
                            "data": {
                                "resolvedProject": true
                            }
                        }
                    }),
                },
            },
        ),
    )];

    let rows = qianji_control_run_stream_rows(&run_id, &events);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].source, QianjiControlRunStreamSource::Tool);
    assert_eq!(rows[0].element_id.as_deref(), Some("resolve_project"));
    assert_eq!(
        rows[0].message,
        "host work completed: {\"resolvedProject\":true}"
    );
}

#[test]
fn qianji_control_run_stream_rows_render_agent_rows_for_ui() {
    let run_id = run_id();
    let step_id = step_id("research");
    let proposal_id = AgentProposalId::new("proposal-1")
        .unwrap_or_else(|error| panic!("proposal id should be valid: {error}"));
    let proposal = AgentProposal::new(
        proposal_id.clone(),
        step_id.clone(),
        TokenId::new("token-1").unwrap_or_else(|error| panic!("token id should be valid: {error}")),
        "call_tool",
    )
    .with_tool_name("wendao.search")
    .with_confidence_millis(920);
    let decision = AgentDecision::new(
        AgentDecisionId::new("decision-1")
            .unwrap_or_else(|error| panic!("decision id should be valid: {error}")),
        proposal_id,
        AgentDecisionOutcome::Accepted,
        DecisionReasonCode::new("policy_passed")
            .unwrap_or_else(|error| panic!("reason code should be valid: {error}")),
    )
    .with_scheduled_activity_id(
        ActivityId::new("bpmn.instance.process.research.1")
            .unwrap_or_else(|error| panic!("activity id should be valid: {error}")),
    );
    let events = vec![
        record(
            1,
            ControlEvent::step(
                run_id.clone(),
                step_id.clone(),
                10,
                ControlEventKind::AgentProposalRecorded { proposal },
            ),
        ),
        record(
            2,
            ControlEvent::step(
                run_id.clone(),
                step_id,
                11,
                ControlEventKind::AgentDecisionRecorded { decision },
            ),
        ),
    ];

    let rows = qianji_control_run_stream_rows(&run_id, &events);

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].source, QianjiControlRunStreamSource::Subagent);
    assert_eq!(rows[0].element_id.as_deref(), Some("research"));
    assert_eq!(
        rows[0].message,
        "agent proposed call_tool with wendao.search"
    );
    assert_eq!(
        rows[0]
            .metadata
            .get("proposed_action")
            .and_then(serde_json::Value::as_str),
        Some("call_tool")
    );
    assert_eq!(rows[1].source, QianjiControlRunStreamSource::Subagent);
    assert_eq!(
        rows[1].message,
        "agent decision accepted: policy_passed; scheduled bpmn.instance.process.research.1"
    );
    assert_eq!(
        rows[1]
            .metadata
            .get("scheduled_activity_id")
            .and_then(serde_json::Value::as_str),
        Some("bpmn.instance.process.research.1")
    );
}

#[test]
fn qianji_control_run_stream_rows_render_precision_observation_rows_for_ui() {
    let run_id = run_id();
    let step_id = step_id("validate");
    let events = vec![
        record(
            1,
            ControlEvent::step(
                run_id.clone(),
                step_id.clone(),
                10,
                ControlEventKind::EvidenceAttached {
                    evidence: EvidenceRef {
                        evidence_id: EvidenceId::new("evidence-1")
                            .unwrap_or_else(|error| panic!("evidence id should be valid: {error}")),
                        requirement_key: Some("validation_path".to_owned()),
                        source: "wendao.search".to_owned(),
                        uri: None,
                        summary: Some("package test evidence passed".to_owned()),
                        metadata: serde_json::Value::Null,
                    },
                },
            ),
        ),
        record(
            2,
            ControlEvent::step(
                run_id.clone(),
                step_id.clone(),
                11,
                ControlEventKind::GateEvaluated {
                    result: GateResult {
                        gate_name: GateName::new("required_evidence")
                            .unwrap_or_else(|error| panic!("gate name should be valid: {error}")),
                        passed: false,
                        required_evidence_covered: false,
                        selected_required_evidence: vec!["ownership_boundary".to_owned()],
                        missing_required_evidence: vec!["validation_path".to_owned()],
                        reasons: vec!["validation path missing".to_owned()],
                        metadata: serde_json::Value::Null,
                    },
                },
            ),
        ),
        record(
            3,
            ControlEvent::step(
                run_id.clone(),
                step_id,
                12,
                ControlEventKind::CostObserved {
                    observation: CostObservation {
                        provider: "deepseek".to_owned(),
                        model: Some("deepseek-chat".to_owned()),
                        prompt_tokens: 100,
                        completion_tokens: 23,
                        total_tokens: None,
                        cost_usd_micros: 456,
                        latency_ms: Some(789),
                    },
                },
            ),
        ),
    ];

    let rows = qianji_control_run_stream_rows(&run_id, &events);

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].source, QianjiControlRunStreamSource::System);
    assert_eq!(
        rows[0].message,
        "evidence attached for validation_path from wendao.search: package test evidence passed"
    );
    assert_eq!(
        rows[1].message,
        "gate required_evidence failed: required evidence missing (1 selected, 1 missing)"
    );
    assert_eq!(
        rows[2].message,
        "cost observed: deepseek/deepseek-chat · 123 tokens · 456 usd_micros · 789 ms"
    );
}
