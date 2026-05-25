use crate::orgize_runtime::support::{assert_cli_success, run_orgize, tempdir_or_panic};

#[test]
fn standalone_orgize_ogrid_show_restores_exact_section_by_orgid() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    write_ogrid_show_agenda(&agenda);

    let output = run_orgize(
        temp.path(),
        &["ogrid-show", "--id", "target-task", "agenda.org"],
        "ogrid-show target",
    );

    assert_cli_success(&output);
    assert_ogrid_show_compact_output(&output.stdout);
}

#[test]
fn standalone_orgize_ogrid_show_full_renders_exact_section_source() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Target task :agent:\n",
            ":PROPERTIES:\n",
            ":ID: target-task\n",
            ":END:\n",
            "** Evidence\n",
            "Full body.\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = run_orgize(
        temp.path(),
        &["ogrid-show", "--id", "target-task", "--full", "agenda.org"],
        "ogrid-show target full",
    );

    assert_cli_success(&output);
    assert!(
        output.stdout.contains("section:"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output.stdout.contains("Full body."),
        "stdout: {}",
        output.stdout
    );
}

fn write_ogrid_show_agenda(agenda: &std::path::Path) {
    std::fs::write(
        agenda,
        concat!(
            "* TODO First task :agent:\n",
            ":PROPERTIES:\n",
            ":ID: first-task\n",
            ":END:\n",
            "First body.\n",
            "* TODO Target task :agent:\n",
            ":PROPERTIES:\n",
            ":ID: target-task\n",
            ":NEXT_ACTION: Continue exact section\n",
            ":END:\n",
            "** Checklist\n",
            "- [X] Restore lookup\n",
            "- [ ] Finish restoration\n",
            "** Evidence\n",
            "Large evidence body should not render by default.\n",
            "* TODO Third task :agent:\n",
            ":PROPERTIES:\n",
            ":ID: third-task\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));
}

fn assert_ogrid_show_compact_output(stdout: &str) {
    assert!(!stdout.contains("backend:"), "stdout: {stdout}");
    assert!(!stdout.contains("database:"), "stdout: {stdout}");
    assert!(!stdout.contains("snapshot:"), "stdout: {stdout}");
    assert!(!stdout.contains("table:"), "stdout: {stdout}");
    assert!(!stdout.contains("orgid: target-task"), "stdout: {stdout}");
    assert!(!stdout.contains("range:"), "stdout: {stdout}");
    assert!(!stdout.contains("outline: Target task"), "stdout: {stdout}");
    assert!(
        stdout.contains("next: Continue exact section"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("checklist-progress: [1/2] [50%]"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("next-unchecked: - [ ] Finish restoration"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("checklist:"), "stdout: {stdout}");
    assert!(
        stdout.contains("children:\n** Checklist\n** Evidence"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("full: wendao-client orgize ogrid-show --cached --id target-task --full"),
        "stdout: {stdout}"
    );
    assert!(
        !stdout.contains("Large evidence body should not render by default."),
        "stdout: {stdout}"
    );
    assert!(!stdout.contains("Third task"), "stdout: {stdout}");
}
