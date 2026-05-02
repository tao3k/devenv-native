use super::{
    DirCliCommand, ShowCliTarget, TempDir, anchored_workdir_fixture_anchor,
    anchored_workdir_fixture_scenario, assert_common_show_shape,
    create_anchored_runtime_state_fixture, must_ok, run_dir_command,
};

#[test]
fn run_show_anchor_command_renders_research_execution_brief() {
    let output = must_ok(
        run_dir_command(DirCliCommand::Show {
            target: ShowCliTarget::AnchoredScenario {
                anchor: anchored_workdir_fixture_anchor(),
                scenario: anchored_workdir_fixture_scenario().to_string(),
                dir: None,
            },
        }),
        "show anchor command should resolve one execution brief",
    );

    assert_eq!(output.exit_code, 0);
    assert_common_show_shape(&output.rendered);
    assert!(output.rendered.starts_with("# Execution Brief"));
    assert!(output.rendered.contains("Scenario: deep_read"));
    assert!(
        output
            .rendered
            .contains("tests/fixtures/flowhub_modules/paper_deep_read_workdir/qianji.toml")
    );
    assert!(
        output
            .rendered
            .contains("tests/fixtures/flowhub_modules/paper_deep_read_workdir/paper-deep-read.mmd")
    );
    assert!(output.rendered.contains("## Goal"));
    assert!(
        output
            .rendered
            .contains("canonical paper package -> deep read package")
    );
    assert!(output.rendered.contains("## Execution"));
    assert!(output.rendered.contains("## Check Surface"));
}

#[test]
fn run_show_anchor_command_renders_runtime_overlay() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let runtime_dir = create_anchored_runtime_state_fixture(&temp_dir);

    let output = must_ok(
        run_dir_command(DirCliCommand::Show {
            target: ShowCliTarget::AnchoredScenario {
                anchor: anchored_workdir_fixture_anchor(),
                scenario: anchored_workdir_fixture_scenario().to_string(),
                dir: Some(runtime_dir),
            },
        }),
        "show anchor command should render runtime overlay",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.contains("## Current State"));
    assert!(output.rendered.contains("Current node: claim_extract"));
    assert!(
        output
            .rendered
            .contains("Allowed next: `diagnostics`, `evidence_ground`")
    );
    assert!(
        output
            .rendered
            .contains("## Writable Surface For This Step")
    );
    assert!(output.rendered.contains("checkpoints/claim_extract.json"));
    assert!(
        output
            .rendered
            .contains("staging/semantics/claim_ledger.patch.jsonl")
    );
    assert!(output.rendered.contains("## Merge Target For This Step"));
    assert!(output.rendered.contains("semantics/claim_ledger.jsonl"));
    assert!(output.rendered.contains("## Success Condition"));
}

#[test]
fn run_show_anchor_command_blocks_missing_scenario() {
    let error = run_dir_command(DirCliCommand::Show {
        target: ShowCliTarget::AnchoredScenario {
            anchor: anchored_workdir_fixture_anchor(),
            scenario: "missing".to_string(),
            dir: None,
        },
    })
    .err()
    .unwrap_or_else(|| panic!("missing anchored scenario should fail"));

    let message = error.to_string();
    assert!(message.contains("does not declare scenario `missing`"));
    assert!(message.contains("paper-deep-read.mmd"));
}
