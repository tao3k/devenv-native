use std::process::Command;

use crate::orgize_runtime::support::tempdir_or_panic;

#[test]
fn standalone_orgize_task_list_lists_active_rows_from_duckdb_snapshot() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Agent task :agent:performance:\n",
            "SCHEDULED: <2026-05-17 Sun ++1w> DEADLINE: <2026-05-20 Wed .+2d>\n",
            ":PROPERTIES:\n",
            ":NEXT_ACTION: Verify DuckDB snapshot\n",
            ":RESUME_QUERY: wendao-client orgize task-list --text 'Agent task'\n",
            ":END:\n",
            "** Evidence\n",
            "This inherited agent-tag section should not become a task row.\n",
            "** TODO Task Checklist\n",
            "This inherited agent-tag TODO section should not become a task row.\n",
            "* DONE Closed task :agent:\n",
            "CLOSED: [2026-05-16 Sat]\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .env_remove("PRJ_CACHE_HOME")
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("task-list")
        .arg("--text")
        .arg("++1w")
        .arg("--tag")
        .arg("performance")
        .arg("agenda.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize task-list: {error}"));

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
        stdout.contains("orgize agent task-list"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("rows: 1"), "stdout: {stdout}");
    assert!(stdout.contains("[TASK001] Agent task"), "stdout: {stdout}");
    assert!(
        stdout.contains("tags: agent:performance"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("scheduled: <2026-05-17 Sun ++1w>"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("deadline: <2026-05-20 Wed .+2d>"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("repeat: scheduled ++1w (catchUp), deadline .+2d (restart)"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("next: Verify DuckDB snapshot"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("resume: wendao-client orgize task-list --text 'Agent task'"),
        "stdout: {stdout}"
    );
    assert!(!stdout.contains("Closed task"), "stdout: {stdout}");
    assert!(!stdout.contains("Evidence"), "stdout: {stdout}");
}
