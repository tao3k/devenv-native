use std::process::Command;

use crate::orgize_runtime::support::tempdir_or_panic;

#[test]
fn standalone_orgize_sdd_status_renders_child_edges() {
    let temp = tempdir_or_panic();
    std::fs::write(
        temp.path().join("sdd.org"),
        concat!(
            "* System SDD :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11\n",
            ":SDD_KIND: system\n",
            ":SDD_STATUS: review\n",
            ":SDD_CONCERN: Agent planning architecture boundaries.\n",
            ":END:\n",
            "** Runtime View :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-55a2-70c0-98db-7ac2c4d80d78\n",
            ":SDD_KIND: view\n",
            ":SDD_PARENT: [[id:018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11][System SDD]]\n",
            ":SDD_VIEWPOINT: runtime\n",
            ":SDD_CONCERN: Recovery query and design-governance flow.\n",
            ":SDD_STATUS: review\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write sdd org: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("sdd")
        .arg("status")
        .arg("sdd.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize sdd status: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let stderr =
        String::from_utf8(output.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("architecture nodes: 2"), "stdout: {stdout}");
    assert!(
        stdout.contains("- view review: Runtime View"),
        "stdout: {stdout}"
    );
}
#[test]
fn standalone_orgize_sdd_status_reports_missing_path_recovery() {
    let temp = tempdir_or_panic();

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("sdd")
        .arg("status")
        .arg("agent/sdd")
        .output()
        .unwrap_or_else(|error| panic!("run orgize missing sdd status: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let stderr =
        String::from_utf8(output.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("missing-path"), "stdout: {stdout}");
    assert!(
        stdout.contains("copy `.agent/sdd/_architecture_template.org`"),
        "stdout: {stdout}"
    );
}
#[test]
fn standalone_orgize_sdd_status_defaults_to_agent_sdd_cache() {
    let temp = tempdir_or_panic();
    let sdd_dir = xiuxian_db_store::state::project_cache_root_from_config(
        xiuxian_db_store::state::ProjectCacheRootConfig {
            project_root: Some(temp.path().to_path_buf()),
            cache_home: Some(temp.path().join(".cache")),
            project_namespace: None,
        },
    )
    .join("agent")
    .join("sdd");
    std::fs::create_dir_all(&sdd_dir).unwrap_or_else(|error| panic!("create sdd dir: {error}"));
    std::fs::write(
        sdd_dir.join("default.org"),
        concat!(
            "* System SDD :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11\n",
            ":SDD_KIND: system\n",
            ":SDD_STATUS: review\n",
            ":SDD_CONCERN: Default active SDD lookup.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write default sdd org: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .env_remove("PRJ_CACHE_HOME")
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("sdd")
        .arg("status")
        .output()
        .unwrap_or_else(|error| panic!("run default orgize sdd status: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let stderr =
        String::from_utf8(output.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("architecture nodes: 1"), "stdout: {stdout}");
    assert!(
        stdout.contains("Default active SDD lookup"),
        "stdout: {stdout}"
    );
}
