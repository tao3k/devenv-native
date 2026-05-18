use super::{
    ControlCliCommand, TempDir, must_ok, must_some, parse_control_command, run_control_command,
    to_args,
};
use xiuxian_qianji_control::{
    ControlEvent, ControlEventKind, ControlLedger, DuckDbControlLedger, RunId, StepId,
};

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
    assert_eq!(json["steps"].as_object().map(|steps| steps.len()), Some(1));
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

    let error = run_control_command(&ControlCliCommand::Step {
        ledger_path,
        run_id: run_id.as_str().to_string(),
        step_id: "missing-step".to_string(),
        json: false,
    })
    .expect_err("missing step should fail");

    assert!(
        error
            .to_string()
            .contains("could not find step `missing-step`")
    );
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
