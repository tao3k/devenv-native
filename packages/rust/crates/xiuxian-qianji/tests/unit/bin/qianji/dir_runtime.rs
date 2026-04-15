use super::*;

#[test]
fn run_show_workdir_command_renders_surface_summary() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_workdir_fixture(&temp_dir);

    let output = must_ok(
        run_dir_command(DirCliCommand::Show {
            target: ShowCliTarget::Dir(workdir.clone()),
        }),
        "show command should render",
    );

    assert_eq!(output.exit_code, 0);
    assert_common_show_shape(&output.rendered);
    assert!(output.rendered.contains("# Work Surface"));
    assert!(output.rendered.contains("## blueprint"));
    assert!(output.rendered.contains("- architecture.md"));
    assert!(output.rendered.contains("## plan"));
    assert!(output.rendered.contains("- tasks.md"));
}

#[test]
fn run_check_workdir_command_blocks_invalid_surface() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_workdir_fixture(&temp_dir);
    must_ok(
        fs::remove_file(workdir.join("plan/tasks.md")),
        "should remove plan markdown for failing check",
    );

    let output = must_ok(
        run_dir_command(DirCliCommand::Check { dir: workdir }),
        "check command should render diagnostics",
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.rendered.contains("# Validation Failed"));
    assert!(output.rendered.contains("Missing required glob matches"));
    assert!(output.rendered.contains("## Follow-up Query"));
    assert!(output.rendered.contains("Surfaces: plan"));
}

#[test]
fn run_check_workdir_command_renders_follow_up_query() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_workdir_fixture(&temp_dir);
    must_ok(
        fs::remove_file(workdir.join("plan/tasks.md")),
        "should remove plan markdown for follow-up rendering",
    );

    let output = must_ok(
        run_dir_command(DirCliCommand::Check { dir: workdir }),
        "check command should render follow-up query",
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.rendered.contains("## Follow-up Query"));
    assert!(output.rendered.contains(
        "select path, surface, heading_path, skeleton \
from markdown \
where surface = 'plan' \
order by surface, path, heading_path"
    ));
}

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

#[test]
fn run_show_graph_command_renders_flowhub_mermaid_graph() {
    let output = must_ok(
        run_dir_command(DirCliCommand::Show {
            target: ShowCliTarget::Graph(flowhub_root().join("plan/codex-plan.mmd")),
        }),
        "show graph command should render Flowhub Mermaid preview",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.starts_with("# Graph"));
    assert!(output.rendered.contains("Name: codex-plan"));
    assert!(
        output
            .rendered
            .contains("Path: ./qianji-flowhub/plan/codex-plan.mmd")
    );
    assert!(output.rendered.contains("Kind: scenario"));
    assert!(output.rendered.contains("## Mermaid"));
    assert!(output.rendered.contains("```mermaid"));
    assert!(output.rendered.contains("flowchart LR"));
    assert!(output.rendered.contains("## Nodes"));
    assert!(output.rendered.contains("### coding"));
    assert!(output.rendered.contains("Kind: context"));
    assert!(output.rendered.contains("### blueprint"));
    assert!(output.rendered.contains("Kind: artifact"));
    assert!(output.rendered.contains("### domain validators"));
    assert!(output.rendered.contains("Kind: validator"));
    assert!(output.rendered.contains("## Module contract"));
    assert!(output.rendered.contains("- qianji.toml"));
    assert!(output.rendered.contains("- codex-plan.mmd"));
    assert!(output.rendered.contains("## Owning qianji.toml"));
    assert!(output.rendered.contains("[module]"));
    assert!(output.rendered.contains("name = \"plan\""));
}

#[test]
fn run_show_graph_command_prefers_declared_graph_name_override() {
    let output = must_ok(
        run_dir_command(DirCliCommand::Show {
            target: ShowCliTarget::Graph(flowhub_root().join("wendao/docs-search.mmd")),
        }),
        "show graph command should respect the local Flowhub graph name override",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.starts_with("# Graph"));
    assert!(output.rendered.contains("Name: DOC_SEARCH"));
    assert!(
        output
            .rendered
            .contains("Path: ./qianji-flowhub/wendao/docs-search.mmd")
    );
    assert!(output.rendered.contains("Declared topology: bounded_loop"));
    assert!(output.rendered.contains("## Owning qianji.toml"));
    assert!(output.rendered.contains("name = \"DOC_SEARCH\""));
}

#[test]
fn run_show_contract_command_renders_wendao_docs_contract_snapshot() {
    let output = must_ok(
        run_dir_command(DirCliCommand::Show {
            target: ShowCliTarget::Contract("wendao.docs.navigation".to_string()),
        }),
        "show contract command should render Wendao docs contract snapshot",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.starts_with("# Contract"));
    assert!(output.rendered.contains("Name: wendao.docs.navigation"));
    assert!(
        output
            .rendered
            .contains("Kind: wendao-docs-invocation-contract")
    );
    assert!(output.rendered.contains("## Contract TOML"));
    assert!(output.rendered.contains("task_types = ["));
    assert!(output.rendered.contains("path = \"/api/docs/navigation\""));
    assert!(output.rendered.contains("## Schema JSON"));
    assert!(
        output
            .rendered
            .contains("\"title\": \"DocsNavigationToolArgs\"")
    );
    assert!(output.rendered.contains("\"page_id\""));
}

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
