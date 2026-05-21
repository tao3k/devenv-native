use std::process::Command;

use crate::orgize_runtime::support::tempdir_or_panic;

#[test]
fn standalone_orgize_sdd_status_renders_json_contract() {
    let temp = tempdir_or_panic();
    std::fs::write(
        temp.path().join("sdd.org"),
        concat!(
            "* System SDD :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11\n",
            ":SDD_KIND: system\n",
            ":SDD_STATUS: review\n",
            ":SDD_CONCERN: JSON SDD status contract.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write json sdd org: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("sdd")
        .arg("status")
        .arg("--json")
        .arg("sdd.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize sdd status json: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let stderr =
        String::from_utf8(output.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    let payload: serde_json::Value =
        serde_json::from_str(&stdout).unwrap_or_else(|error| panic!("json parse: {error}"));
    assert_eq!(payload["format"], "orgize.sdd.status.v1");
    assert_eq!(payload["files"][0]["architectureNodes"], 1);
    assert_eq!(payload["files"][0]["summary"]["issues"], 0);
}
#[test]
fn standalone_orgize_sdd_status_filters_issues_only_json() {
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
        .arg("--json")
        .arg("--issues-only")
        .arg(".")
        .output()
        .unwrap_or_else(|error| panic!("run orgize sdd status json issues-only: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let stderr =
        String::from_utf8(output.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    let payload: serde_json::Value =
        serde_json::from_str(&stdout).unwrap_or_else(|error| panic!("json parse: {error}"));
    assert_eq!(payload["files"].as_array().map(Vec::len), Some(1));
    assert_eq!(payload["files"][0]["nodes"][0]["title"], "Drifted View");
    assert!(
        payload["files"][0]["summary"]["issues"]
            .as_u64()
            .unwrap_or(0)
            > 0
    );
}
