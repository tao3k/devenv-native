pub(super) use crate::orgize_runtime::support::tempdir_or_panic;
use std::process::Command;

pub(super) fn answered_closure_questions() -> &'static str {
    concat!(
        "** Closure Questions\n",
        "| Question | Value |\n",
        "|---+---|\n",
        "| Did the task meet its expected outcome? | Yes, the slice landed and targeted validation passed. |\n",
    )
}
pub(super) fn answered_reflection_questions() -> &'static str {
    concat!(
        "** Reflection Questions\n",
        "| Question | Value |\n",
        "|---+---|\n",
        "| What finality signal should future agents recall from this slice? | The slice landed with validation evidence. |\n",
    )
}
pub(super) fn unanswered_closure_questions() -> &'static str {
    concat!(
        "** Closure Questions\n",
        "| Question | Value |\n",
        "|---+---|\n",
        "| Did the task meet its expected outcome? | |\n",
    )
}
pub(super) struct LintOutput {
    pub stdout: String,
    pub status_code: Option<i32>,
}
pub(super) struct FixLintOutput {
    pub stdout: String,
    pub status_code: Option<i32>,
    pub updated: String,
}
pub(super) fn run_lint_for_agent_heading(heading: &str) -> LintOutput {
    run_lint_for_agent_heading_and_body(heading, "- [X] one\n- [ ] two\n")
}
pub(super) fn run_lint_for_agent_heading_and_body(heading: &str, body: &str) -> LintOutput {
    run_lint_for_agent_source(&format!(
        "{heading}:PROPERTIES:\n:ID: test-agent-task\n:END:\n{body}"
    ))
}
pub(super) fn run_lint_for_agent_source(source: &str) -> LintOutput {
    run_lint_for_raw_agent_source(&agent_org_source(source))
}

pub(super) fn run_lint_for_raw_agent_source(source: &str) -> LintOutput {
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

pub(super) fn run_lint_fix_for_agent_source(source: &str) -> FixLintOutput {
    run_lint_fix_for_raw_agent_source(&agent_org_source(source))
}

pub(super) fn run_lint_fix_for_raw_agent_source(source: &str) -> FixLintOutput {
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

pub(super) fn assert_agent_org_date_has_seconds(source: &str) {
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
