use xiuxian_qianji_control::{
    ActivityFailure, ActivityId, ActivityResult, ActivityTask, ActivityType, AgentDecision,
    AgentDecisionId, AgentDecisionOutcome, AgentProposalId, ControlEvent, ControlEventKind,
    ControlLedger, CostObservation, DecisionReasonCode, DuckDbControlLedger, ErrorCode, GateName,
    GateResult, IdempotencyKey, LeaseId, RunId, SignalName, StepId, StepLease, TaskQueue, TimerId,
    TimerRecord, WorkerId,
};

pub(super) fn to_args(values: &[&str]) -> Vec<String> {
    values.iter().map(ToString::to_string).collect()
}

pub(super) fn must_ok<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) -> T {
    result.unwrap_or_else(|error| panic!("{context}: {error}"))
}

pub(super) fn must_some<T>(value: Option<T>, context: &str) -> T {
    value.unwrap_or_else(|| panic!("{context}"))
}

pub(super) fn append_empty_control_run(ledger_path: &std::path::Path) -> RunId {
    let ledger = must_ok(
        DuckDbControlLedger::open(ledger_path),
        "should open temporary control ledger",
    );
    let run_id = must_ok(RunId::new("run-control-cli"), "should build control run id");
    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            1,
            ControlEventKind::RunCreated {
                intent: "test qianji control recovery snapshot".to_string(),
                budget: None,
                metadata: serde_json::Value::Null,
            },
        )),
        "should append run-created event",
    );
    run_id
}

pub(super) fn append_control_run_with_step(ledger_path: &std::path::Path) -> RunId {
    let run_id = append_empty_control_run(ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(ledger_path),
        "should reopen temporary control ledger",
    );
    let step_id = must_ok(
        StepId::new("run-control-step"),
        "should build control step id",
    );
    must_ok(
        ledger.append_event(ControlEvent::step(
            run_id.clone(),
            step_id,
            2,
            ControlEventKind::StepCreated {
                title: "Review durable state".to_string(),
                required_evidence: vec!["history_visible".to_string()],
                budget: None,
            },
        )),
        "should append step-created event",
    );
    run_id
}

pub(super) fn append_control_run_with_active_step_lease(ledger_path: &std::path::Path) -> RunId {
    let run_id = append_control_run_with_step(ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(ledger_path),
        "should reopen temporary control ledger",
    );
    let step_id = must_ok(
        StepId::new("run-control-step"),
        "should build control step id",
    );
    let lease = StepLease {
        lease_id: must_ok(LeaseId::new("lease-control-step"), "should build lease id"),
        run_id: run_id.clone(),
        step_id: step_id.clone(),
        worker_id: must_ok(WorkerId::new("worker-control"), "should build worker id"),
        acquired_at_ms: 10_000,
        expires_at_ms: 20_000,
    };

    must_ok(
        ledger.append_event(ControlEvent::step(
            run_id.clone(),
            step_id,
            10_000,
            ControlEventKind::StepLeaseAcquired { lease },
        )),
        "should append step lease acquired event",
    );
    run_id
}

pub(super) fn append_control_run_with_run_activity(ledger_path: &std::path::Path) -> RunId {
    let run_id = append_empty_control_run(ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(ledger_path),
        "should reopen temporary control ledger",
    );
    let activity_id = must_ok(
        ActivityId::new("activity-run-search"),
        "should build run activity id",
    );
    let worker_id = must_ok(WorkerId::new("worker-search"), "should build worker id");

    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            2,
            ControlEventKind::ActivityScheduled {
                task: activity_task(activity_id.clone(), "wendao.search", "wendao.search"),
            },
        )),
        "should append run activity schedule",
    );
    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            3,
            ControlEventKind::ActivityStarted {
                activity_id: activity_id.clone(),
                worker_id: Some(worker_id),
                attempt: 1,
            },
        )),
        "should append run activity start",
    );
    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            4,
            ControlEventKind::ActivityCompleted {
                activity_id,
                result: ActivityResult {
                    output_ref: None,
                    output_hash: Some("sha256:run-search-output".to_string()),
                    metadata: serde_json::Value::Null,
                },
            },
        )),
        "should append run activity completion",
    );
    run_id
}

pub(super) fn append_control_run_with_step_activity(ledger_path: &std::path::Path) -> RunId {
    let run_id = append_control_run_with_step(ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(ledger_path),
        "should reopen temporary control ledger",
    );
    let step_id = must_ok(
        StepId::new("run-control-step"),
        "should build control step id",
    );
    let activity_id = must_ok(
        ActivityId::new("activity-step-llm"),
        "should build step activity id",
    );

    must_ok(
        ledger.append_event(ControlEvent::step(
            run_id.clone(),
            step_id.clone(),
            3,
            ControlEventKind::ActivityScheduled {
                task: activity_task(activity_id.clone(), "llm.plan", "llm.openai"),
            },
        )),
        "should append step activity schedule",
    );
    must_ok(
        ledger.append_event(ControlEvent::step(
            run_id.clone(),
            step_id,
            4,
            ControlEventKind::ActivityFailed {
                activity_id,
                failure: ActivityFailure {
                    error_code: must_ok(
                        ErrorCode::new("rate_limited"),
                        "should build activity error code",
                    ),
                    message: "provider rejected request".to_string(),
                    retryable: true,
                    attempt: 2,
                    metadata: serde_json::Value::Null,
                },
            },
        )),
        "should append step activity failure",
    );
    run_id
}

pub(super) fn append_control_run_with_scheduled_activity_queue(
    ledger_path: &std::path::Path,
) -> RunId {
    let run_id = append_control_run_with_step(ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(ledger_path),
        "should reopen temporary control ledger",
    );
    let step_id = must_ok(
        StepId::new("run-control-step"),
        "should build control step id",
    );
    let scheduled_run_activity = must_ok(
        ActivityId::new("activity-run-scheduled"),
        "should build scheduled run activity id",
    );
    let started_activity = must_ok(
        ActivityId::new("activity-run-started"),
        "should build started run activity id",
    );
    let scheduled_step_activity = must_ok(
        ActivityId::new("activity-step-scheduled"),
        "should build scheduled step activity id",
    );

    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            3,
            ControlEventKind::ActivityScheduled {
                task: activity_task(scheduled_run_activity, "llm.plan", "llm.openai"),
            },
        )),
        "should append scheduled run activity",
    );
    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            4,
            ControlEventKind::ActivityScheduled {
                task: activity_task(started_activity.clone(), "llm.plan", "llm.openai"),
            },
        )),
        "should append started run activity schedule",
    );
    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            5,
            ControlEventKind::ActivityStarted {
                activity_id: started_activity,
                worker_id: None,
                attempt: 1,
            },
        )),
        "should append started run activity",
    );
    must_ok(
        ledger.append_event(ControlEvent::step(
            run_id.clone(),
            step_id,
            6,
            ControlEventKind::ActivityScheduled {
                task: activity_task(scheduled_step_activity, "tool.github", "tool.github"),
            },
        )),
        "should append scheduled step activity",
    );
    run_id
}

pub(super) fn append_control_run_with_llm_activity_inventory(
    ledger_path: &std::path::Path,
) -> RunId {
    let run_id = append_control_run_with_step(ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(ledger_path),
        "should reopen temporary control ledger",
    );
    let step_id = must_ok(
        StepId::new("run-control-step"),
        "should build control step id",
    );
    let completed_activity = must_ok(
        ActivityId::new("activity-run-llm-plan"),
        "should build completed llm activity id",
    );
    let missing_audit_activity = must_ok(
        ActivityId::new("activity-step-llm-repair"),
        "should build missing-audit llm activity id",
    );

    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            3,
            ControlEventKind::ActivityScheduled {
                task: llm_activity_task(completed_activity.clone(), "llm.plan", "llm.openai"),
            },
        )),
        "should append scheduled audited llm activity",
    );
    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            4,
            ControlEventKind::ActivityStarted {
                activity_id: completed_activity.clone(),
                worker_id: None,
                attempt: 1,
            },
        )),
        "should append started audited llm activity",
    );
    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            5,
            ControlEventKind::ActivityCompleted {
                activity_id: completed_activity,
                result: ActivityResult {
                    output_ref: None,
                    output_hash: Some("sha256:llm-plan-output".to_string()),
                    metadata: serde_json::Value::Null,
                },
            },
        )),
        "should append completed audited llm activity",
    );
    must_ok(
        ledger.append_event(ControlEvent::step(
            run_id.clone(),
            step_id,
            6,
            ControlEventKind::ActivityScheduled {
                task: activity_task(missing_audit_activity, "llm.repair", "llm.openai"),
            },
        )),
        "should append scheduled missing-audit llm activity",
    );

    run_id
}

pub(super) fn append_control_run_with_run_decision(ledger_path: &std::path::Path) -> RunId {
    let run_id = append_empty_control_run(ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(ledger_path),
        "should reopen temporary control ledger",
    );
    let decision_id = must_ok(
        AgentDecisionId::new("decision-run-search"),
        "should build run decision id",
    );
    let proposal_id = must_ok(
        AgentProposalId::new("proposal-run-search"),
        "should build run proposal id",
    );
    let reason_code = must_ok(
        DecisionReasonCode::new("authorized"),
        "should build decision reason code",
    );
    let activity_id = must_ok(
        ActivityId::new("activity-run-search"),
        "should build scheduled activity id",
    );

    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            2,
            ControlEventKind::AgentDecisionRecorded {
                decision: AgentDecision::new(
                    decision_id,
                    proposal_id,
                    AgentDecisionOutcome::Accepted,
                    reason_code,
                )
                .with_scheduled_activity_id(activity_id)
                .with_checkpoint_seq(7),
            },
        )),
        "should append run decision",
    );
    run_id
}

pub(super) fn append_control_run_with_step_decision(ledger_path: &std::path::Path) -> RunId {
    let run_id = append_control_run_with_step(ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(ledger_path),
        "should reopen temporary control ledger",
    );
    let step_id = must_ok(
        StepId::new("run-control-step"),
        "should build control step id",
    );
    let decision_id = must_ok(
        AgentDecisionId::new("decision-step-approval"),
        "should build step decision id",
    );
    let proposal_id = must_ok(
        AgentProposalId::new("proposal-step-llm"),
        "should build step proposal id",
    );
    let reason_code = must_ok(
        DecisionReasonCode::new("approval_required"),
        "should build step decision reason code",
    );

    must_ok(
        ledger.append_event(ControlEvent::step(
            run_id.clone(),
            step_id,
            3,
            ControlEventKind::AgentDecisionRecorded {
                decision: AgentDecision::new(
                    decision_id,
                    proposal_id,
                    AgentDecisionOutcome::ApprovalRequired,
                    reason_code,
                )
                .with_gate_result(GateResult {
                    gate_name: must_ok(
                        GateName::new("required-evidence"),
                        "should build gate name",
                    ),
                    passed: false,
                    required_evidence_covered: false,
                    selected_required_evidence: vec!["history_visible".to_string()],
                    missing_required_evidence: vec!["approval_signal".to_string()],
                    reasons: vec!["human approval required".to_string()],
                    metadata: serde_json::Value::Null,
                }),
            },
        )),
        "should append step decision",
    );
    run_id
}

pub(super) fn append_control_run_with_run_timer(ledger_path: &std::path::Path) -> RunId {
    let run_id = append_empty_control_run(ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(ledger_path),
        "should reopen temporary control ledger",
    );
    let timer_id = must_ok(
        TimerId::new("timer-run-wakeup"),
        "should build run timer id",
    );

    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            2,
            ControlEventKind::TimerScheduled {
                timer: TimerRecord {
                    timer_id: timer_id.clone(),
                    fire_at_ms: 10_000,
                    metadata: serde_json::Value::Null,
                },
            },
        )),
        "should append run timer schedule",
    );
    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            10_250,
            ControlEventKind::TimerFired { timer_id },
        )),
        "should append run timer fire",
    );
    run_id
}

pub(super) fn append_control_run_with_step_timer(ledger_path: &std::path::Path) -> RunId {
    let run_id = append_control_run_with_step(ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(ledger_path),
        "should reopen temporary control ledger",
    );
    let step_id = must_ok(
        StepId::new("run-control-step"),
        "should build control step id",
    );
    let timer_id = must_ok(
        TimerId::new("timer-step-approval-timeout"),
        "should build step timer id",
    );

    must_ok(
        ledger.append_event(ControlEvent::step(
            run_id.clone(),
            step_id,
            3,
            ControlEventKind::TimerScheduled {
                timer: TimerRecord {
                    timer_id,
                    fire_at_ms: 20_000,
                    metadata: serde_json::Value::Null,
                },
            },
        )),
        "should append step timer schedule",
    );
    run_id
}

pub(super) fn append_control_run_with_run_and_step_timers(ledger_path: &std::path::Path) -> RunId {
    let run_id = append_control_run_with_run_timer(ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(ledger_path),
        "should reopen temporary control ledger",
    );
    let step_id = must_ok(
        StepId::new("run-control-step"),
        "should build control step id",
    );
    let timer_id = must_ok(
        TimerId::new("timer-step-approval-timeout"),
        "should build step timer id",
    );

    must_ok(
        ledger.append_event(ControlEvent::step(
            run_id.clone(),
            step_id.clone(),
            11_000,
            ControlEventKind::StepCreated {
                title: "Wait for approval".to_string(),
                required_evidence: vec!["approval_signal".to_string()],
                budget: None,
            },
        )),
        "should append step-created event",
    );
    must_ok(
        ledger.append_event(ControlEvent::step(
            run_id.clone(),
            step_id,
            12_000,
            ControlEventKind::TimerScheduled {
                timer: TimerRecord {
                    timer_id,
                    fire_at_ms: 20_000,
                    metadata: serde_json::Value::Null,
                },
            },
        )),
        "should append step timer schedule",
    );
    run_id
}

pub(super) fn append_control_run_with_step_signal_and_timer(
    ledger_path: &std::path::Path,
) -> RunId {
    let run_id = append_control_run_with_step_timer(ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(ledger_path),
        "should reopen temporary control ledger",
    );
    let step_id = must_ok(
        StepId::new("run-control-step"),
        "should build control step id",
    );

    must_ok(
        ledger.append_event(ControlEvent::step(
            run_id.clone(),
            step_id,
            12_000,
            ControlEventKind::SignalReceived {
                signal: xiuxian_qianji_control::SignalRecord {
                    signal_name: must_ok(
                        SignalName::new("human.approval"),
                        "should build signal name",
                    ),
                    payload_ref: None,
                    payload_hash: None,
                    metadata: serde_json::json!({"approved": true}),
                },
            },
        )),
        "should append step signal",
    );
    run_id
}

pub(super) fn append_control_run_with_run_and_step_signals(ledger_path: &std::path::Path) -> RunId {
    let run_id = append_control_run_with_step(ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(ledger_path),
        "should reopen temporary control ledger",
    );
    let step_id = must_ok(
        StepId::new("run-control-step"),
        "should build control step id",
    );

    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            11_000,
            ControlEventKind::SignalReceived {
                signal: xiuxian_qianji_control::SignalRecord {
                    signal_name: must_ok(
                        SignalName::new("run.refresh"),
                        "should build run signal name",
                    ),
                    payload_ref: None,
                    payload_hash: None,
                    metadata: serde_json::json!({"reason": "manual"}),
                },
            },
        )),
        "should append run signal",
    );
    must_ok(
        ledger.append_event(ControlEvent::step(
            run_id.clone(),
            step_id,
            12_000,
            ControlEventKind::SignalReceived {
                signal: xiuxian_qianji_control::SignalRecord {
                    signal_name: must_ok(
                        SignalName::new("human.approval"),
                        "should build step signal name",
                    ),
                    payload_ref: None,
                    payload_hash: None,
                    metadata: serde_json::json!({"approved": true}),
                },
            },
        )),
        "should append step signal",
    );
    run_id
}

pub(super) fn append_control_run_with_run_and_step_costs(ledger_path: &std::path::Path) -> RunId {
    let run_id = append_control_run_with_step(ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(ledger_path),
        "should reopen temporary control ledger",
    );
    let step_id = must_ok(
        StepId::new("run-control-step"),
        "should build control step id",
    );

    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            11_000,
            ControlEventKind::CostObserved {
                observation: cost_observation("llm.openai", Some("gpt-test"), 10, 20, 100, 1_000),
            },
        )),
        "should append run cost",
    );
    must_ok(
        ledger.append_event(ControlEvent::step(
            run_id.clone(),
            step_id,
            12_000,
            ControlEventKind::CostObserved {
                observation: cost_observation("tool.github", None, 5, 7, 30, 250),
            },
        )),
        "should append step cost",
    );
    run_id
}

pub(super) fn append_control_run_with_operator_summary_facts(
    ledger_path: &std::path::Path,
) -> RunId {
    let run_id = append_control_run_with_active_step_lease(ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(ledger_path),
        "should reopen temporary control ledger",
    );
    let step_id = must_ok(
        StepId::new("run-control-step"),
        "should build control step id",
    );
    let activity_id = must_ok(
        ActivityId::new("activity-summary-scheduled"),
        "should build summary activity id",
    );
    let timer_id = must_ok(
        TimerId::new("timer-summary-expired"),
        "should build summary timer id",
    );

    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            11_000,
            ControlEventKind::ActivityScheduled {
                task: activity_task(activity_id, "llm.plan", "llm.openai"),
            },
        )),
        "should append summary activity",
    );
    must_ok(
        ledger.append_event(ControlEvent::step(
            run_id.clone(),
            step_id.clone(),
            12_000,
            ControlEventKind::TimerScheduled {
                timer: TimerRecord {
                    timer_id,
                    fire_at_ms: 13_000,
                    metadata: serde_json::Value::Null,
                },
            },
        )),
        "should append summary timer",
    );
    must_ok(
        ledger.append_event(ControlEvent::step(
            run_id.clone(),
            step_id,
            12_500,
            ControlEventKind::SignalReceived {
                signal: xiuxian_qianji_control::SignalRecord {
                    signal_name: must_ok(
                        SignalName::new("human.approval"),
                        "should build summary signal name",
                    ),
                    payload_ref: None,
                    payload_hash: None,
                    metadata: serde_json::json!({"approved": true}),
                },
            },
        )),
        "should append summary signal",
    );
    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            12_750,
            ControlEventKind::CostObserved {
                observation: cost_observation("llm.openai", Some("gpt-test"), 10, 20, 100, 1_250),
            },
        )),
        "should append summary cost",
    );
    run_id
}

fn cost_observation(
    provider: &str,
    model: Option<&str>,
    prompt_tokens: u64,
    completion_tokens: u64,
    cost_usd_micros: u64,
    latency_ms: u64,
) -> CostObservation {
    CostObservation {
        provider: provider.to_owned(),
        model: model.map(str::to_owned),
        prompt_tokens,
        completion_tokens,
        total_tokens: None,
        cost_usd_micros,
        latency_ms: Some(latency_ms),
    }
}

pub(super) fn activity_task(
    activity_id: ActivityId,
    activity_type: &str,
    task_queue: &str,
) -> ActivityTask {
    ActivityTask::new(
        activity_id,
        must_ok(
            ActivityType::new(activity_type),
            "should build activity type",
        ),
        must_ok(TaskQueue::new(task_queue), "should build task queue"),
        must_ok(
            IdempotencyKey::new("activity-idempotency-key"),
            "should build idempotency key",
        ),
    )
    .with_timeout_ms(30_000)
}

fn llm_activity_task(
    activity_id: ActivityId,
    activity_type: &str,
    task_queue: &str,
) -> ActivityTask {
    let mut task = activity_task(activity_id, activity_type, task_queue);
    task.metadata = serde_json::json!({
        "qianji_llm_activity_request": {
            "model": "openai/gpt-5-mini",
            "prompt_ref": "artifact://prompt/plan",
            "context_ref": "artifact://context/plan",
            "tool_schema_hash": "sha256:tool-schema",
            "response_schema_ref": "artifact://schema/agent-proposal",
            "temperature": "0.0",
            "max_tokens": 1024,
            "budget": {
                "max_cost_usd_micros": 2500
            }
        }
    });
    task
}
