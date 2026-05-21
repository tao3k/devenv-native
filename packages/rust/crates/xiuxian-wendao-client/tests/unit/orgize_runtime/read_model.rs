use std::process::Command;

use crate::orgize_runtime::support::{assert_cli_success, run_orgize, tempdir_or_panic};

#[cfg(feature = "orgize-agent-read-model")]
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

#[cfg(feature = "orgize-agent-read-model")]
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

#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn standalone_orgize_task_list_named_views_select_control_rows() {
    let temp = tempdir_or_panic();
    write_task_list_named_views_fixture(temp.path());

    assert_task_list_view(
        temp.path(),
        "achievement",
        &["view: achievement", "[TASK001] Achievement slice"],
        &["Archived achievement"],
    );
    assert_task_list_view(
        temp.path(),
        "archive-candidate",
        &[
            "view: archive-candidate",
            "Achievement slice",
            "Archive candidate",
        ],
        &["Performance cadence"],
    );
    assert_task_list_view(
        temp.path(),
        "closure-needed",
        &["view: closure-needed", "Completed but open"],
        &["Active task"],
    );
    assert_task_list_view(
        temp.path(),
        "repeating",
        &[
            "view: repeating",
            "Performance cadence",
            "repeat: scheduled ++1d (catchUp)",
        ],
        &[],
    );
    assert_task_list_view(
        temp.path(),
        "archived",
        &["view: archived", "Archived achievement"],
        &["Achievement slice"],
    );
}

fn write_task_list_named_views_fixture(root: &std::path::Path) {
    std::fs::write(
        root.join("agenda.org"),
        concat!(
            "* TODO Active task :agent:\n",
            "* TODO Completed but open [2/2] [100%] :agent:\n",
            "- [X] Implementation\n",
            "- [X] Validation\n",
            "* TODO Performance cadence :agent:performance:\n",
            "SCHEDULED: <2026-05-18 Mon ++1d>\n",
            "* DONE Achievement slice :agent:achievement:\n",
            "CLOSED: [2026-05-18 Mon]\n",
            "* DONE Archive candidate :agent:\n",
            "CLOSED: [2026-05-18 Mon]\n",
            "* DONE Archived achievement :agent:achievement:ARCHIVE:\n",
            "CLOSED: [2026-05-17 Sun]\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));
}

fn assert_task_list_view(root: &std::path::Path, view: &str, expected: &[&str], absent: &[&str]) {
    let output = run_orgize(
        root,
        &["task-list", "--view", view, "agenda.org"],
        &format!("task-list {view} view"),
    );
    assert_cli_success(&output);
    for needle in expected {
        assert!(output.stdout.contains(needle), "stdout: {}", output.stdout);
    }
    for needle in absent {
        assert!(!output.stdout.contains(needle), "stdout: {}", output.stdout);
    }
}

#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn standalone_orgize_task_list_uses_read_only_snapshot_when_refresh_is_locked() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    std::fs::write(&agenda, "* TODO Agent task :agent:\n")
        .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let first_refresh = run_orgize(temp.path(), &["read-model", "agenda.org"], "read-model");
    assert_cli_success(&first_refresh);
    let database_path = temp
        .path()
        .join(".cache")
        .join("agent")
        .join("readmodels")
        .join("org_agent_tasks.duckdb");
    let _writer_lock = xiuxian_db_store::duckdb_crate::Connection::open(&database_path)
        .unwrap_or_else(|error| panic!("open writer lock: {error}"));

    let list = run_orgize(temp.path(), &["task-list", "agenda.org"], "task-list");

    assert_cli_success(&list);
    assert!(
        list.stdout.contains("snapshot: in-memory-fallback"),
        "stdout: {}",
        list.stdout
    );
    assert!(
        list.stdout.contains("[TASK001] Agent task"),
        "stdout: {}",
        list.stdout
    );
}

#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn standalone_orgize_task_list_cached_reuses_existing_snapshot() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    std::fs::write(&agenda, "* TODO Cached task :agent:\n")
        .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let first_refresh = run_orgize(temp.path(), &["read-model", "agenda.org"], "read-model");
    assert_cli_success(&first_refresh);
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Cached task :agent:\n",
            "* TODO New task after snapshot :agent:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("rewrite agenda: {error}"));

    let cached = run_orgize(
        temp.path(),
        &["task-list", "--cached", "agenda.org"],
        "task-list cached",
    );
    assert_cli_success(&cached);
    assert!(
        cached.stdout.contains("snapshot: cached"),
        "stdout: {}",
        cached.stdout
    );
    assert!(
        cached.stdout.contains("rows: 1"),
        "stdout: {}",
        cached.stdout
    );
    assert!(
        cached.stdout.contains("Cached task"),
        "stdout: {}",
        cached.stdout
    );
    assert!(
        !cached.stdout.contains("New task after snapshot"),
        "stdout: {}",
        cached.stdout
    );

    let refreshed = run_orgize(
        temp.path(),
        &["task-list", "agenda.org"],
        "task-list refreshed",
    );
    assert_cli_success(&refreshed);
    assert!(
        refreshed.stdout.contains("snapshot: refreshed"),
        "stdout: {}",
        refreshed.stdout
    );
    assert!(
        refreshed.stdout.contains("rows: 2"),
        "stdout: {}",
        refreshed.stdout
    );
    assert!(
        refreshed.stdout.contains("New task after snapshot"),
        "stdout: {}",
        refreshed.stdout
    );
}

#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn standalone_orgize_task_list_cached_respects_requested_source_path() {
    let temp = tempdir_or_panic();
    let first = temp.path().join("first.org");
    let second = temp.path().join("second.org");
    std::fs::write(&first, "* TODO First cached task :agent:\n")
        .unwrap_or_else(|error| panic!("write first agenda: {error}"));
    std::fs::write(&second, "* TODO Second cached task :agent:\n")
        .unwrap_or_else(|error| panic!("write second agenda: {error}"));

    let first_refresh = run_orgize(
        temp.path(),
        &["read-model", "first.org", "second.org"],
        "read-model",
    );
    assert_cli_success(&first_refresh);

    let cached = run_orgize(
        temp.path(),
        &["task-list", "--cached", "second.org"],
        "task-list cached second",
    );
    assert_cli_success(&cached);
    assert!(
        cached.stdout.contains("snapshot: cached"),
        "stdout: {}",
        cached.stdout
    );
    assert!(
        cached.stdout.contains("rows: 1"),
        "stdout: {}",
        cached.stdout
    );
    assert!(
        cached.stdout.contains("Second cached task"),
        "stdout: {}",
        cached.stdout
    );
    assert!(
        !cached.stdout.contains("First cached task"),
        "stdout: {}",
        cached.stdout
    );
}

#[cfg(feature = "orgize-agent-read-model")]
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
#[cfg(feature = "orgize-agent-read-model")]
fn assert_agent_task_row_count(database_path: &std::path::Path, expected: i64) {
    let connection = xiuxian_db_store::duckdb_crate::Connection::open(database_path)
        .unwrap_or_else(|error| panic!("open read-model duckdb: {error}"));
    let count = connection
        .query_row("SELECT COUNT(*) FROM agent_org_tasks", [], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap_or_else(|error| panic!("query read-model row count: {error}"));
    assert_eq!(count, expected);
}
