use std::process::Command;

use crate::orgize_runtime::support::tempdir_or_panic;

use super::assert_agent_task_row_count;

#[test]
fn standalone_orgize_read_model_materializes_default_duckdb_snapshot() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Agent task :agent:\n",
            "SCHEDULED: <2026-05-17 Sun>\n",
            ":PROPERTIES:\n",
            ":NEXT_ACTION: Verify DuckDB snapshot\n",
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
        .arg("read-model")
        .arg("agenda.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize read-model: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    assert_eq!(output.status.code(), Some(0), "stdout: {stdout}");
    assert!(stdout.contains("backend: duckdb"), "stdout: {stdout}");
    assert!(stdout.contains("rows: 2"), "stdout: {stdout}");
    let database_path = temp
        .path()
        .join(".cache")
        .join("agent")
        .join("readmodels")
        .join("org_agent_tasks.duckdb");
    assert!(database_path.is_file());
    assert_agent_task_row_count(&database_path, 2);
}
