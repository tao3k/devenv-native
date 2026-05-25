use std::process::Command;

use crate::orgize_runtime::support::tempdir_or_panic;

#[test]
fn standalone_orgize_task_list_json_outputs_limited_recovery_rows() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO First active task :agent:\n",
            ":PROPERTIES:\n",
            ":ID: first-active\n",
            ":NEXT_ACTION: Continue first\n",
            ":RESUME_QUERY: wendao-client orgize task-list --text 'First active'\n",
            ":END:\n",
            "** Reflection Questions\n",
            "| Question | Value |\n",
            "|---+---|\n",
            "| Which preference or naming correction should future generated plans preserve? | Prefer orgid-show for exact recovery. |\n",
            "* TODO Second active task :agent:performance:\n",
            "SCHEDULED: <2026-05-18 Mon ++1d>\n",
            ":PROPERTIES:\n",
            ":ID: second-active\n",
            ":END:\n",
            "* TODO Third active task :agent:\n",
            ":PROPERTIES:\n",
            ":ID: third-active\n",
            ":END:\n",
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
        .arg("task-list")
        .arg("--limit")
        .arg("2")
        .arg("agenda.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize task-list json: {error}"));
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
    assert_eq!(parsed["command"], "orgize task-list");
    assert_eq!(parsed["backend"], "duckdb");
    assert_eq!(parsed["rows"], 3);
    assert_eq!(parsed["showing"], 2);
    assert_eq!(parsed["active"], 3);
    assert_eq!(parsed["tasks"].as_array().map_or(0, Vec::len), 2);
    assert_eq!(parsed["tasks"][0]["orgid"], "first-active");
    assert_eq!(parsed["tasks"][0]["title"], "First active task");
    assert_eq!(parsed["tasks"][0]["next"], "Continue first");
    assert_eq!(
        parsed["tasks"][0]["resume"],
        "wendao-client orgize task-list --text 'First active'"
    );
    assert_eq!(parsed["tasks"][0]["memoryObjects"][0]["kind"], "preference");
    assert_eq!(
        parsed["tasks"][0]["memoryObjects"][0]["facet"],
        "memory-preference"
    );
    assert_eq!(
        parsed["tasks"][0]["memoryObjects"][0]["value"],
        "Prefer orgid-show for exact recovery."
    );
    assert_eq!(parsed["tasks"][1]["title"], "Second active task");
    assert_eq!(parsed["tasks"][1]["repeat"][0], "scheduled ++1d (catchUp)");
}
