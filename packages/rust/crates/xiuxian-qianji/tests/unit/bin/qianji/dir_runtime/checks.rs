use super::super::*;

#[test]
fn run_check_dir_command_accepts_flowhub_root() {
    let output = must_ok(
        run_dir_command(DirCliCommand::Check {
            dir: flowhub_root(),
        }),
        "check command should validate Flowhub root",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.contains("# Validation Passed"));
    assert!(output.rendered.contains("Checked modules:"));
}

#[test]
fn run_check_dir_command_accepts_scenario_dir() {
    let output = must_ok(
        run_dir_command(DirCliCommand::Check {
            dir: scenario_fixture_dir("coding_rust_blueprint_plan"),
        }),
        "check command should validate scenario dir",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.contains("# Validation Passed"));
    assert!(
        output
            .rendered
            .contains("Scenario: coding-rust-blueprint-plan-demo")
    );
    assert!(
        output
            .rendered
            .contains("Visible surfaces: flowchart.mmd, coding, rust, blueprint, plan")
    );
}

#[test]
fn run_check_dir_command_blocks_invalid_scenario_dir() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let scenario_dir = create_invalid_scenario_fixture(&temp_dir);

    let output = must_ok(
        run_dir_command(DirCliCommand::Check { dir: scenario_dir }),
        "check command should render scenario diagnostics",
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.rendered.contains("# Validation Failed"));
    assert!(output.rendered.contains("Scenario resolve failed"));
    assert!(output.rendered.contains("missing-module"));
}

#[test]
fn run_check_dir_command_reports_missing_workdir_root() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let missing_workdir = temp_dir.path().join("runs/run_001");

    let output = must_ok(
        run_dir_command(DirCliCommand::Check {
            dir: missing_workdir.clone(),
        }),
        "check command should render bootstrap diagnostics for missing workdir roots",
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.rendered.contains("# Validation Failed"));
    assert!(output.rendered.contains("Missing workdir root"));
    assert!(output.rendered.contains("localized run root"));
    assert!(
        output
            .rendered
            .contains(missing_workdir.to_string_lossy().as_ref())
    );
    assert!(
        output
            .rendered
            .contains("qianji show --anchor <module-qianji.toml> --scenario <name> --dir")
    );
    assert!(!output.rendered.contains("## Follow-up Query"));
}

#[test]
fn run_check_dir_command_reports_uninitialized_workdir_root() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = temp_dir.path().join("runs/run_001");
    must_ok(
        fs::create_dir_all(&workdir),
        "should create an empty run root for bootstrap diagnostics",
    );

    let output = must_ok(
        run_dir_command(DirCliCommand::Check {
            dir: workdir.clone(),
        }),
        "check command should render bootstrap diagnostics for empty workdir roots",
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.rendered.contains("# Validation Failed"));
    assert!(output.rendered.contains("Uninitialized workdir root"));
    assert!(output.rendered.contains("bounded work-surface manifest"));
    assert!(output.rendered.contains(workdir.to_string_lossy().as_ref()));
    assert!(output.rendered.contains("qianji.toml"));
    assert!(!output.rendered.contains("## Follow-up Query"));
}
