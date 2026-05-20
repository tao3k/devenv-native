use std::process::Command;

use crate::orgize_runtime::support::tempdir_or_panic;

#[test]
fn standalone_orgize_sparse_tree_finds_done_achievements_without_include_flags() {
    let temp = tempdir_or_panic();
    std::fs::write(
        temp.path().join("achievement.org"),
        concat!(
            "#+TITLE: Achievement Ledger\n",
            "#+FILETAGS: :agent:achievement:\n",
            "\n",
            "* DONE Completed slice [2/2] [100%] :agent:achievement:\n",
            "SCHEDULED: <2026-05-17 Sun>\n",
            "CLOSED: [2026-05-17 Sun]\n",
            "- [X] Implementation complete.\n",
            "- [X] Validation complete.\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write achievement org: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("sparse-tree")
        .arg("--match")
        .arg("+agent+achievement")
        .arg("achievement.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize sparse-tree: {error}"));

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
        stdout.contains("[SPARSE001] Match: Completed slice [2/2] [100%]"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("state: DONE"), "stdout: {stdout}");
}

#[test]
fn standalone_orgize_sparse_tree_exclude_done_hides_completed_subtrees() {
    let temp = tempdir_or_panic();
    std::fs::write(
        temp.path().join("active_filter.org"),
        concat!(
            "#+TITLE: Active Filter\n",
            "#+FILETAGS: :agent:\n",
            "\n",
            "* DONE Completed slice :agent:\n",
            "** Context\n",
            "completed context should not leak into active recovery.\n",
            "* TODO Active slice :agent:\n",
            "** Context\n",
            "active context should remain visible.\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write active filter org: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("sparse-tree")
        .arg("--text")
        .arg("context")
        .arg("--match")
        .arg("+agent")
        .arg("--exclude-done")
        .arg("active_filter.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize sparse-tree: {error}"));

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
        stdout.contains("active context should remain visible"),
        "stdout: {stdout}"
    );
    assert!(!stdout.contains("Completed slice"), "stdout: {stdout}");
    assert!(
        !stdout.contains("completed context should not leak"),
        "stdout: {stdout}"
    );
}
