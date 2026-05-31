use super::support::{
    answered_closure_questions, assert_agent_org_date_has_seconds, run_lint_fix_for_agent_source,
    run_lint_fix_for_raw_agent_source, tempdir_or_panic,
};
use std::process::Command;

#[test]
fn standalone_orgize_lint_fix_adds_missing_agent_task_id() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    std::fs::write(
        &agenda,
        concat!(
            "#+TITLE: Agent task\n",
            "#+AUTHOR: CyberXiuXian Artisan workshop\n",
            "#+FILETAGS: :agent:\n",
            "#+DATE: 2026-05-24 Sun 12:27:45\n",
            "\n",
            "* TODO Agent task :agent:\n",
            "SCHEDULED: <2026-05-23 Sat>\n",
            ":PROPERTIES:\n",
            ":NEXT_ACTION: Continue task\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("lint")
        .arg("--fix")
        .arg("--format")
        .arg("compact")
        .arg("agenda.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize lint --fix: {error}"));

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
        stdout.contains("[fixed] orgize lint: added 1 missing ID properties"),
        "stdout: {stdout}"
    );
    let updated =
        std::fs::read_to_string(&agenda).unwrap_or_else(|error| panic!("read agenda: {error}"));
    assert!(updated.contains(":ID: "), "updated: {updated}");
    assert!(
        updated.contains(":ID: ") && updated.contains(":NEXT_ACTION: Continue task"),
        "updated: {updated}"
    );
}
#[test]
fn standalone_orgize_lint_fix_adds_missing_agent_org_metadata() {
    let output = run_lint_fix_for_raw_agent_source(concat!(
        "* TODO Agent slice [0/2] [0%] :agent:\n",
        ":PROPERTIES:\n",
        ":ID: test-agent-task\n",
        ":END:\n",
        "- [ ] one\n",
        "- [ ] two\n",
    ));
    assert_eq!(output.status_code, Some(0), "stdout: {}", output.stdout);
    assert!(
        output.stdout.contains("fixed 4 agent Org metadata lines"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output
            .updated
            .starts_with("#+TITLE: Agent slice\n#+AUTHOR: CyberXiuXian Artisan workshop\n#+FILETAGS: :agent:\n#+DATE: "),
        "updated: {}",
        output.updated
    );
    assert_agent_org_date_has_seconds(&output.updated);
}
#[test]
fn standalone_orgize_lint_fix_updates_agent_org_date_precision() {
    let output = run_lint_fix_for_raw_agent_source(concat!(
        "#+TITLE: Agent Template\n",
        "#+AUTHOR: CyberXiuXian Artisan workshop\n",
        "#+FILETAGS: :agent:\n",
        "#+DATE: 2026-05-24 Sun\n",
        "\n",
        "* TODO Agent slice [0/2] [0%] :agent:\n",
        ":PROPERTIES:\n",
        ":ID: test-agent-task\n",
        ":END:\n",
        "- [ ] one\n",
        "- [ ] two\n",
    ));
    assert_eq!(output.status_code, Some(0), "stdout: {}", output.stdout);
    assert!(
        output.stdout.contains("fixed 1 agent Org metadata lines"),
        "stdout: {}",
        output.stdout
    );
    assert!(!output.updated.contains("#+DATE: 2026-05-24 Sun\n"));
    assert_agent_org_date_has_seconds(&output.updated);
}
#[test]
fn standalone_orgize_lint_fix_removes_redundant_agent_properties() {
    let output = run_lint_fix_for_agent_source(concat!(
        "* TODO Agent slice [0/2] [0%] :agent:\n",
        ":PROPERTIES:\n",
        ":ID: test-agent-task\n",
        ":STATUS: active\n",
        ":EXECPLAN: none\n",
        ":NEXT_ACTION: Continue task\n",
        ":END:\n",
        "- [ ] one\n",
        "- [ ] two\n",
        ":STATUS: body marker, not a property drawer entry\n",
    ));
    assert_eq!(output.status_code, Some(0), "stdout: {}", output.stdout);
    assert!(
        output.stdout.contains("removed 2 redundant properties"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        !output.updated.contains(":STATUS: active")
            && !output.updated.contains(":EXECPLAN: none")
            && output.updated.contains(":NEXT_ACTION: Continue task")
            && output
                .updated
                .contains(":STATUS: body marker, not a property drawer entry"),
        "updated: {}",
        output.updated
    );
}
#[test]
fn standalone_orgize_lint_fix_updates_started_agent_task_to_doing() {
    let output = run_lint_fix_for_agent_source(concat!(
        "* TODO Agent slice [1/2] [50%] :agent:\n",
        ":PROPERTIES:\n",
        ":ID: test-agent-task\n",
        ":END:\n",
        "- [X] one\n",
        "- [ ] two\n",
    ));
    assert_eq!(output.status_code, Some(0), "stdout: {}", output.stdout);
    assert!(
        output.stdout.contains("updated 1 lifecycle keywords"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output
            .updated
            .contains("* DOING Agent slice [1/2] [50%] :agent:\n"),
        "updated: {}",
        output.updated
    );
}
#[test]
fn standalone_orgize_lint_fix_converts_closed_timestamp_to_inactive() {
    let source = format!(
        concat!(
            "* DONE Agent slice [2/2] [100%] :agent:ARCHIVE:\n",
            "CLOSED: <2026-05-24 Sun>\n",
            ":PROPERTIES:\n",
            ":ID: test-agent-task\n",
            ":END:\n",
            "- [X] one\n",
            "- [X] two\n",
            "{}",
        ),
        answered_closure_questions(),
    );
    let output = run_lint_fix_for_agent_source(&source);
    assert_eq!(output.status_code, Some(0), "stdout: {}", output.stdout);
    assert!(
        output.stdout.contains("fixed 1 CLOSED timestamps"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output.updated.contains("CLOSED: [2026-05-24 Sun]")
            && !output.updated.contains("CLOSED: <2026-05-24 Sun>"),
        "updated: {}",
        output.updated
    );
}
