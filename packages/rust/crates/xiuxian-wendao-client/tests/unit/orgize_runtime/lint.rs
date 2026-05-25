use std::process::Command;

use crate::orgize_runtime::support::tempdir_or_panic;

fn answered_closure_questions() -> &'static str {
    concat!(
        "** Closure Questions\n",
        "| Question | Value |\n",
        "|---+---|\n",
        "| Did the task meet its expected outcome? | Yes, the slice landed and targeted validation passed. |\n",
    )
}

fn unanswered_closure_questions() -> &'static str {
    concat!(
        "** Closure Questions\n",
        "| Question | Value |\n",
        "|---+---|\n",
        "| Did the task meet its expected outcome? | |\n",
    )
}

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
            .contains("change lifecycle state to DONE, add inactive CLOSED: [YYYY-MM-DD Day], and complete Closure Questions"),
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
        output.stdout.contains("missing Closure Questions answers"),
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

struct LintOutput {
    stdout: String,
    status_code: Option<i32>,
}

struct FixLintOutput {
    stdout: String,
    status_code: Option<i32>,
    updated: String,
}

fn run_lint_for_agent_heading(heading: &str) -> LintOutput {
    run_lint_for_agent_heading_and_body(heading, "- [X] one\n- [ ] two\n")
}

fn run_lint_for_agent_heading_and_body(heading: &str, body: &str) -> LintOutput {
    run_lint_for_agent_source(&format!(
        "{heading}:PROPERTIES:\n:ID: test-agent-task\n:END:\n{body}"
    ))
}

fn run_lint_for_agent_source(source: &str) -> LintOutput {
    run_lint_for_raw_agent_source(&agent_org_source(source))
}

fn run_lint_for_raw_agent_source(source: &str) -> LintOutput {
    let temp = tempdir_or_panic();
    std::fs::write(temp.path().join("agent.org"), source)
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

    LintOutput {
        stdout: String::from_utf8(output.stdout)
            .unwrap_or_else(|error| panic!("stdout utf8: {error}")),
        status_code: output.status.code(),
    }
}

fn run_lint_fix_for_agent_source(source: &str) -> FixLintOutput {
    run_lint_fix_for_raw_agent_source(&agent_org_source(source))
}

fn run_lint_fix_for_raw_agent_source(source: &str) -> FixLintOutput {
    let temp = tempdir_or_panic();
    let path = temp.path().join("agent.org");
    std::fs::write(&path, source).unwrap_or_else(|error| panic!("write agent org: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("lint")
        .arg("--fix")
        .arg("--format")
        .arg("compact")
        .arg("agent.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize lint --fix: {error}"));
    let updated =
        std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read agent org: {error}"));

    FixLintOutput {
        stdout: String::from_utf8(output.stdout)
            .unwrap_or_else(|error| panic!("stdout utf8: {error}")),
        status_code: output.status.code(),
        updated,
    }
}

fn assert_agent_org_date_has_seconds(source: &str) {
    let date_line = source
        .lines()
        .find(|line| line.starts_with("#+DATE: "))
        .unwrap_or_else(|| panic!("missing #+DATE line: {source}"));
    let value = date_line.trim_start_matches("#+DATE: ");
    let parts = value.split_whitespace().collect::<Vec<_>>();
    assert_eq!(parts.len(), 3, "date line: {date_line}");
    assert_eq!(parts[0].len(), 10, "date line: {date_line}");
    assert_eq!(parts[2].len(), 8, "date line: {date_line}");
}

fn agent_org_source(body: &str) -> String {
    format!(
        concat!(
            "#+TITLE: Agent Template\n",
            "#+AUTHOR: CyberXiuXian Artisan workshop\n",
            "#+FILETAGS: :agent:\n",
            "#+DATE: 2026-05-24 Sun 12:27:45\n",
            "\n",
            "{}"
        ),
        body
    )
}
