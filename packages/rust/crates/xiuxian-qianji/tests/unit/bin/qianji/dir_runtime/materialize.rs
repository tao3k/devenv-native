use super::super::*;

#[test]
fn run_materialize_anchor_command_generates_checkable_run_root() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let run_dir = temp_dir.path().join("runs/run_001");

    let output = must_ok(
        run_dir_command(DirCliCommand::Materialize {
            target: MaterializeCliTarget::AnchoredScenario {
                anchor: anchored_workdir_fixture_anchor(),
                scenario: anchored_workdir_fixture_scenario().to_string(),
                dir: run_dir.clone(),
                current_node: None,
            },
        }),
        "materialize command should create a localized run root",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.starts_with("# Materialized Work Surface"));
    assert!(output.rendered.contains("Scenario: deep_read"));
    assert!(output.rendered.contains("Current node: research/paper"));
    assert!(
        output
            .rendered
            .contains("Allowed next: `load_paper_package`")
    );
    assert!(output.rendered.contains("## Current Step Surface"));
    assert!(run_dir.join("qianji.toml").is_file());
    assert!(run_dir.join("flowchart.mmd").is_file());
    assert!(run_dir.join("refs/paper.json").is_file());
    assert!(run_dir.join("refs/topic.json").is_file());
    assert!(run_dir.join("state/current_node.toml").is_file());
    assert!(run_dir.join("state/allowed_next.json").is_file());

    let report = xiuxian_qianji::check_workdir(&run_dir)
        .unwrap_or_else(|error| panic!("materialized run root should check: {error}"));
    assert!(report.is_valid());
}

#[test]
fn run_materialize_anchor_command_rejects_non_empty_output_dir() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let run_dir = temp_dir.path().join("runs/run_001");
    must_ok(
        fs::create_dir_all(&run_dir),
        "should create non-empty materialize target",
    );
    write_file(&run_dir.join("stale.txt"), "stale\n");

    let error = run_dir_command(DirCliCommand::Materialize {
        target: MaterializeCliTarget::AnchoredScenario {
            anchor: anchored_workdir_fixture_anchor(),
            scenario: anchored_workdir_fixture_scenario().to_string(),
            dir: run_dir,
            current_node: None,
        },
    })
    .err()
    .unwrap_or_else(|| panic!("non-empty materialize target should fail"));

    assert!(error.to_string().contains("must be empty"));
}

#[test]
fn run_materialize_anchor_command_scaffolds_selected_current_node() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let run_dir = temp_dir.path().join("runs/run_003");

    let output = must_ok(
        run_dir_command(DirCliCommand::Materialize {
            target: MaterializeCliTarget::AnchoredScenario {
                anchor: anchored_workdir_fixture_anchor(),
                scenario: anchored_workdir_fixture_scenario().to_string(),
                dir: run_dir.clone(),
                current_node: Some("claim_extract".to_string()),
            },
        }),
        "materialize command should scaffold one selected current node",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.contains("Current node: claim_extract"));
    assert!(
        output
            .rendered
            .contains("Allowed next: `diagnostics`, `evidence_ground`")
    );
    assert!(output.rendered.contains("- checkpoints/claim_extract.json"));
    assert!(
        output
            .rendered
            .contains("- staging/semantics/claim_ledger.patch.jsonl")
    );
    assert!(run_dir.join("checkpoints/claim_extract.json").is_file());
    assert!(
        run_dir
            .join("staging/semantics/claim_ledger.patch.jsonl")
            .is_file()
    );
    assert!(!run_dir.join("checkpoints/evidence_ground.json").exists());

    let report = xiuxian_qianji::check_workdir(&run_dir)
        .unwrap_or_else(|error| panic!("current-node scaffolded run root should check: {error}"));
    assert!(report.is_valid());
}
