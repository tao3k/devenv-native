use std::{path::Path, process::Command};

use crate::orgize_runtime::support::tempdir_or_panic;

use super::{
    assert_agent_memory_object_projection, assert_agent_memory_object_row_count,
    assert_agent_memory_object_row_count_for_orgid, assert_agent_org_element_projection_exists,
    assert_agent_org_element_row_count_at_least, assert_agent_task_row_count,
};

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
            ":ID: agent-task\n",
            ":NEXT_ACTION: Verify DuckDB snapshot\n",
            ":CLAIM: Org properties feed the memory object read model.\n",
            ":END:\n",
            "** Evidence\n",
            "This inherited agent-tag section should not become a task row.\n",
            "** TODO Task Checklist\n",
            "This inherited agent-tag TODO section should not become a task row.\n",
            "** Reflection Questions\n",
            "| Question | Value |\n",
            "|---+---|\n",
            "| Which evidence or proof should future recovery preserve? | DuckDB stores source-keyed memory object rows. |\n",
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
        .arg("read-model")
        .arg("agenda.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize read-model: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    assert_eq!(output.status.code(), Some(0), "stdout: {stdout}");
    assert!(stdout.contains("backend: duckdb"), "stdout: {stdout}");
    assert!(
        stdout.contains("memory-table: agent_org_memory_objects"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("element-table: agent_org_elements"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("rows: 2"), "stdout: {stdout}");
    assert!(stdout.contains("memory-objects: 2"), "stdout: {stdout}");
    assert!(stdout.contains("org-elements: "), "stdout: {stdout}");
    let database_path = temp
        .path()
        .join(".cache")
        .join("agent")
        .join("readmodels")
        .join("org_agent_tasks.duckdb");
    assert!(database_path.is_file());
    assert_agent_task_row_count(&database_path, 2);
    assert_agent_memory_object_row_count(&database_path, 2);
    assert_agent_org_element_row_count_at_least(&database_path, 1);
    assert_agent_org_element_projection_exists(
        &database_path,
        (
            "property",
            "node-property",
            "Org properties feed the memory object read model.",
        ),
    );
    assert_agent_memory_object_projection(
        &database_path,
        1,
        (
            "agent-task",
            "claim",
            "memory-claim",
            "property",
            "CLAIM",
            "Org properties feed the memory object read model.",
        ),
    );
    assert_agent_memory_object_projection(
        &database_path,
        2,
        (
            "agent-task",
            "evidence",
            "memory-evidence",
            "reflection",
            "Which evidence or proof should future recovery preserve?",
            "DuckDB stores source-keyed memory object rows.",
        ),
    );
}

#[test]
fn standalone_orgize_read_model_materializes_serverless_memory_reference_fixture() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    write_serverless_memory_reference_fixture(&agenda);
    let output = run_read_model(temp.path());

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let stderr =
        String::from_utf8(output.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("memory-objects: 5"), "stdout: {stdout}");
    let database_path = temp
        .path()
        .join(".cache")
        .join("agent")
        .join("readmodels")
        .join("org_agent_tasks.duckdb");
    assert_agent_task_row_count(&database_path, 2);
    assert_agent_memory_object_row_count(&database_path, 5);
    assert_agent_memory_object_row_count_for_orgid(&database_path, "superseded-memory-sample", 0);
    assert_serverless_memory_object_projection(&database_path);
}

fn write_serverless_memory_reference_fixture(agenda: &Path) {
    std::fs::write(
        agenda,
        concat!(
            "* DONE Copied Codex reference memory sample :agent:memory:\n",
            "CLOSED: [2026-05-25 Mon]\n",
            ":PROPERTIES:\n",
            ":ID: codex-reference-sample\n",
            ":SOURCE_FAMILY: codex-reference-raw-rollout\n",
            ":TASK_OUTCOME: success\n",
            ":PREFERENCE_SIGNAL: Prefer compact rendered snapshots over JSON-heavy agent output.\n",
            ":REUSABLE_KNOWLEDGE: The serverless recall path uses Org, DuckDB, and compact session packets.\n",
            ":FAILURE_NOTE: Runtime recall must not read ~/.codex/memories directly.\n",
            ":REFERENCE: rollout_summaries/2026-05-24T01-49-12-memory-reference.md\n",
            ":END:\n",
            "* DONE Superseded memory sample :agent:memory:\n",
            "CLOSED: [2026-05-25 Mon]\n",
            ":PROPERTIES:\n",
            ":ID: superseded-memory-sample\n",
            ":MEMORY_STATUS: superseded\n",
            ":TASK_OUTCOME: success\n",
            ":REUSABLE_KNOWLEDGE: This stale compact session packet claim must not project.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));
}

fn run_read_model(root: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .env_remove("PRJ_CACHE_HOME")
        .arg("--root")
        .arg(root)
        .arg("orgize")
        .arg("read-model")
        .arg("agenda.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize read-model: {error}"))
}

fn assert_serverless_memory_object_projection(database_path: &Path) {
    assert_agent_memory_object_projection(
        database_path,
        1,
        (
            "codex-reference-sample",
            "finality",
            "memory-finality",
            "property",
            "TASK_OUTCOME",
            "success",
        ),
    );
    assert_agent_memory_object_projection(
        database_path,
        2,
        (
            "codex-reference-sample",
            "preference",
            "memory-preference",
            "property",
            "PREFERENCE_SIGNAL",
            "Prefer compact rendered snapshots over JSON-heavy agent output.",
        ),
    );
    assert_agent_memory_object_projection(
        database_path,
        3,
        (
            "codex-reference-sample",
            "claim",
            "memory-claim",
            "property",
            "REUSABLE_KNOWLEDGE",
            "The serverless recall path uses Org, DuckDB, and compact session packets.",
        ),
    );
    assert_agent_memory_object_projection(
        database_path,
        4,
        (
            "codex-reference-sample",
            "failure",
            "memory-failure",
            "property",
            "FAILURE_NOTE",
            "Runtime recall must not read ~/.codex/memories directly.",
        ),
    );
    assert_agent_memory_object_projection(
        database_path,
        5,
        (
            "codex-reference-sample",
            "evidence",
            "memory-evidence",
            "property",
            "REFERENCE",
            "rollout_summaries/2026-05-24T01-49-12-memory-reference.md",
        ),
    );
}
