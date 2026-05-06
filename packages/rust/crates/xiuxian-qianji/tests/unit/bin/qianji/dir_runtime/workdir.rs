use super::{
    DirCliCommand, ShowCliTarget, TempDir, assert_common_show_shape, create_workdir_fixture, fs,
    must_ok, run_dir_command,
};

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
        "select path, surface, surface_kind, heading_path, skeleton \
from markdown \
where surface = 'plan' \
order by surface, path, heading_path"
    ));
}
