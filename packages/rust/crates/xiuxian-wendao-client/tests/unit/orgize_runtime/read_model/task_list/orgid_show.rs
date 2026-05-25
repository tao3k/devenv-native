use std::process::Command;

use crate::orgize_runtime::support::{assert_cli_success, run_orgize, tempdir_or_panic};

#[test]
fn standalone_orgize_orgid_show_restores_exact_section_by_orgid() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    write_orgid_show_agenda(&agenda);

    let output = run_orgize(
        temp.path(),
        &["orgid-show", "--id", "target-task", "agenda.org"],
        "orgid-show target",
    );

    assert_cli_success(&output);
    assert_orgid_show_compact_output(&output.stdout);
}

#[test]
fn standalone_orgize_orgid_show_full_renders_exact_section_source() {
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
        &["orgid-show", "--id", "target-task", "--full", "agenda.org"],
        "orgid-show target full",
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

#[test]
fn standalone_orgize_orgid_show_json_projects_inferred_memory_objects() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Target task :agent:\n",
            ":PROPERTIES:\n",
            ":ID: target-task\n",
            ":END:\n",
            "** Reflection Questions\n",
            "| Question | Value |\n",
            "|---+---|\n",
            "| Which failure mode should future agents avoid? | Do not add a redundant memory subcommand. |\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .env_remove("PRJ_CACHE_HOME")
        .arg("--root")
        .arg(temp.path())
        .arg("--output")
        .arg("json")
        .arg("orgize")
        .arg("orgid-show")
        .arg("--id")
        .arg("target-task")
        .arg("agenda.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize orgid-show json: {error}"));
    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let stderr =
        String::from_utf8(output.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).unwrap_or_else(|error| panic!("parse json: {error}"));

    assert_eq!(parsed["command"], "orgize orgid-show");
    assert_eq!(parsed["task"]["orgid"], "target-task");
    assert_eq!(parsed["task"]["memoryObjects"][0]["kind"], "failure");
    assert_eq!(
        parsed["task"]["memoryObjects"][0]["facet"],
        "memory-failure"
    );
    assert_eq!(
        parsed["task"]["memoryObjects"][0]["question"],
        "Which failure mode should future agents avoid?"
    );
    assert_eq!(
        parsed["task"]["memoryObjects"][0]["value"],
        "Do not add a redundant memory subcommand."
    );
}

fn write_orgid_show_agenda(agenda: &std::path::Path) {
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

fn assert_orgid_show_compact_output(stdout: &str) {
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
        stdout.contains("full: wendao-client orgize orgid-show --cached --id target-task --full"),
        "stdout: {stdout}"
    );
    assert!(
        !stdout.contains("Large evidence body should not render by default."),
        "stdout: {stdout}"
    );
    assert!(!stdout.contains("Third task"), "stdout: {stdout}");
}
