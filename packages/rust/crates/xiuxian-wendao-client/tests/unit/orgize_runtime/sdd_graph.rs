use std::process::Command;

use crate::orgize_runtime::support::tempdir_or_panic;

#[test]
fn standalone_orgize_sdd_graph_diff_renders_aligned_edges() {
    let temp = tempdir_or_panic();
    std::fs::write(
        temp.path().join("sdd.org"),
        concat!(
            "* System SDD :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11\n",
            ":SDD_KIND: system\n",
            ":SDD_STATUS: review\n",
            ":SDD_CONCERN: Graph diff runtime alignment.\n",
            ":END:\n",
            "** Runtime View :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-55a2-70c0-98db-7ac2c4d80d78\n",
            ":SDD_KIND: view\n",
            ":SDD_PARENT: [[id:018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11][System SDD]]\n",
            ":SDD_VIEWPOINT: runtime\n",
            ":SDD_CONCERN: Parent edge matches outline nesting.\n",
            ":SDD_STATUS: review\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write aligned graph sdd org: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("sdd")
        .arg("graph-diff")
        .arg("--fail-on-drift")
        .arg("sdd.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize sdd graph-diff: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let stderr =
        String::from_utf8(output.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("[SDD-GRAPH]"), "stdout: {stdout}");
    assert!(stdout.contains("drift=0"), "stdout: {stdout}");
    assert!(
        stdout.contains("- aligned: Runtime View"),
        "stdout: {stdout}"
    );
}

#[test]
fn standalone_orgize_sdd_graph_diff_fail_on_drift_returns_failure() {
    let temp = tempdir_or_panic();
    std::fs::write(
        temp.path().join("sdd.org"),
        concat!(
            "* System A :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11\n",
            ":SDD_KIND: system\n",
            ":SDD_STATUS: review\n",
            ":SDD_CONCERN: Physical outline parent.\n",
            ":END:\n",
            "** Runtime View :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-55a2-70c0-98db-7ac2c4d80d78\n",
            ":SDD_KIND: view\n",
            ":SDD_PARENT: [[id:018f3f9c-4c78-7f24-bc2c-e1aa0d7cb881][System B]]\n",
            ":SDD_VIEWPOINT: runtime\n",
            ":SDD_CONCERN: Semantic parent differs from outline parent.\n",
            ":SDD_STATUS: review\n",
            ":END:\n",
            "* System B :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-4c78-7f24-bc2c-e1aa0d7cb881\n",
            ":SDD_KIND: system\n",
            ":SDD_STATUS: review\n",
            ":SDD_CONCERN: Semantic parent.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write moved graph sdd org: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("sdd")
        .arg("graph-diff")
        .arg("--fail-on-drift")
        .arg("sdd.org")
        .output()
        .unwrap_or_else(|error| panic!("run drift orgize sdd graph-diff: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let stderr =
        String::from_utf8(output.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("- semantic-move: Runtime View"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("semantic: System B"), "stdout: {stdout}");
    assert!(stdout.contains("outline: System A"), "stdout: {stdout}");
}
