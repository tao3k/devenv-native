use std::process::Command;

use crate::orgize_runtime::support::tempdir_or_panic;

use crate::orgize_runtime::read_model::{
    agent_read_model_database_path, assert_agent_memory_object_row_count,
    assert_agent_org_element_join_matches_text, assert_agent_org_element_projection_exists,
    assert_agent_org_element_row_count_at_least,
};

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
            ":ID: agent-task\n",
            ":NEXT_ACTION: Verify DuckDB snapshot\n",
            ":RESUME_QUERY: wendao-client orgize task-list --text 'Agent task'\n",
            ":END:\n",
            "** Evidence\n",
            "This inherited agent-tag section should not become a task row.\n",
            "** TODO Task Checklist\n",
            "This inherited agent-tag TODO section should not become a task row.\n",
            "* DONE Closed task :agent:\n",
            "CLOSED: [2026-05-16 Sat]\n",
            ":PROPERTIES:\n",
            ":ID: closed-task\n",
            ":END:\n",
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
    assert!(stdout.contains("[TASK001] Agent task"), "stdout: {stdout}");
    assert!(stdout.contains("orgid: agent-task"), "stdout: {stdout}");
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
    assert!(
        stdout.contains("show: wendao-client orgize orgid-show --cached --id agent-task"),
        "stdout: {stdout}"
    );
    assert!(!stdout.contains("Closed task"), "stdout: {stdout}");
    assert!(!stdout.contains("Evidence"), "stdout: {stdout}");
}

#[test]
fn standalone_orgize_task_list_text_matches_memory_object_rows() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Memory recall target :agent:\n",
            ":PROPERTIES:\n",
            ":ID: memory-target\n",
            ":END:\n",
            "** Reflection Questions\n",
            "| Question | Value |\n",
            "|---+---|\n",
            "| Which evidence or proof should future recovery preserve? | DuckDB stores source-keyed memory object rows. |\n",
            "* TODO Unrelated task :agent:\n",
            ":PROPERTIES:\n",
            ":ID: unrelated-task\n",
            ":END:\n",
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
        .arg("DuckDB stores source")
        .arg("agenda.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize task-list: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let stderr =
        String::from_utf8(output.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    let database_path = agent_read_model_database_path(temp.path());
    assert_agent_memory_object_row_count(&database_path, 1);
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("[TASK001] Memory recall target"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("orgid: memory-target"), "stdout: {stdout}");
    assert!(!stdout.contains("Unrelated task"), "stdout: {stdout}");
}

#[test]
fn standalone_orgize_task_list_text_matches_org_elements_sql_rows() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Structural recall target :agent:memory:\n",
            ":PROPERTIES:\n",
            ":ID: structural-recall-target\n",
            ":END:\n",
            "This body paragraph is only available through org-elements SQL recall.\n",
            "* TODO Unrelated task :agent:memory:\n",
            ":PROPERTIES:\n",
            ":ID: unrelated-structural-task\n",
            ":END:\n",
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
        .arg("org-elements SQL recall")
        .arg("agenda.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize task-list: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let stderr =
        String::from_utf8(output.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    let database_path = agent_read_model_database_path(temp.path());
    assert_agent_org_element_row_count_at_least(&database_path, 1);
    assert_agent_org_element_projection_exists(
        &database_path,
        (
            "element",
            "paragraph",
            "only available through org-elements SQL recall",
        ),
    );
    assert_agent_org_element_join_matches_text(
        &database_path,
        "org-elements SQL recall",
        "structural-recall-target",
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("[TASK001] Structural recall target"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("orgid: structural-recall-target"),
        "stdout: {stdout}"
    );
    assert!(!stdout.contains("Unrelated task"), "stdout: {stdout}");
}

#[test]
fn standalone_orgize_task_list_recalls_serverless_memory_reference_fixture() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    std::fs::write(
        &agenda,
        concat!(
            "* DONE Copied Codex reference memory sample :agent:memory:\n",
            "CLOSED: [2026-05-25 Mon]\n",
            ":PROPERTIES:\n",
            ":ID: codex-reference-sample\n",
            ":SOURCE_FAMILY: codex-reference-memory-index\n",
            ":TASK_OUTCOME: success\n",
            ":PREFERENCE_SIGNAL: Prefer compact rendered snapshots over JSON-heavy agent output.\n",
            ":REUSABLE_KNOWLEDGE: The serverless recall path uses Org, DuckDB, and compact session packets.\n",
            ":FAILURE_NOTE: Runtime recall must not read ~/.codex/memories directly.\n",
            ":REFERENCE: rollout_summaries/2026-05-24T01-49-12-memory-reference.md\n",
            ":END:\n",
            "* TODO Noisy local task :agent:\n",
            ":PROPERTIES:\n",
            ":ID: noisy-task\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .env_remove("PRJ_CACHE_HOME")
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("task-list")
        .arg("--include-done")
        .arg("--text")
        .arg("compact session packets")
        .arg("agenda.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize task-list: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let stderr =
        String::from_utf8(output.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    let database_path = agent_read_model_database_path(temp.path());
    assert_agent_memory_object_row_count(&database_path, 5);
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("[TASK001] Copied Codex reference memory sample"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("orgid: codex-reference-sample"),
        "stdout: {stdout}"
    );
    assert!(!stdout.contains("recallPacket"), "stdout: {stdout}");
    assert!(!stdout.contains("memoryObjects"), "stdout: {stdout}");
    assert!(!stdout.contains("Noisy local task"), "stdout: {stdout}");
}
