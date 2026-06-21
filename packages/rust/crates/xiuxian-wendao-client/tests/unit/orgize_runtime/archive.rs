use std::path::{Path, PathBuf};

use crate::orgize_runtime::support::{assert_cli_success, run_orgize, tempdir_or_panic};

fn project_cache_root_rel(root: &Path) -> String {
    let cache_root = xiuxian_db_store::state::project_cache_root_from_config(
        xiuxian_db_store::state::ProjectCacheRootConfig {
            project_root: Some(root.to_path_buf()),
            cache_home: Some(root.join(".cache")),
            project_namespace: None,
        },
    );
    cache_root
        .strip_prefix(root)
        .unwrap_or(cache_root.as_path())
        .display()
        .to_string()
}

fn archive_rel_path(root: &Path, file_name: &str) -> String {
    format!(
        "{}/agent/org/archives/{file_name}",
        project_cache_root_rel(root)
    )
}

fn expected_archive_path(root: &Path, file_name: &str) -> PathBuf {
    xiuxian_db_store::state::project_cache_root_from_config(
        xiuxian_db_store::state::ProjectCacheRootConfig {
            project_root: Some(root.to_path_buf()),
            cache_home: Some(root.join(".cache")),
            project_namespace: None,
        },
    )
    .join("agent")
    .join("org")
    .join("archives")
    .join(file_name)
}

#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn standalone_orgize_task_archive_plans_and_applies_completed_rows() {
    let temp = tempdir_or_panic();
    let fixture = write_archive_fixture(temp.path());

    assert_archive_plan(temp.path(), &fixture);
    assert_archive_target_filter(temp.path(), &fixture);
    assert_archive_closed_before_filter(temp.path(), &fixture);
    assert_archive_expect_selected_gate(temp.path(), &fixture);
    assert_archive_plan_json(temp.path(), &fixture);
    assert_archive_apply(temp.path(), &fixture);
    assert_archive_report(temp.path());
}

#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn standalone_orgize_task_archive_apply_json_outputs_write_receipt() {
    let temp = tempdir_or_panic();
    let fixture = write_archive_fixture(temp.path());

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .env_remove("PRJ_CACHE_HOME")
        .arg("--root")
        .arg(temp.path())
        .arg("--output")
        .arg("json")
        .arg("orgize")
        .arg("task-archive")
        .arg("--apply")
        .arg("--target")
        .arg("completed.org")
        .arg("--closed-before")
        .arg("2026-05-18")
        .arg("--expect-selected")
        .arg("1")
        .arg("agenda.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize task-archive apply json: {error}"));
    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let stderr =
        String::from_utf8(output.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).unwrap_or_else(|error| panic!("parse json: {error}"));
    assert_eq!(parsed["command"], "orgize task-archive");
    assert_eq!(parsed["mode"], "apply");
    assert_eq!(parsed["applied"], 1);
    assert_eq!(parsed["selected"], 1);
    assert_eq!(parsed["sourcesUpdated"][0], "agenda.org", "json: {parsed}");
    assert_eq!(
        parsed["targetsUpdated"][0],
        archive_rel_path(temp.path(), "completed.org"),
        "json: {parsed}"
    );
    assert_eq!(parsed["postApplyRefresh"], "refreshed");
    assert_eq!(parsed["postApplyRows"], 4);
    let agenda_after = std::fs::read_to_string(&fixture.agenda)
        .unwrap_or_else(|error| panic!("read agenda after json apply: {error}"));
    assert!(!agenda_after.contains("Completed slice"));
    assert!(fixture.archive_path.is_file());
}

#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn standalone_orgize_task_archive_defaults_to_source_file_target() {
    let temp = tempdir_or_panic();
    let source = temp.path().join("completed_task.org");
    std::fs::write(
        &source,
        concat!(
            "* DONE Completed task [1/1] [100%] :agent:\n",
            "CLOSED: [2026-05-17 Sun]\n",
            ":PROPERTIES:\n",
            ":ID: archive-default-target-task\n",
            ":END:\n",
            "- [X] Land completed task.\n",
            "** Reflection Questions\n",
            "- Summary: The completed task landed.\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write source: {error}"));

    let apply = run_orgize(
        temp.path(),
        &[
            "task-archive",
            "--apply",
            "--expect-selected",
            "1",
            "completed_task.org",
        ],
        "task-archive default target",
    );

    assert_cli_success(&apply);
    assert!(
        apply.stdout.contains(
            format!(
                "- target: {}",
                archive_rel_path(temp.path(), "completed_task.org")
            )
            .as_str()
        ),
        "stdout: {}",
        apply.stdout
    );
    let archive_path = expected_archive_path(temp.path(), "completed_task.org");
    assert!(archive_path.is_file());
}

#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn standalone_orgize_task_archive_ignores_deprecated_year_bucket_target() {
    let temp = tempdir_or_panic();
    let source = temp.path().join("year_bucket_task.org");
    std::fs::write(
        &source,
        concat!(
            "* DONE Year bucket task [1/1] [100%] :agent:\n",
            "CLOSED: [2026-05-17 Sun]\n",
            ":PROPERTIES:\n",
            ":ID: archive-year-bucket-task\n",
            ":ARCHIVE_TARGET: $PRJ_CACHE_HOME/agent/org/archives/2026.org\n",
            ":END:\n",
            "- [X] Land completed task.\n",
            "** Reflection Questions\n",
            "- Summary: The completed task landed.\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write source: {error}"));

    let apply = run_orgize(
        temp.path(),
        &[
            "task-archive",
            "--apply",
            "--expect-selected",
            "1",
            "year_bucket_task.org",
        ],
        "task-archive deprecated year target",
    );

    assert_cli_success(&apply);
    assert!(
        apply.stdout.contains(
            format!(
                "- target: {}",
                archive_rel_path(temp.path(), "year_bucket_task.org")
            )
            .as_str()
        ),
        "stdout: {}",
        apply.stdout
    );
    assert!(
        !expected_archive_path(temp.path(), "2026.org").exists(),
        "deprecated yearly bucket should not be created"
    );
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
    let archive_path = expected_archive_path(root, "completed.org");
    let secondary_archive_path = expected_archive_path(root, "secondary.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Active task :agent:\n",
            "SCHEDULED: <2026-05-18 Mon>\n",
            ":PROPERTIES:\n",
            ":ID: archive-active-task\n",
            ":END:\n",
            "* DONE Completed slice [1/1] [100%] :agent:achievement:\n",
            "CLOSED: [2026-05-17 Sun]\n",
            ":PROPERTIES:\n",
            ":ID: archive-completed-slice\n",
            ":ARCHIVE_TARGET: $PRJ_CACHE_HOME/agent/org/archives/completed.org\n",
            ":END:\n",
            "- [X] Land completed slice.\n",
            "Evidence remains with the archived subtree.\n",
            "** Reflection Questions\n",
            "- Summary: The completed slice landed.\n",
            "* DONE Recent completed slice [1/1] [100%] :agent:\n",
            "CLOSED: [2026-05-20 Wed]\n",
            ":PROPERTIES:\n",
            ":ID: archive-recent-completed-slice\n",
            ":ARCHIVE_TARGET: $PRJ_CACHE_HOME/agent/org/archives/completed.org\n",
            ":END:\n",
            "- [X] Land recent slice.\n",
            "** Closure Questions\n",
            "- Summary: The recent completed slice landed.\n",
            "* DONE Secondary completed slice [1/1] [100%] :agent:\n",
            "CLOSED: [2026-05-17 Sun]\n",
            ":PROPERTIES:\n",
            ":ID: archive-secondary-completed-slice\n",
            ":ARCHIVE_TARGET: $PRJ_CACHE_HOME/agent/org/archives/secondary.org\n",
            ":END:\n",
            "- [X] Land secondary slice.\n",
            "** Reflection\n",
            "- Summary: The secondary completed slice landed.\n",
            "* DONE Repeating cadence :agent:\n",
            "SCHEDULED: <2026-05-18 Mon ++1d>\n",
            "CLOSED: [2026-05-18 Mon]\n",
            ":PROPERTIES:\n",
            ":ID: archive-repeating-cadence\n",
            ":END:\n",
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
            .contains(format!("{}: 2", archive_rel_path(root, "completed.org")).as_str()),
        "stdout: {}",
        plan.stdout
    );
    assert!(
        plan.stdout
            .contains(format!("{}: 1", archive_rel_path(root, "secondary.org")).as_str()),
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
fn assert_archive_expect_selected_gate(root: &Path, fixture: &ArchiveFixture) {
    let plan = run_orgize(
        root,
        &[
            "task-archive",
            "--target",
            "completed.org",
            "--closed-before",
            "2026-05-18",
            "--expect-selected",
            "1",
            "agenda.org",
        ],
        "task-archive expect-selected plan",
    );
    assert_cli_success(&plan);
    assert!(
        plan.stdout.contains("expect-selected: 1"),
        "stdout: {}",
        plan.stdout
    );
    let mismatch = run_orgize(
        root,
        &[
            "task-archive",
            "--target",
            "completed.org",
            "--closed-before",
            "2026-05-18",
            "--expect-selected",
            "2",
            "agenda.org",
        ],
        "task-archive expect-selected mismatch",
    );
    assert_eq!(
        mismatch.status_code,
        Some(1),
        "stdout: {}\nstderr: {}",
        mismatch.stdout,
        mismatch.stderr
    );
    assert!(
        mismatch
            .stderr
            .contains("archive selected row count mismatch: expected 2, selected 1"),
        "stderr: {}",
        mismatch.stderr
    );
    assert!(
        std::fs::read_to_string(&fixture.agenda)
            .unwrap_or_else(|error| panic!("read agenda after mismatch: {error}"))
            .contains("Completed slice"),
        "mismatched expect-selected must not edit source"
    );
    assert!(
        !fixture.archive_path.exists(),
        "mismatched expect-selected must not write archive"
    );
}

#[cfg(feature = "orgize-agent-read-model")]
fn assert_archive_plan_json(root: &Path, _fixture: &ArchiveFixture) {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .env_remove("PRJ_CACHE_HOME")
        .arg("--root")
        .arg(root)
        .arg("--output")
        .arg("json")
        .arg("orgize")
        .arg("task-archive")
        .arg("--target")
        .arg("completed.org")
        .arg("--closed-before")
        .arg("2026-05-18")
        .arg("--expect-selected")
        .arg("1")
        .arg("agenda.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize task-archive json: {error}"));
    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let stderr =
        String::from_utf8(output.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).unwrap_or_else(|error| panic!("parse json: {error}"));
    assert_eq!(parsed["command"], "orgize task-archive");
    assert_eq!(parsed["mode"], "plan");
    assert_eq!(parsed["targetFilter"], "completed.org");
    assert_eq!(parsed["closedBefore"], "2026-05-18");
    assert_eq!(parsed["expectSelected"], 1);
    assert_eq!(parsed["candidates"], 1);
    assert_eq!(parsed["selected"], 1);
    let archive_target = archive_rel_path(root, "completed.org");
    assert_eq!(parsed["archiveTargets"][archive_target.as_str()], 1);
    assert_eq!(parsed["items"].as_array().map_or(0, Vec::len), 1);
    assert_eq!(parsed["items"][0]["title"], "Completed slice [1/1] [100%]");
    assert_eq!(parsed["items"][0]["target"], archive_target);
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
            "--expect-selected",
            "1",
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
        apply.stdout.contains("sources-updated: 1"),
        "stdout: {}",
        apply.stdout
    );
    assert!(
        apply.stdout.contains("- source: agenda.org"),
        "stdout: {}",
        apply.stdout
    );
    assert!(
        apply.stdout.contains("targets-updated: 1"),
        "stdout: {}",
        apply.stdout
    );
    let archive_target = archive_rel_path(root, "completed.org");
    assert!(
        apply
            .stdout
            .contains(&format!("- target: {archive_target}")),
        "stdout: {}",
        apply.stdout
    );
    assert!(
        apply.stdout.contains("post-apply-refresh: refreshed"),
        "stdout: {}",
        apply.stdout
    );
    assert!(
        apply.stdout.contains("post-apply-rows: 4"),
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
    assert!(
        !archive_after.starts_with("#+TITLE:"),
        "archive should contain only archived subtrees: {archive_after}"
    );
    assert!(
        !archive_after.contains("#+FILETAGS: :ARCHIVE:"),
        "archive should not synthesize document metadata: {archive_after}"
    );
    assert!(
        archive_after.starts_with("* DONE Completed slice"),
        "archive should start directly with the archived subtree: {archive_after}"
    );
    assert!(
        archive_after.contains("* DONE Completed slice [1/1] [100%] :agent:achievement:ARCHIVE:"),
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
