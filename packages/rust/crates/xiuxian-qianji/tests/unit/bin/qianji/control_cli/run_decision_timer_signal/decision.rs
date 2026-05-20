use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use crate::qianji_cli::tests::control_cli::support::{
    append_control_run_with_run_decision, append_control_run_with_step_decision, must_ok,
};
use tempfile::TempDir;

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
