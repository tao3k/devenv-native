use std::process::Command;

use crate::orgize_runtime::support::tempdir_or_panic;

#[test]
fn standalone_orgize_agent_planning_renders_org_cards() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Agent task :agent:\n",
            "SCHEDULED: <2026-05-17 Sun>\n",
            "* DONE Closed task :agent:\n",
            "SCHEDULED: <2026-05-17 Sun>\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("agent-planning")
        .arg("--date")
        .arg("2026-05-17")
        .arg("agenda.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize agent-planning: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    assert_eq!(output.status.code(), Some(0));
    assert!(stdout.contains("task: Agent task"), "stdout: {stdout}");
    assert!(!stdout.contains("Closed task"), "stdout: {stdout}");
}
