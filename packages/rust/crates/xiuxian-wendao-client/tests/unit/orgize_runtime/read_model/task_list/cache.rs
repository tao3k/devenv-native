use crate::orgize_runtime::support::{assert_cli_success, run_orgize, tempdir_or_panic};

use crate::orgize_runtime::read_model::agent_read_model_database_path;

#[test]
fn standalone_orgize_task_list_uses_read_only_snapshot_when_refresh_is_locked() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    std::fs::write(
        &agenda,
        "* TODO Agent task :agent:\n:PROPERTIES:\n:ID: agent-task\n:END:\n",
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let first_refresh = run_orgize(temp.path(), &["read-model", "agenda.org"], "read-model");
    assert_cli_success(&first_refresh);
    let database_path = agent_read_model_database_path(temp.path());
    let _writer_lock = xiuxian_db_store::duckdb_crate::Connection::open(&database_path)
        .unwrap_or_else(|error| panic!("open writer lock: {error}"));

    let list = run_orgize(temp.path(), &["task-list", "agenda.org"], "task-list");

    assert_cli_success(&list);
    assert!(
        list.stdout.contains("[TASK001] Agent task"),
        "stdout: {}",
        list.stdout
    );
}

#[test]
fn standalone_orgize_task_list_cached_reuses_existing_snapshot() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    std::fs::write(
        &agenda,
        "* TODO Cached task :agent:\n:PROPERTIES:\n:ID: cached-task\n:END:\n",
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let first_refresh = run_orgize(temp.path(), &["read-model", "agenda.org"], "read-model");
    assert_cli_success(&first_refresh);
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Cached task :agent:\n",
            ":PROPERTIES:\n",
            ":ID: cached-task\n",
            ":END:\n",
            "* TODO New task after snapshot :agent:\n",
            ":PROPERTIES:\n",
            ":ID: new-task\n",
            ":END:\n",
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
        refreshed.stdout.contains("New task after snapshot"),
        "stdout: {}",
        refreshed.stdout
    );
}

#[test]
fn standalone_orgize_task_list_cached_respects_requested_source_path() {
    let temp = tempdir_or_panic();
    let first = temp.path().join("first.org");
    let second = temp.path().join("second.org");
    std::fs::write(
        &first,
        "* TODO First cached task :agent:\n:PROPERTIES:\n:ID: first-cached\n:END:\n",
    )
    .unwrap_or_else(|error| panic!("write first agenda: {error}"));
    std::fs::write(
        &second,
        "* TODO Second cached task :agent:\n:PROPERTIES:\n:ID: second-cached\n:END:\n",
    )
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
