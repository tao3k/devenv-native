use super::support::{
    answered_closure_questions, answered_reflection_questions, run_lint_for_agent_heading_and_body,
    run_lint_for_raw_agent_source, tempdir_or_panic,
};
use std::process::Command;

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
            "#+DATE: 2026-05-24 Sun 12:27:45\n",
            "\n",
            "* TODO Agent slice [0/4] [0%] :agent:\n",
            ":PROPERTIES:\n",
            ":SDD: <sdd-path-or-none>\n",
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
#[test]
fn standalone_orgize_lint_accepts_agent_org_date_template_placeholder() {
    let output = run_lint_for_raw_agent_source(concat!(
        "#+TITLE: Agent Template\n",
        "#+AUTHOR: CyberXiuXian Artisan workshop\n",
        "#+FILETAGS: :agent:\n",
        "#+DATE: YYYY-MM-DD Day HH:MM:SS\n",
        "\n",
        "* TODO Agent slice [0/2] [0%] :agent:\n",
        ":PROPERTIES:\n",
        ":ID: test-agent-task\n",
        ":END:\n",
        "- [ ] one\n",
        "- [ ] two\n",
    ));
    assert_eq!(output.status_code, Some(0), "stdout: {}", output.stdout);
    assert_eq!(output.stdout, "[ok] orgize lint\n");
}
#[test]
fn standalone_orgize_lint_accepts_reflection_questions_for_done_agent_task() {
    let body = format!("- [X] one\n- [X] two\n{}", answered_reflection_questions());
    let output = run_lint_for_agent_heading_and_body(
        "* DONE Agent slice [2/2] [100%] :agent:ARCHIVE:\nCLOSED: [2026-05-24 Sun]\n",
        &body,
    );
    assert_eq!(output.status_code, Some(0), "stdout: {}", output.stdout);
    assert_eq!(output.stdout, "[ok] orgize lint\n");
}
#[test]
fn standalone_orgize_lint_accepts_closed_answered_agent_task_after_archive() {
    let body = format!("- [X] one\n- [X] two\n{}", answered_closure_questions());
    let output = run_lint_for_agent_heading_and_body(
        "* DONE Agent slice [2/2] [100%] :agent:ARCHIVE:\nCLOSED: [2026-05-24 Sun]\n",
        &body,
    );
    assert_eq!(output.status_code, Some(0), "stdout: {}", output.stdout);
    assert!(
        !output.stdout.contains("agent-task-archive-candidate"),
        "stdout: {}",
        output.stdout
    );
}
#[test]
fn standalone_orgize_lint_accepts_raw_archive_subtree_sink() {
    let temp = tempdir_or_panic();
    let archive_dir = temp.path().join("archives");
    std::fs::create_dir_all(&archive_dir)
        .unwrap_or_else(|error| panic!("create archive dir: {error}"));
    std::fs::write(
        archive_dir.join("completed_task.org"),
        concat!(
            "* DONE First archived slice [1/1] [100%] :agent:ARCHIVE:\n",
            "CLOSED: [2026-05-24 Sun]\n",
            ":PROPERTIES:\n",
            ":ID: first-archived-slice\n",
            ":END:\n",
            "- [X] one\n",
            "** Closure Questions\n",
            "| Question | Value |\n",
            "|---+---|\n",
            "| Did the task meet its expected outcome? | First slice landed. |\n",
            "* DONE Second archived slice [1/1] [100%] :agent:ARCHIVE:\n",
            "CLOSED: [2026-05-24 Sun]\n",
            ":PROPERTIES:\n",
            ":ID: second-archived-slice\n",
            ":END:\n",
            "- [X] one\n",
            "** Closure Questions\n",
            "| Question | Value |\n",
            "|---+---|\n",
            "| Did the task meet its expected outcome? | Second slice landed. |\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write archive: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("lint")
        .arg("--format")
        .arg("compact")
        .arg("archives/completed_task.org")
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
