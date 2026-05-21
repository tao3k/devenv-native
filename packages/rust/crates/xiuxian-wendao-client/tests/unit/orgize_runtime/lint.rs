use std::process::Command;

use crate::orgize_runtime::support::tempdir_or_panic;

#[test]
fn standalone_orgize_lint_accepts_clean_org_file() {
    let temp = tempdir_or_panic();
    std::fs::write(temp.path().join("agenda.org"), "* TODO Agent task\n")
        .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("lint")
        .arg("--format")
        .arg("compact")
        .arg("agenda.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize lint: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout, "[ok] orgize lint\n");
}

#[test]
fn standalone_orgize_lint_accepts_agent_progress_cookie_template() {
    let temp = tempdir_or_panic();
    std::fs::write(
        temp.path().join("agent.org"),
        concat!(
            "#+TITLE: Agent Template\n",
            "#+AUTHOR: CyberXiuXian Artisan workshop\n",
            "#+FILETAGS: :agent:\n",
            "\n",
            "* TODO Agent slice [0/4] [0%] :agent:\n",
            ":PROPERTIES:\n",
            ":SDD: <sdd-path-or-none>\n",
            ":EXECPLAN: <execplan-path>\n",
            ":STATUS: active\n",
            ":COOKIE_DATA: direct\n",
            ":END:\n",
            "SCHEDULED: <2026-05-17 Sun>\n",
            "\n",
            "- [ ] Scope confirmed.\n",
            "- [ ] Implementation complete.\n",
            "- [ ] Validation complete.\n",
            "\n",
            "** TODO Task Checklist [0/2] [0%]\n",
            "- [ ] Targeted tests passed.\n",
            "- [ ] Recovery query checked.\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agent org: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("lint")
        .arg("--format")
        .arg("compact")
        .arg("agent.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize lint: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let stderr =
        String::from_utf8(output.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert_eq!(stdout, "[ok] orgize lint\n");
}
