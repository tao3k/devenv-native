use std::path::{Path, PathBuf};

use crate::orgize_runtime::support::{assert_cli_success, run_orgize, tempdir_or_panic};

#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn standalone_orgize_task_archive_plans_and_applies_completed_rows() {
    let temp = tempdir_or_panic();
    let fixture = write_archive_fixture(temp.path());

    assert_archive_plan(temp.path(), &fixture);
    assert_archive_target_filter(temp.path(), &fixture);
    assert_archive_closed_before_filter(temp.path(), &fixture);
    assert_archive_apply(temp.path(), &fixture);
    assert_archive_report(temp.path());
}

#[cfg(feature = "orgize-agent-read-model")]
struct ArchiveFixture {
    agenda: PathBuf,
    archive_path: PathBuf,
    secondary_archive_path: PathBuf,
}

#[cfg(feature = "orgize-agent-read-model")]
fn write_archive_fixture(root: &Path) -> ArchiveFixture {
    let agenda = root.join("agenda.org");
    let archive_path = root
        .join(".cache")
        .join("agent")
        .join("org")
        .join("archives")
        .join("completed.org");
    let secondary_archive_path = root
        .join(".cache")
        .join("agent")
        .join("org")
        .join("archives")
        .join("secondary.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Active task :agent:\n",
            "SCHEDULED: <2026-05-18 Mon>\n",
            "* DONE Completed slice :agent:achievement:\n",
            "CLOSED: [2026-05-17 Sun]\n",
            ":PROPERTIES:\n",
            ":ARCHIVE_TARGET: $PRJ_CACHE_HOME/agent/org/archives/completed.org\n",
            ":END:\n",
            "Evidence remains with the archived subtree.\n",
            "* DONE Recent completed slice :agent:\n",
            "CLOSED: [2026-05-20 Wed]\n",
            ":PROPERTIES:\n",
            ":ARCHIVE_TARGET: $PRJ_CACHE_HOME/agent/org/archives/completed.org\n",
            ":END:\n",
            "* DONE Secondary completed slice :agent:\n",
            "CLOSED: [2026-05-17 Sun]\n",
            ":PROPERTIES:\n",
            ":ARCHIVE_TARGET: $PRJ_CACHE_HOME/agent/org/archives/secondary.org\n",
            ":END:\n",
            "* DONE Repeating cadence :agent:\n",
            "SCHEDULED: <2026-05-18 Mon ++1d>\n",
            "CLOSED: [2026-05-18 Mon]\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));
    ArchiveFixture {
        agenda,
        archive_path,
        secondary_archive_path,
    }
}

#[cfg(feature = "orgize-agent-read-model")]
fn assert_archive_plan(root: &Path, fixture: &ArchiveFixture) {
    let plan = run_orgize(root, &["task-archive", "agenda.org"], "task-archive plan");
    assert_cli_success(&plan);
    assert!(
        plan.stdout.contains("mode: plan"),
        "stdout: {}",
        plan.stdout
    );
    assert!(
        plan.stdout.contains("candidates: 3"),
        "stdout: {}",
        plan.stdout
    );
    assert!(
        plan.stdout.contains("[ARCHIVE001] Completed slice"),
        "stdout: {}",
        plan.stdout
    );
    assert!(
        plan.stdout.contains("Archive Targets"),
        "stdout: {}",
        plan.stdout
    );
    assert!(
        plan.stdout
            .contains(".cache/agent/org/archives/completed.org: 2"),
        "stdout: {}",
        plan.stdout
    );
    assert!(
        plan.stdout
            .contains(".cache/agent/org/archives/secondary.org: 1"),
        "stdout: {}",
        plan.stdout
    );
    assert!(!fixture.archive_path.exists());
    assert!(!fixture.secondary_archive_path.exists());
    assert!(
        std::fs::read_to_string(&fixture.agenda)
            .unwrap_or_else(|error| panic!("read agenda after plan: {error}"))
            .contains("Completed slice")
    );
}

#[cfg(feature = "orgize-agent-read-model")]
fn assert_archive_target_filter(root: &Path, _fixture: &ArchiveFixture) {
    let plan = run_orgize(
        root,
        &["task-archive", "--target", "completed.org", "agenda.org"],
        "task-archive target filter",
    );
    assert_cli_success(&plan);
    assert!(
        plan.stdout.contains("target-filter: completed.org"),
        "stdout: {}",
        plan.stdout
    );
    assert!(
        plan.stdout.contains("candidates: 2"),
        "stdout: {}",
        plan.stdout
    );
    assert!(
        plan.stdout.contains("[ARCHIVE001] Completed slice"),
        "stdout: {}",
        plan.stdout
    );
    assert!(
        plan.stdout.contains("Recent completed slice"),
        "stdout: {}",
        plan.stdout
    );
    assert!(
        !plan.stdout.contains("Secondary completed slice"),
        "stdout: {}",
        plan.stdout
    );
}

#[cfg(feature = "orgize-agent-read-model")]
fn assert_archive_closed_before_filter(root: &Path, _fixture: &ArchiveFixture) {
    let plan = run_orgize(
        root,
        &[
            "task-archive",
            "--target",
            "completed.org",
            "--closed-before",
            "2026-05-18",
            "agenda.org",
        ],
        "task-archive closed-before filter",
    );
    assert_cli_success(&plan);
    assert!(
        plan.stdout.contains("target-filter: completed.org"),
        "stdout: {}",
        plan.stdout
    );
    assert!(
        plan.stdout.contains("closed-before: 2026-05-18"),
        "stdout: {}",
        plan.stdout
    );
    assert!(
        plan.stdout.contains("candidates: 1"),
        "stdout: {}",
        plan.stdout
    );
    assert!(
        plan.stdout.contains("[ARCHIVE001] Completed slice"),
        "stdout: {}",
        plan.stdout
    );
    assert!(
        !plan.stdout.contains("Recent completed slice"),
        "stdout: {}",
        plan.stdout
    );
}

#[cfg(feature = "orgize-agent-read-model")]
fn assert_archive_apply(root: &Path, fixture: &ArchiveFixture) {
    let apply = run_orgize(
        root,
        &[
            "task-archive",
            "--apply",
            "--target",
            "completed.org",
            "--closed-before",
            "2026-05-18",
            "agenda.org",
        ],
        "task-archive apply",
    );
    assert_cli_success(&apply);
    assert!(
        apply.stdout.contains("mode: apply"),
        "stdout: {}",
        apply.stdout
    );
    assert!(
        apply.stdout.contains("applied: 1"),
        "stdout: {}",
        apply.stdout
    );
    assert!(
        apply.stdout.contains("Archive Targets"),
        "stdout: {}",
        apply.stdout
    );
    let agenda_after = std::fs::read_to_string(&fixture.agenda)
        .unwrap_or_else(|error| panic!("read agenda: {error}"));
    assert!(!agenda_after.contains("Completed slice"));
    assert!(agenda_after.contains("Recent completed slice"));
    assert!(agenda_after.contains("Secondary completed slice"));
    assert!(agenda_after.contains("Active task"));
    assert!(agenda_after.contains("Repeating cadence"));
    let archive_after = std::fs::read_to_string(&fixture.archive_path)
        .unwrap_or_else(|error| panic!("read archive: {error}"));
    assert!(archive_after.contains("#+FILETAGS: :ARCHIVE:"));
    assert!(
        archive_after.contains("* DONE Completed slice :agent:achievement:ARCHIVE:"),
        "archive: {archive_after}"
    );
    assert!(
        archive_after.contains("Evidence remains with the archived subtree."),
        "archive: {archive_after}"
    );
    assert!(!fixture.secondary_archive_path.exists());
}

#[cfg(feature = "orgize-agent-read-model")]
fn assert_archive_report(root: &Path) {
    let report = run_orgize(
        root,
        &["task-report", "--include-archived", "."],
        "task-report",
    );
    assert_cli_success(&report);
    assert!(
        report.stdout.contains("archived: 1"),
        "stdout: {}",
        report.stdout
    );
    assert!(
        report.stdout.contains("archive-candidates: 2"),
        "stdout: {}",
        report.stdout
    );
}
