use super::*;

#[test]
fn run_show_dir_command_renders_flowhub_summary() {
    let output = must_ok(
        run_dir_command(DirCliCommand::Show {
            target: ShowCliTarget::Dir(flowhub_root()),
        }),
        "show command should render Flowhub summary",
    );

    assert_eq!(output.exit_code, 0);
    assert_common_show_shape(&output.rendered);
    assert!(output.rendered.contains("# Flowhub"));
    assert!(output.rendered.contains("## rust"));
    assert!(output.rendered.contains("## blueprint"));
    assert!(output.rendered.contains("## research"));
}

#[test]
fn run_show_dir_command_renders_scenario_preview() {
    let output = must_ok(
        run_dir_command(DirCliCommand::Show {
            target: ShowCliTarget::Dir(scenario_fixture_dir("coding_rust_blueprint_plan")),
        }),
        "show command should render scenario preview",
    );

    assert_eq!(output.exit_code, 0);
    assert_common_show_shape(&output.rendered);
    assert!(output.rendered.contains("# Scenario Work Surface Preview"));
    assert!(
        output
            .rendered
            .contains("Scenario: coding-rust-blueprint-plan-demo")
    );
    assert!(output.rendered.contains("## blueprint"));
    assert!(output.rendered.contains("## plan"));
    assert!(output.rendered.contains("blueprint --> plan"));
}
