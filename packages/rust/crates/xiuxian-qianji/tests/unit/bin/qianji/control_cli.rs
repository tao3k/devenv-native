use super::{
    ControlCliCommand, TempDir, must_ok, must_some, parse_control_command, run_control_command,
    to_args,
};
use xiuxian_qianji_control::{
    ActivityFailure, ActivityId, ActivityResult, ActivityTask, ActivityType, AgentDecision,
    AgentDecisionId, AgentDecisionOutcome, AgentProposalId, ControlEvent, ControlEventKind,
    ControlLedger, DecisionReasonCode, DuckDbControlLedger, ErrorCode, GateName, GateResult,
    IdempotencyKey, RunId, SignalName, StepId, TaskQueue, TimerId, TimerRecord, WorkerId,
};

#[test]
fn parse_control_activity_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "activity",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--step-id",
                    "run-control-step",
                    "--activity-id",
                    "activity-doc",
                    "--json",
                ])),
                "control activity parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Activity {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: Some("run-control-step".to_string()),
            activity_id: "activity-doc".to_string(),
            json: true,
        },
    );
}

#[test]
fn parse_control_activity_command_without_step_scope() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "activity",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--activity-id",
                    "activity-run",
                ])),
                "control activity parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Activity {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: None,
            activity_id: "activity-run".to_string(),
            json: false,
        },
    );
}

#[test]
fn parse_control_activity_queue_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "activity-queue",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--task-queue",
                    "llm.openai",
                    "--json",
                ])),
                "control activity-queue parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::ActivityQueue {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            task_queue: Some("llm.openai".to_string()),
            json: true,
        },
    );
}

#[test]
fn parse_control_activity_queue_rejects_missing_ledger() {
    let result = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "activity-queue",
        "--run-id",
        "run-control",
    ]));
    let error = match result {
        Ok(value) => panic!("missing activity queue ledger should fail, got {value:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing `--ledger <path>` for `control activity-queue`"),
        "unexpected error: {error}"
    );
}

#[test]
fn parse_control_apply_recovery_plan_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "apply-recovery-plan",
                    "--ledger",
                    "control.duckdb",
                    "--valkey-url",
                    "redis://127.0.0.1:6379",
                    "--namespace",
                    "qianji:test",
                    "--run-id",
                    "run-control",
                    "--now-ms",
                    "12345",
                    "--attempt",
                    "2",
                    "--reason",
                    "operator recovery",
                    "--max-attempts",
                    "5",
                    "--backoff-ms",
                    "250",
                    "--require-human-approval",
                    "--priority",
                    "9",
                    "--json",
                ])),
                "control apply-recovery-plan parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::ApplyRecoveryPlan {
            ledger_path: "control.duckdb".into(),
            valkey_url: "redis://127.0.0.1:6379".to_string(),
            namespace: Some("qianji:test".to_string()),
            run_id: "run-control".to_string(),
            now_ms: 12_345,
            attempt: 2,
            reason: "operator recovery".to_string(),
            max_attempts: 5,
            backoff_ms: 250,
            require_human_approval: true,
            priority: 9,
            json: true,
        },
    );
}

#[test]
fn parse_control_apply_recovery_plan_rejects_missing_valkey_url() {
    let result = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "apply-recovery-plan",
        "--ledger",
        "control.duckdb",
        "--run-id",
        "run-control",
        "--now-ms",
        "12345",
        "--attempt",
        "1",
        "--reason",
        "operator recovery",
        "--max-attempts",
        "3",
    ]));
    let error = match result {
        Ok(value) => panic!("missing recovery Valkey URL should fail, got {value:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing `--valkey-url <url>` for `control apply-recovery-plan`"),
        "unexpected error: {error}"
    );
}

#[test]
fn parse_control_decision_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "decision",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--step-id",
                    "run-control-step",
                    "--decision-id",
                    "decision-doc",
                    "--json",
                ])),
                "control decision parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Decision {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: Some("run-control-step".to_string()),
            decision_id: "decision-doc".to_string(),
            json: true,
        },
    );
}

#[test]
fn parse_control_decision_command_without_step_scope() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "decision",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--decision-id",
                    "decision-run",
                ])),
                "control decision parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Decision {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: None,
            decision_id: "decision-run".to_string(),
            json: false,
        },
    );
}

#[test]
fn parse_control_timer_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "timer",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--step-id",
                    "run-control-step",
                    "--timer-id",
                    "timer-doc",
                    "--json",
                ])),
                "control timer parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Timer {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: Some("run-control-step".to_string()),
            timer_id: "timer-doc".to_string(),
            json: true,
        },
    );
}

#[test]
fn parse_control_timer_command_without_step_scope() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "timer",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--timer-id",
                    "timer-run",
                ])),
                "control timer parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Timer {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: None,
            timer_id: "timer-run".to_string(),
            json: false,
        },
    );
}

#[test]
fn parse_control_signal_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "signal",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--step-id",
                    "run-control-step",
                    "--signal-name",
                    "human.approval",
                    "--payload",
                    r#"{"approved":true}"#,
                    "--received-at-ms",
                    "12345",
                    "--json",
                ])),
                "control signal parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Signal {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: Some("run-control-step".to_string()),
            signal_name: "human.approval".to_string(),
            payload: r#"{"approved":true}"#.to_string(),
            received_at_ms: 12_345,
            json: true,
        },
    );
}

#[test]
fn parse_control_signal_command_without_step_scope() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "signal",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--signal-name",
                    "run.refresh",
                    "--payload",
                    r#"{"reason":"manual"}"#,
                    "--received-at-ms",
                    "54321",
                ])),
                "control signal parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Signal {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: None,
            signal_name: "run.refresh".to_string(),
            payload: r#"{"reason":"manual"}"#.to_string(),
            received_at_ms: 54_321,
            json: false,
        },
    );
}

#[test]
fn parse_control_heartbeat_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "heartbeat",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--worker-id",
                    "worker-a",
                    "--observed-at-ms",
                    "1000",
                    "--expires-at-ms",
                    "3000",
                    "--metadata",
                    r#"{"queue":"llm.openai"}"#,
                    "--json",
                ])),
                "control heartbeat parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Heartbeat {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            worker_id: "worker-a".to_string(),
            observed_at_ms: 1_000,
            expires_at_ms: 3_000,
            metadata: Some(r#"{"queue":"llm.openai"}"#.to_string()),
            json: true,
        },
    );
}

#[test]
fn parse_control_hot_state_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "hot-state",
                    "--valkey-url",
                    "redis://127.0.0.1:6379",
                    "--namespace",
                    "qianji:test",
                    "--now-ms",
                    "12345",
                    "--json",
                ])),
                "control hot-state parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::HotState {
            valkey_url: "redis://127.0.0.1:6379".to_string(),
            namespace: Some("qianji:test".to_string()),
            now_ms: 12_345,
            json: true,
        },
    );
}

#[test]
fn parse_control_hot_state_rejects_missing_now_ms() {
    let result = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "hot-state",
        "--valkey-url",
        "redis://127.0.0.1:6379",
    ]));
    let error = match result {
        Ok(value) => panic!("missing hot-state timestamp should fail, got {value:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing `--now-ms <ms>` for `control hot-state`"),
        "unexpected error: {error}"
    );
}

#[test]
fn parse_control_query_state_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "query",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--state",
                    "--now-ms",
                    "1234",
                    "--json",
                ])),
                "control query state parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::QueryState {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            now_ms: 1234,
            json: true,
        },
    );
}

#[test]
fn parse_control_query_rejects_missing_state_flag() {
    let Err(error) = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "query",
        "--ledger",
        "control.duckdb",
        "--run-id",
        "run-control",
        "--now-ms",
        "1234",
    ])) else {
        panic!("missing query kind should fail");
    };

    assert!(error.to_string().contains("missing `--state`"));
}

#[test]
fn parse_control_history_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "history",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--json",
                ])),
                "control history parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::History {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            json: true,
        },
    );
}

#[test]
fn parse_control_recovery_snapshot_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "recovery-snapshot",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--now-ms",
                    "1234",
                    "--json",
                ])),
                "control recovery snapshot parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::RecoverySnapshot {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            now_ms: 1234,
            json: true,
        },
    );
}

#[test]
fn parse_control_view_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "view",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--json",
                ])),
                "control view parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::View {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            json: true,
        },
    );
}

#[test]
fn parse_control_step_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "step",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--step-id",
                    "run-control-step",
                    "--json",
                ])),
                "control step parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Step {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: "run-control-step".to_string(),
            json: true,
        },
    );
}

#[test]
fn run_control_history_renders_json() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::History {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            json: true,
        }),
        "control history json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "history output should be valid json",
    );

    assert_eq!(json.as_array().map(Vec::len), Some(1));
    assert_eq!(json[0]["sequence"], 1);
    assert_eq!(json[0]["event"]["run_id"], "run-control-cli");
    assert_eq!(json[0]["event"]["kind"]["event"], "run_created");
    Ok(())
}

#[test]
fn run_control_history_renders_text_timeline() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::History {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            json: false,
        }),
        "control history text should render",
    );

    assert!(output.rendered.starts_with("# Qianji Control History"));
    assert!(output.rendered.contains("- Run: `run-control-cli`"));
    assert!(output.rendered.contains("- Events: `1`"));
    assert!(output.rendered.contains("- #1 @1 [run] `run_created`"));
    Ok(())
}

#[test]
fn run_control_view_renders_json() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::View {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            json: true,
        }),
        "control view json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "view output should be valid json",
    );

    assert_eq!(json["run_id"], "run-control-cli");
    assert_eq!(json["status"], "draft");
    assert_eq!(json["steps"].as_object().map(serde_json::Map::len), Some(1));
    assert_eq!(
        json["steps"]["run-control-step"]["title"],
        "Review durable state"
    );
    Ok(())
}

#[test]
fn run_control_view_renders_text_summary() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::View {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            json: false,
        }),
        "control view text should render",
    );

    assert!(output.rendered.starts_with("# Qianji Control View"));
    assert!(output.rendered.contains("- Run: `run-control-cli`"));
    assert!(output.rendered.contains("- Status: `draft`"));
    assert!(output.rendered.contains("- Steps: `1`"));
    assert!(
        output
            .rendered
            .contains("`run-control-step` [pending] Review durable state")
    );
    Ok(())
}

#[test]
fn run_control_step_renders_json() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Step {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            step_id: "run-control-step".to_string(),
            json: true,
        }),
        "control step json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "step output should be valid json",
    );

    assert_eq!(json["step_id"], "run-control-step");
    assert_eq!(json["status"], "pending");
    assert_eq!(json["title"], "Review durable state");
    assert_eq!(json["required_evidence"].as_array().map(Vec::len), Some(1));
    Ok(())
}

#[test]
fn run_control_step_renders_text_summary() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Step {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            step_id: "run-control-step".to_string(),
            json: false,
        }),
        "control step text should render",
    );

    assert!(output.rendered.starts_with("# Qianji Control Step"));
    assert!(output.rendered.contains("- Step: `run-control-step`"));
    assert!(output.rendered.contains("- Status: `pending`"));
    assert!(output.rendered.contains("- Required evidence: `1`"));
    assert!(output.rendered.contains("- Covered evidence: `0`"));
    Ok(())
}

#[test]
fn run_control_step_rejects_missing_step() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step(&ledger_path);

    let Err(error) = run_control_command(&ControlCliCommand::Step {
        ledger_path,
        run_id: run_id.as_str().to_string(),
        step_id: "missing-step".to_string(),
        json: false,
    }) else {
        return Err("missing step should fail".to_string());
    };

    assert!(
        error
            .to_string()
            .contains("could not find step `missing-step`")
    );
    Ok(())
}

#[test]
fn run_control_activity_renders_run_scope_json() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_run_activity(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Activity {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            step_id: None,
            activity_id: "activity-run-search".to_string(),
            json: true,
        }),
        "control activity json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity output should be valid json",
    );

    assert_eq!(json["activity_id"], "activity-run-search");
    assert_eq!(json["status"], "completed");
    assert_eq!(json["task"]["activity_type"], "wendao.search");
    assert_eq!(json["worker_id"], "worker-search");
    assert_eq!(json["attempt"], 1);
    assert_eq!(json["result"]["output_hash"], "sha256:run-search-output");
    Ok(())
}

#[test]
fn run_control_activity_renders_step_scope_text_summary() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step_activity(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Activity {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            step_id: Some("run-control-step".to_string()),
            activity_id: "activity-step-llm".to_string(),
            json: false,
        }),
        "control activity text should render",
    );

    assert!(output.rendered.starts_with("# Qianji Control Activity"));
    assert!(output.rendered.contains("- Activity: `activity-step-llm`"));
    assert!(output.rendered.contains("- Status: `failed`"));
    assert!(output.rendered.contains("- Activity type: `llm.plan`"));
    assert!(output.rendered.contains("- Task queue: `llm.openai`"));
    assert!(output.rendered.contains("- Attempt: `2`"));
    assert!(output.rendered.contains("- Failure code: `rate_limited`"));
    assert!(output.rendered.contains("- Failure retryable: `true`"));
    Ok(())
}

#[test]
fn run_control_activity_rejects_missing_activity() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_run_activity(&ledger_path);

    let Err(error) = run_control_command(&ControlCliCommand::Activity {
        ledger_path,
        run_id: run_id.as_str().to_string(),
        step_id: None,
        activity_id: "missing-activity".to_string(),
        json: false,
    }) else {
        return Err("missing activity should fail".to_string());
    };

    assert!(
        error
            .to_string()
            .contains("could not find activity `missing-activity`")
    );
    Ok(())
}

#[test]
fn run_control_activity_queue_renders_json_without_appending() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let before_count = must_ok(
        ledger.load_events(&run_id),
        "should load events before activity queue query",
    )
    .len();

    let output = must_ok(
        run_control_command(&ControlCliCommand::ActivityQueue {
            ledger_path: ledger_path.clone(),
            run_id: run_id.as_str().to_string(),
            task_queue: Some("llm.openai".to_string()),
            json: true,
        }),
        "control activity queue json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity queue output should be valid json",
    );
    let after_count = must_ok(
        ledger.load_events(&run_id),
        "should load events after activity queue query",
    )
    .len();

    assert_eq!(json["run_id"], "run-control-cli");
    assert_eq!(json["task_queue"], "llm.openai");
    assert_eq!(json["items"].as_array().map(Vec::len), Some(1));
    assert_eq!(
        json["items"][0]["activity"]["activity_id"],
        "activity-run-scheduled"
    );
    assert_eq!(before_count, after_count);
    Ok(())
}

#[test]
fn run_control_activity_queue_renders_text_summary() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::ActivityQueue {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            task_queue: None,
            json: false,
        }),
        "control activity queue text should render",
    );

    assert!(
        output
            .rendered
            .starts_with("# Qianji Control Activity Queue")
    );
    assert!(output.rendered.contains("- Run: `run-control-cli`"));
    assert!(output.rendered.contains("- Task queue: `<all>`"));
    assert!(output.rendered.contains("- Scheduled activities: `2`"));
    assert!(output.rendered.contains("`activity-run-scheduled` [run]"));
    assert!(
        output
            .rendered
            .contains("`activity-step-scheduled` [step:run-control-step]")
    );
    Ok(())
}

#[test]
fn run_control_decision_renders_run_scope_json() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_run_decision(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Decision {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            step_id: None,
            decision_id: "decision-run-search".to_string(),
            json: true,
        }),
        "control decision json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "decision output should be valid json",
    );

    assert_eq!(json["decision_id"], "decision-run-search");
    assert_eq!(json["proposal_id"], "proposal-run-search");
    assert_eq!(json["outcome"], "accepted");
    assert_eq!(json["reason_code"], "authorized");
    assert_eq!(json["scheduled_activity_id"], "activity-run-search");
    assert_eq!(json["checkpoint_seq"], 7);
    Ok(())
}

#[test]
fn run_control_decision_renders_step_scope_text_summary() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step_decision(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Decision {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            step_id: Some("run-control-step".to_string()),
            decision_id: "decision-step-approval".to_string(),
            json: false,
        }),
        "control decision text should render",
    );

    assert!(output.rendered.starts_with("# Qianji Control Decision"));
    assert!(
        output
            .rendered
            .contains("- Decision: `decision-step-approval`")
    );
    assert!(output.rendered.contains("- Proposal: `proposal-step-llm`"));
    assert!(output.rendered.contains("- Outcome: `approval_required`"));
    assert!(
        output
            .rendered
            .contains("- Reason code: `approval_required`")
    );
    assert!(output.rendered.contains("- Scheduled activity: `<none>`"));
    assert!(output.rendered.contains("- Gate: `required-evidence`"));
    assert!(output.rendered.contains("- Gate passed: `false`"));
    assert!(output.rendered.contains("- Gate missing evidence: `1`"));
    Ok(())
}

#[test]
fn run_control_decision_rejects_missing_decision() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_run_decision(&ledger_path);

    let Err(error) = run_control_command(&ControlCliCommand::Decision {
        ledger_path,
        run_id: run_id.as_str().to_string(),
        step_id: None,
        decision_id: "missing-decision".to_string(),
        json: false,
    }) else {
        return Err("missing decision should fail".to_string());
    };

    assert!(
        error
            .to_string()
            .contains("could not find decision `missing-decision`")
    );
    Ok(())
}

#[test]
fn run_control_timer_renders_run_scope_json() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_run_timer(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Timer {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            step_id: None,
            timer_id: "timer-run-wakeup".to_string(),
            json: true,
        }),
        "control timer json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "timer output should be valid json",
    );

    assert_eq!(json["timer_id"], "timer-run-wakeup");
    assert_eq!(json["status"], "fired");
    assert_eq!(json["timer"]["fire_at_ms"], 10_000);
    assert_eq!(json["fired_at_ms"], 10_250);
    Ok(())
}

#[test]
fn run_control_timer_renders_step_scope_text_summary() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step_timer(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Timer {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            step_id: Some("run-control-step".to_string()),
            timer_id: "timer-step-approval-timeout".to_string(),
            json: false,
        }),
        "control timer text should render",
    );

    assert!(output.rendered.starts_with("# Qianji Control Timer"));
    assert!(
        output
            .rendered
            .contains("- Timer: `timer-step-approval-timeout`")
    );
    assert!(output.rendered.contains("- Status: `scheduled`"));
    assert!(output.rendered.contains("- Fire at ms: `20000`"));
    assert!(output.rendered.contains("- Fired at ms: `<none>`"));
    Ok(())
}

#[test]
fn run_control_timer_rejects_missing_timer() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_run_timer(&ledger_path);

    let Err(error) = run_control_command(&ControlCliCommand::Timer {
        ledger_path,
        run_id: run_id.as_str().to_string(),
        step_id: None,
        timer_id: "missing-timer".to_string(),
        json: false,
    }) else {
        return Err("missing timer should fail".to_string());
    };

    assert!(
        error
            .to_string()
            .contains("could not find timer `missing-timer`")
    );
    Ok(())
}

#[test]
fn run_control_signal_appends_run_scope_json() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Signal {
            ledger_path: ledger_path.clone(),
            run_id: run_id.as_str().to_string(),
            step_id: None,
            signal_name: "run.refresh".to_string(),
            payload: r#"{"reason":"manual"}"#.to_string(),
            received_at_ms: 11_000,
            json: true,
        }),
        "control signal json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "signal output should be valid json",
    );
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let view = must_ok(
        ledger.load_run_view(&run_id),
        "signal append should replay into run view",
    );

    assert_eq!(json["sequence"], 2);
    assert_eq!(json["event"]["run_id"], "run-control-cli");
    assert_eq!(json["event"]["kind"]["event"], "signal_received");
    assert_eq!(
        json["event"]["kind"]["signal"]["signal_name"],
        "run.refresh"
    );
    assert_eq!(
        json["event"]["kind"]["signal"]["metadata"]["reason"],
        "manual"
    );
    assert_eq!(view.signals.len(), 1);
    assert_eq!(
        view.signals[0].signal_name,
        must_ok(SignalName::new("run.refresh"), "should build signal name")
    );
    Ok(())
}

#[test]
fn run_control_signal_appends_step_scope_text_and_replays() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Signal {
            ledger_path: ledger_path.clone(),
            run_id: run_id.as_str().to_string(),
            step_id: Some("run-control-step".to_string()),
            signal_name: "human.approval".to_string(),
            payload: r#"{"approved":true,"approver":"operator"}"#.to_string(),
            received_at_ms: 12_000,
            json: false,
        }),
        "control signal text should render",
    );
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let view = must_ok(
        ledger.load_run_view(&run_id),
        "signal append should replay into run view",
    );
    let step = must_some(
        view.steps.get(&must_ok(
            StepId::new("run-control-step"),
            "should build step id",
        )),
        "step signal should replay into step view",
    );

    assert!(output.rendered.starts_with("# Qianji Control Signal"));
    assert!(output.rendered.contains("- Run: `run-control-cli`"));
    assert!(output.rendered.contains("- Scope: `run-control-step`"));
    assert!(output.rendered.contains("- Signal: `human.approval`"));
    assert!(output.rendered.contains("- Received at ms: `12000`"));
    assert_eq!(step.signals.len(), 1);
    assert_eq!(
        step.signals[0].signal_name,
        must_ok(
            SignalName::new("human.approval"),
            "should build signal name"
        )
    );
    assert_eq!(step.signals[0].metadata["approved"], true);
    Ok(())
}

#[test]
fn run_control_signal_rejects_invalid_payload_without_append() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let Err(error) = run_control_command(&ControlCliCommand::Signal {
        ledger_path: ledger_path.clone(),
        run_id: run_id.as_str().to_string(),
        step_id: None,
        signal_name: "run.refresh".to_string(),
        payload: "{not-json".to_string(),
        received_at_ms: 11_000,
        json: false,
    }) else {
        return Err("invalid signal payload should fail".to_string());
    };
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let records = must_ok(
        ledger.load_events(&run_id),
        "invalid payload should not append an event",
    );

    assert!(
        error
            .to_string()
            .contains("invalid `--payload` JSON for `control signal`")
    );
    assert_eq!(records.len(), 1);
    Ok(())
}

#[test]
fn run_control_heartbeat_appends_json() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Heartbeat {
            ledger_path: ledger_path.clone(),
            run_id: run_id.as_str().to_string(),
            worker_id: "worker-control".to_string(),
            observed_at_ms: 20_000,
            expires_at_ms: 35_000,
            metadata: Some(r#"{"queue":"llm.openai"}"#.to_string()),
            json: true,
        }),
        "control heartbeat json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "heartbeat output should be valid json",
    );
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let records = must_ok(
        ledger.load_events(&run_id),
        "heartbeat append should persist an event",
    );

    assert_eq!(json["sequence"], 2);
    assert_eq!(json["event"]["run_id"], "run-control-cli");
    assert_eq!(json["event"]["kind"]["event"], "worker_heartbeat_observed");
    assert_eq!(
        json["event"]["kind"]["heartbeat"]["worker_id"],
        "worker-control"
    );
    assert_eq!(
        json["event"]["kind"]["heartbeat"]["metadata"]["queue"],
        "llm.openai"
    );
    assert_eq!(records.len(), 2);
    Ok(())
}

#[test]
fn run_control_heartbeat_renders_text() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Heartbeat {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            worker_id: "worker-control".to_string(),
            observed_at_ms: 20_000,
            expires_at_ms: 35_000,
            metadata: None,
            json: false,
        }),
        "control heartbeat text should render",
    );

    assert!(output.rendered.starts_with("# Qianji Control Heartbeat"));
    assert!(output.rendered.contains("- Run: `run-control-cli`"));
    assert!(output.rendered.contains("- Worker: `worker-control`"));
    assert!(output.rendered.contains("- Observed at ms: `20000`"));
    assert!(output.rendered.contains("- Expires at ms: `35000`"));
    Ok(())
}

#[test]
fn run_control_heartbeat_rejects_invalid_metadata_without_append() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let Err(error) = run_control_command(&ControlCliCommand::Heartbeat {
        ledger_path: ledger_path.clone(),
        run_id: run_id.as_str().to_string(),
        worker_id: "worker-control".to_string(),
        observed_at_ms: 20_000,
        expires_at_ms: 35_000,
        metadata: Some("{not-json".to_string()),
        json: false,
    }) else {
        return Err("invalid heartbeat metadata should fail".to_string());
    };
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let records = must_ok(
        ledger.load_events(&run_id),
        "invalid heartbeat metadata should not append an event",
    );

    assert!(
        error
            .to_string()
            .contains("invalid `--metadata` JSON for `control heartbeat`")
    );
    assert_eq!(records.len(), 1);
    Ok(())
}

#[test]
fn run_control_query_state_renders_json_without_appending() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step_timer(&ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let before_count = must_ok(
        ledger.load_events(&run_id),
        "should load events before state query",
    )
    .len();

    let output = must_ok(
        run_control_command(&ControlCliCommand::QueryState {
            ledger_path: ledger_path.clone(),
            run_id: run_id.as_str().to_string(),
            now_ms: 9_000,
            json: true,
        }),
        "control query state json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "state query output should be valid json",
    );
    let after_count = must_ok(
        ledger.load_events(&run_id),
        "should load events after state query",
    )
    .len();

    assert_eq!(json["event_count"], before_count);
    assert_eq!(json["run_view"]["run_id"], "run-control-cli");
    assert_eq!(
        json["run_view"]["steps"]["run-control-step"]["timers"]["timer-step-approval-timeout"]["status"],
        "scheduled"
    );
    assert_eq!(json["recovery_snapshot"]["observed_at_ms"], 9_000);
    assert_eq!(before_count, after_count);
    Ok(())
}

#[test]
fn run_control_query_state_renders_text_summary() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step_signal_and_timer(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::QueryState {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            now_ms: 30_000,
            json: false,
        }),
        "control query state text should render",
    );

    assert!(output.rendered.starts_with("# Qianji Control State"));
    assert!(output.rendered.contains("- Run: `run-control-cli`"));
    assert!(output.rendered.contains("- Status: `draft`"));
    assert!(output.rendered.contains("- Steps: `1`"));
    assert!(output.rendered.contains("- Step timers: `1`"));
    assert!(output.rendered.contains("- Step signals: `1`"));
    assert!(output.rendered.contains("- Fireable timers: `1`"));
    Ok(())
}

#[test]
fn run_control_recovery_snapshot_renders_json() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::RecoverySnapshot {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            now_ms: 9_000,
            json: true,
        }),
        "control recovery snapshot json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "snapshot output should be valid json",
    );

    assert_eq!(json["run_id"], "run-control-cli");
    assert_eq!(json["observed_at_ms"], 9_000);
    assert_eq!(json["summary"]["total_actions"], 0);
    assert_eq!(json["plan"]["actions"].as_array().map(Vec::len), Some(0));
    Ok(())
}

#[test]
fn run_control_recovery_snapshot_renders_text_summary() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::RecoverySnapshot {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            now_ms: 9_000,
            json: false,
        }),
        "control recovery snapshot text should render",
    );

    assert!(
        output
            .rendered
            .starts_with("# Qianji Control Recovery Snapshot")
    );
    assert!(output.rendered.contains("- Run: `run-control-cli`"));
    assert!(output.rendered.contains("- Total actions: `0`"));
    Ok(())
}

fn append_empty_control_run(ledger_path: &std::path::Path) -> RunId {
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

fn append_control_run_with_step(ledger_path: &std::path::Path) -> RunId {
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

fn append_control_run_with_run_activity(ledger_path: &std::path::Path) -> RunId {
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

fn append_control_run_with_step_activity(ledger_path: &std::path::Path) -> RunId {
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

fn append_control_run_with_scheduled_activity_queue(ledger_path: &std::path::Path) -> RunId {
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

fn append_control_run_with_run_decision(ledger_path: &std::path::Path) -> RunId {
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

fn append_control_run_with_step_decision(ledger_path: &std::path::Path) -> RunId {
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

fn append_control_run_with_run_timer(ledger_path: &std::path::Path) -> RunId {
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

fn append_control_run_with_step_timer(ledger_path: &std::path::Path) -> RunId {
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

fn append_control_run_with_step_signal_and_timer(ledger_path: &std::path::Path) -> RunId {
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

fn activity_task(activity_id: ActivityId, activity_type: &str, task_queue: &str) -> ActivityTask {
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
