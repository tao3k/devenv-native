use std::process::Command;

use crate::orgize_runtime::support::tempdir_or_panic;

#[test]
fn standalone_orgize_sdd_status_filters_issues_only_text() {
    let temp = tempdir_or_panic();
    std::fs::write(
        temp.path().join("clean.org"),
        concat!(
            "* Clean System :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11\n",
            ":SDD_KIND: system\n",
            ":SDD_STATUS: review\n",
            ":SDD_CONCERN: Clean status should be filtered.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write clean sdd org: {error}"));
    std::fs::write(
        temp.path().join("drifted.org"),
        concat!(
            "* Drifted View :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-55a2-70c0-98db-7ac2c4d80d78\n",
            ":SDD_KIND: view\n",
            ":SDD_STATUS: review\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write drifted sdd org: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("sdd")
        .arg("status")
        .arg("--issues-only")
        .arg(".")
        .output()
        .unwrap_or_else(|error| panic!("run orgize sdd status issues-only: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let stderr =
        String::from_utf8(output.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(!stdout.contains("Clean System"), "stdout: {stdout}");
    assert!(stdout.contains("Drifted View"), "stdout: {stdout}");
    assert!(stdout.contains("[missing-parent]"), "stdout: {stdout}");
    assert!(!stdout.contains("tree:"), "stdout: {stdout}");
}
#[test]
fn standalone_orgize_sdd_status_fail_on_issues_returns_failure() {
    let temp = tempdir_or_panic();
    std::fs::write(
        temp.path().join("drifted.org"),
        concat!(
            "* Drifted View :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-55a2-70c0-98db-7ac2c4d80d78\n",
            ":SDD_KIND: view\n",
            ":SDD_STATUS: review\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write drifted sdd org: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("sdd")
        .arg("status")
        .arg("--fail-on-issues")
        .arg("drifted.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize sdd status fail-on-issues: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let stderr =
        String::from_utf8(output.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("[missing-parent]"), "stdout: {stdout}");
}
#[test]
fn standalone_orgize_sdd_status_fail_on_issues_accepts_clean_status() {
    let temp = tempdir_or_panic();
    std::fs::write(
        temp.path().join("clean.org"),
        concat!(
            "* Clean System :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11\n",
            ":SDD_KIND: system\n",
            ":SDD_STATUS: review\n",
            ":SDD_CONCERN: Clean gate should pass.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write clean sdd org: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("sdd")
        .arg("status")
        .arg("--fail-on-issues")
        .arg("clean.org")
        .output()
        .unwrap_or_else(|error| panic!("run clean orgize sdd status fail-on-issues: {error}"));

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
        stdout.contains("diagnostics:\n- no issues"),
        "stdout: {stdout}"
    );
}
