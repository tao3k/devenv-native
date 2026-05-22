use crate::orgize_runtime::support::{assert_cli_success, run_orgize, tempdir_or_panic};

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
