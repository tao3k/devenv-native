use crate::orgize_runtime::support::{assert_cli_success, run_orgize, tempdir_or_panic};

#[test]
fn standalone_orgize_task_recover_renders_recent_orgid_candidates() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Recent candidate :agent:flowhub:\n",
            ":PROPERTIES:\n",
            ":ID: recent-candidate\n",
            ":NEXT_ACTION: Continue the current reasoning branch\n",
            ":END:\n",
            "* TODO Other candidate :agent:\n",
            ":PROPERTIES:\n",
            ":ID: other-candidate\n",
            ":END:\n",
            "* DONE Closed candidate :agent:flowhub:\n",
            "CLOSED: [2026-05-23 Sat]\n",
            ":PROPERTIES:\n",
            ":ID: closed-candidate\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = run_orgize(
        temp.path(),
        &[
            "task-recover",
            "--text",
            "flowhub",
            "--limit",
            "5",
            "agenda.org",
        ],
        "task-recover",
    );

    assert_cli_success(&output);
    assert!(
        output.stdout.contains("title: Recent candidate"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        !output.stdout.contains("[RECENT"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        !output.stdout.contains("orgid: recent-candidate"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        !output.stdout.contains("file-key: agenda"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        !output.stdout.contains("query-file:"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        !output.stdout.contains("query-title:"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        !output.stdout.contains("source-modified-unix-ms:"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        !output.stdout.contains("state: TODO"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output
            .stdout
            .contains("show: wendao-client orgize ogrid-show --cached --id recent-candidate"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        !output.stdout.contains("Closed candidate"),
        "stdout: {}",
        output.stdout
    );
}

#[test]
fn standalone_orgize_task_recover_skips_closure_needed_noise_by_default() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Complete but unclosed [3/3] [100%] :agent:flowhub:\n",
            ":PROPERTIES:\n",
            ":ID: complete-unclosed\n",
            ":NEXT_ACTION: This should be archived, not recovered as active work.\n",
            ":END:\n",
            "* TODO Still active [2/3] [66%] :agent:flowhub:\n",
            ":PROPERTIES:\n",
            ":ID: still-active\n",
            ":NEXT_ACTION: Continue active work.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = run_orgize(
        temp.path(),
        &[
            "task-recover",
            "--text",
            "flowhub",
            "--limit",
            "5",
            "agenda.org",
        ],
        "task-recover closure noise",
    );

    assert_cli_success(&output);
    assert!(
        output.stdout.contains("title: Still active [2/3] [66%]"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        !output.stdout.contains("Complete but unclosed"),
        "stdout: {}",
        output.stdout
    );
}
