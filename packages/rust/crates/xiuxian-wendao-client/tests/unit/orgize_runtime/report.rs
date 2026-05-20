use std::process::Command;

use crate::orgize_runtime::support::tempdir_or_panic;

#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn standalone_orgize_task_report_summarizes_archive_and_repeating_rows() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Performance cadence :agent:performance:\n",
            "SCHEDULED: <2026-05-18 Mon ++1d>\n",
            "* TODO Completed but not closed [3/3] [100%] :agent:\n",
            "- [X] Scope\n",
            "- [X] Implementation\n",
            "- [X] Validation\n",
            "* DONE Completed slice :agent:achievement:\n",
            "CLOSED: [2026-05-17 Sun]\n",
            ":PROPERTIES:\n",
            ":NEXT_ACTION: Archive after review\n",
            ":END:\n",
            "* DONE Completed cadence :agent:performance:\n",
            "SCHEDULED: <2026-05-17 Sun ++1d>\n",
            "CLOSED: [2026-05-17 Sun]\n",
            "* DONE Archived slice :agent:achievement:\n",
            "CLOSED: [2026-05-16 Sat]\n",
            ":PROPERTIES:\n",
            ":ARCHIVE_TIME: 2026-05-16 Sat\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .env_remove("PRJ_CACHE_HOME")
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("task-report")
        .arg("agenda.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize task-report: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let stderr =
        String::from_utf8(output.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("orgize agent task-report"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("rows: 5"), "stdout: {stdout}");
    assert!(stdout.contains("active: 2"), "stdout: {stdout}");
    assert!(stdout.contains("done: 3"), "stdout: {stdout}");
    assert!(stdout.contains("achievements: 2"), "stdout: {stdout}");
    assert!(stdout.contains("archive-candidates: 2"), "stdout: {stdout}");
    assert!(stdout.contains("repeating: 2"), "stdout: {stdout}");
    assert!(stdout.contains("closure-needed: 1"), "stdout: {stdout}");
    assert!(stdout.contains("agent: 5"), "stdout: {stdout}");
    assert!(stdout.contains("achievement: 2"), "stdout: {stdout}");
    assert!(stdout.contains("performance: 2"), "stdout: {stdout}");
    assert!(stdout.contains("Closure Needed: 1"), "stdout: {stdout}");
    assert!(stdout.contains("Archive Candidates: 2"), "stdout: {stdout}");
    assert!(stdout.contains("Achievements: 2"), "stdout: {stdout}");
    assert!(stdout.contains("Repeating Tasks: 2"), "stdout: {stdout}");
    assert!(
        stdout.contains("repeat: scheduled ++1d (catchUp)"),
        "stdout: {stdout}"
    );
}
