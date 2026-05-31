use super::support::{
    answered_closure_questions, run_lint_for_agent_heading, run_lint_for_agent_heading_and_body,
    run_lint_for_agent_source, run_lint_for_raw_agent_source, unanswered_closure_questions,
};

#[test]
fn standalone_orgize_lint_warns_when_agent_org_file_metadata_is_missing() {
    let output = run_lint_for_raw_agent_source(concat!(
        "* TODO Agent slice [0/2] [0%] :agent:\n",
        ":PROPERTIES:\n",
        ":ID: test-agent-task\n",
        ":END:\n",
        "- [ ] one\n",
        "- [ ] two\n",
    ));
    assert_eq!(output.status_code, Some(1), "stdout: {}", output.stdout);
    for code in [
        "agent-org-title-missing",
        "agent-org-author-missing",
        "agent-org-filetags-missing",
        "agent-org-date-missing",
    ] {
        assert!(output.stdout.contains(code), "stdout: {}", output.stdout);
    }
}
#[test]
fn standalone_orgize_lint_warns_when_agent_org_date_lacks_seconds() {
    let output = run_lint_for_raw_agent_source(concat!(
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
    assert_eq!(output.status_code, Some(1), "stdout: {}", output.stdout);
    assert!(
        output.stdout.contains("agent-org-date-precision"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output.stdout.contains("YYYY-MM-DD Day HH:MM:SS"),
        "stdout: {}",
        output.stdout
    );
}
#[test]
fn standalone_orgize_lint_warns_when_sdd_tracking_file_date_lacks_seconds() {
    let output = run_lint_for_raw_agent_source(concat!(
        "#+TITLE: SDD Template\n",
        "#+AUTHOR: CyberXiuXian Artisan workshop\n",
        "#+FILETAGS: :agent:sdd:architecture:\n",
        "#+DATE: 2026-05-24 Sun\n",
        "\n",
        "* System SDD :sdd:system:\n",
        ":PROPERTIES:\n",
        ":ID: test-sdd\n",
        ":SDD_KIND: system\n",
        ":SDD_STATUS: draft\n",
        ":END:\n",
    ));
    assert_eq!(output.status_code, Some(1), "stdout: {}", output.stdout);
    assert!(
        output.stdout.contains("agent-org-date-precision"),
        "stdout: {}",
        output.stdout
    );
}
#[test]
fn standalone_orgize_lint_warns_when_agent_task_uses_status_property() {
    let output = run_lint_for_agent_source(concat!(
        "* TODO Agent slice [0/2] [0%] :agent:\n",
        ":PROPERTIES:\n",
        ":ID: test-agent-task\n",
        ":STATUS: active\n",
        ":END:\n",
        "- [ ] one\n",
        "- [ ] two\n",
    ));
    assert_eq!(output.status_code, Some(1), "stdout: {}", output.stdout);
    assert!(
        output.stdout.contains("agent-task-status-property"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output.stdout.contains("redundant STATUS property"),
        "stdout: {}",
        output.stdout
    );
}
#[test]
fn standalone_orgize_lint_warns_when_agent_task_uses_execplan_property() {
    let output = run_lint_for_agent_source(concat!(
        "* TODO Agent slice [0/2] [0%] :agent:\n",
        ":PROPERTIES:\n",
        ":ID: test-agent-task\n",
        ":EXECPLAN: none\n",
        ":END:\n",
        "- [ ] one\n",
        "- [ ] two\n",
    ));
    assert_eq!(output.status_code, Some(1), "stdout: {}", output.stdout);
    assert!(
        output.stdout.contains("agent-task-execplan-property"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output.stdout.contains("redundant EXECPLAN property"),
        "stdout: {}",
        output.stdout
    );
}
#[test]
fn standalone_orgize_lint_warns_when_progressed_agent_task_is_still_todo() {
    let output = run_lint_for_agent_heading("* TODO Agent slice [1/4] [25%] :agent:\n");
    assert_eq!(output.status_code, Some(1), "stdout: {}", output.stdout);
    assert!(
        output.stdout.contains("agent-task-progress-state"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output
            .stdout
            .contains("change lifecycle state from TODO to DOING"),
        "stdout: {}",
        output.stdout
    );
}
#[test]
fn standalone_orgize_lint_warns_when_complete_agent_task_is_not_done() {
    let output = run_lint_for_agent_heading("* DOING Agent slice [4/4] [100%] :agent:\n");
    assert_eq!(output.status_code, Some(1), "stdout: {}", output.stdout);
    assert!(
        output.stdout.contains("agent-task-progress-complete"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output
            .stdout
            .contains("change lifecycle state to DONE, add inactive CLOSED: [YYYY-MM-DD Day], and complete Reflection Questions"),
        "stdout: {}",
        output.stdout
    );
}
#[test]
fn standalone_orgize_lint_warns_when_done_agent_task_is_missing_closed() {
    let output = run_lint_for_agent_heading("* DONE Agent slice [4/4] [100%] :agent:\n");
    assert_eq!(output.status_code, Some(1), "stdout: {}", output.stdout);
    assert!(
        output.stdout.contains("agent-task-closed-missing"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output.stdout.contains("missing CLOSED"),
        "stdout: {}",
        output.stdout
    );
}
#[test]
fn standalone_orgize_lint_warns_when_done_agent_task_is_missing_closure_questions() {
    let output = run_lint_for_agent_heading_and_body(
        "* DONE Agent slice [4/4] [100%] :agent:\nCLOSED: [2026-05-24 Sun]\n",
        "- [X] one\n- [X] two\n",
    );
    assert_eq!(output.status_code, Some(1), "stdout: {}", output.stdout);
    assert!(
        output
            .stdout
            .contains("agent-task-closure-questions-missing"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output
            .stdout
            .contains("missing Reflection Questions answers"),
        "stdout: {}",
        output.stdout
    );
}
#[test]
fn standalone_orgize_lint_warns_when_closure_question_value_is_empty() {
    let body = format!("- [X] one\n- [X] two\n{}", unanswered_closure_questions());
    let output = run_lint_for_agent_heading_and_body(
        "* DONE Agent slice [4/4] [100%] :agent:\nCLOSED: [2026-05-24 Sun]\n",
        &body,
    );
    assert_eq!(output.status_code, Some(1), "stdout: {}", output.stdout);
    assert!(
        output
            .stdout
            .contains("agent-task-closure-questions-missing"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output.stdout.contains("non-empty Value cells"),
        "stdout: {}",
        output.stdout
    );
}
#[test]
fn standalone_orgize_lint_warns_when_closed_timestamp_is_active() {
    let body = format!("- [X] one\n- [X] two\n{}", answered_closure_questions());
    let output = run_lint_for_agent_heading_and_body(
        "* DONE Agent slice [4/4] [100%] :agent:\nCLOSED: <2026-05-24 Sun>\n",
        &body,
    );
    assert_eq!(output.status_code, Some(1), "stdout: {}", output.stdout);
    assert!(
        output.stdout.contains("agent-task-closed-active-timestamp"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output.stdout.contains("use CLOSED: [YYYY-MM-DD Day]"),
        "stdout: {}",
        output.stdout
    );
}
#[test]
fn standalone_orgize_lint_warns_when_closed_answered_agent_task_needs_archive() {
    let body = format!("- [X] one\n- [X] two\n{}", answered_closure_questions());
    let output = run_lint_for_agent_heading_and_body(
        "* DONE Agent slice [4/4] [100%] :agent:\nCLOSED: [2026-05-24 Sun]\n",
        &body,
    );
    assert_eq!(output.status_code, Some(1), "stdout: {}", output.stdout);
    assert!(
        output.stdout.contains("agent-task-archive-candidate"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output
            .stdout
            .contains("archive it to keep memory recovery clean"),
        "stdout: {}",
        output.stdout
    );
}
