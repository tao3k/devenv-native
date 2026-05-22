use std::process::Command;

use crate::orgize_runtime::support::tempdir_or_panic;

use super::assert_agent_task_row_count;

#[test]
fn standalone_orgize_read_model_uses_wendao_toml_path_overrides() {
    let temp = tempdir_or_panic();
    std::fs::write(
        temp.path().join("wendao.toml"),
        concat!(
            "[agent.org_read_model]\n",
            "database_path = \".cache/custom-agent/tasks.duckdb\"\n",
            "temp_directory = \".cache/custom-agent/tmp\"\n",
            "threads = 1\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write wendao.toml: {error}"));
    std::fs::write(
        temp.path().join("agenda.org"),
        "* TODO Agent task :agent:\n",
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
    assert!(stdout.contains("rows: 1"), "stdout: {stdout}");
    let database_path = temp
        .path()
        .join(".cache")
        .join("custom-agent")
        .join("tasks.duckdb");
    assert!(database_path.is_file());
    assert!(
        stdout.contains(database_path.to_string_lossy().as_ref()),
        "stdout: {stdout}"
    );
    assert_agent_task_row_count(&database_path, 1);
}
