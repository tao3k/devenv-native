use super::super::*;

fn materialize_claim_extract_run(temp_dir: &TempDir) -> PathBuf {
    let run_dir = temp_dir.path().join("runs/run_005");
    must_ok(
        run_dir_command(DirCliCommand::Materialize {
            target: MaterializeCliTarget::AnchoredScenario {
                anchor: anchored_workdir_fixture_anchor(),
                scenario: anchored_workdir_fixture_scenario().to_string(),
                dir: run_dir.clone(),
                current_node: Some("claim_extract".to_string()),
            },
        }),
        "materialize command should scaffold the claim_extract workdir",
    );
    run_dir
}

#[test]
fn run_advance_command_updates_runtime_state_and_trace() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let run_dir = materialize_claim_extract_run(&temp_dir);

    let output = must_ok(
        run_dir_command(DirCliCommand::Advance {
            dir: run_dir.clone(),
            to: "evidence_ground".to_string(),
        }),
        "advance command should move the localized workdir to the next node",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.starts_with("# Advanced Workdir Step"));
    assert!(output.rendered.contains("Previous node: claim_extract"));
    assert!(output.rendered.contains("Current node: evidence_ground"));
    assert!(
        output
            .rendered
            .contains("Nodes: `diagnostics`, `limitation_extract`")
    );
    assert!(output.rendered.contains("## Current Step Surface"));
    assert!(
        output
            .rendered
            .contains("- checkpoints/evidence_ground.json")
    );
    assert!(
        output
            .rendered
            .contains("- staging/semantics/evidence_ledger.patch.jsonl")
    );

    assert_eq!(
        must_ok(
            fs::read_to_string(run_dir.join("state/current_node.toml")),
            "current node state should be readable after advance",
        ),
        "current_node = \"evidence_ground\"\n"
    );
    assert_eq!(
        must_ok(
            fs::read_to_string(run_dir.join("state/allowed_next.json")),
            "allowed-next state should be readable after advance",
        ),
        "[\n  \"diagnostics\",\n  \"limitation_extract\"\n]\n"
    );

    let trace = must_ok(
        fs::read_to_string(run_dir.join("state/trace.jsonl")),
        "trace state should be readable after advance",
    );
    assert!(trace.contains("\"event\":\"step_advance\""));
    assert!(trace.contains("\"from\":\"claim_extract\""));
    assert!(trace.contains("\"to\":\"evidence_ground\""));
    assert!(run_dir.join("checkpoints/evidence_ground.json").is_file());
    assert!(
        run_dir
            .join("staging/semantics/evidence_ledger.patch.jsonl")
            .is_file()
    );

    let report = xiuxian_qianji::check_workdir(&run_dir)
        .unwrap_or_else(|error| panic!("advanced workdir should still check: {error}"));
    assert!(report.is_valid());
}

#[test]
fn run_advance_command_rejects_non_adjacent_target_without_mutation() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let run_dir = materialize_claim_extract_run(&temp_dir);
    let original_current_node = must_ok(
        fs::read_to_string(run_dir.join("state/current_node.toml")),
        "current node state should be readable before failure",
    );
    let original_allowed_next = must_ok(
        fs::read_to_string(run_dir.join("state/allowed_next.json")),
        "allowed-next state should be readable before failure",
    );
    let original_trace = must_ok(
        fs::read_to_string(run_dir.join("state/trace.jsonl")),
        "trace state should be readable before failure",
    );

    let error = run_dir_command(DirCliCommand::Advance {
        dir: run_dir.clone(),
        to: "methods_extract".to_string(),
    })
    .err()
    .unwrap_or_else(|| panic!("non-adjacent advance should fail"));

    assert!(
        error
            .to_string()
            .contains("cannot advance to `methods_extract`")
    );
    assert!(error.to_string().contains("state/allowed_next.json"));

    assert_eq!(
        must_ok(
            fs::read_to_string(run_dir.join("state/current_node.toml")),
            "current node state should remain unchanged after failure",
        ),
        original_current_node
    );
    assert_eq!(
        must_ok(
            fs::read_to_string(run_dir.join("state/allowed_next.json")),
            "allowed-next state should remain unchanged after failure",
        ),
        original_allowed_next
    );
    assert_eq!(
        must_ok(
            fs::read_to_string(run_dir.join("state/trace.jsonl")),
            "trace state should remain unchanged after failure",
        ),
        original_trace
    );

    let report = xiuxian_qianji::check_workdir(&run_dir).unwrap_or_else(|error| {
        panic!("failed advance should leave the localized workdir checkable: {error}")
    });
    assert!(report.is_valid());
}
