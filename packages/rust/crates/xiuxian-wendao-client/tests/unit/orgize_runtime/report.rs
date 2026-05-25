use std::process::Command;

use crate::orgize_runtime::support::{assert_cli_success, run_orgize, tempdir_or_panic};

#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn standalone_orgize_task_report_summarizes_archive_and_repeating_rows() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Performance cadence :agent:performance:\n",
            "SCHEDULED: <2026-05-18 Mon ++1d>\n",
            ":PROPERTIES:\n",
            ":ID: report-performance-cadence\n",
            ":END:\n",
            "* TODO Completed but not closed [3/3] [100%] :agent:\n",
            ":PROPERTIES:\n",
            ":ID: report-completed-but-not-closed\n",
            ":END:\n",
            "- [X] Scope\n",
            "- [X] Implementation\n",
            "- [X] Validation\n",
            "* DONE Completed slice [1/1] [100%] :agent:achievement:\n",
            "CLOSED: [2026-05-17 Sun]\n",
            ":PROPERTIES:\n",
            ":ID: report-completed-slice\n",
            ":NEXT_ACTION: Archive after review\n",
            ":END:\n",
            "- [X] Land completed slice.\n",
            "** Reflection\n",
            "- Summary: The completed slice landed.\n",
            "* DONE Completed cadence :agent:performance:\n",
            "SCHEDULED: <2026-05-17 Sun ++1d>\n",
            "CLOSED: [2026-05-17 Sun]\n",
            ":PROPERTIES:\n",
            ":ID: report-completed-cadence\n",
            ":END:\n",
            "* DONE Archived slice :agent:achievement:\n",
            "CLOSED: [2026-05-16 Sat]\n",
            ":PROPERTIES:\n",
            ":ID: report-archived-slice\n",
            ":ARCHIVE_TIME: 2026-05-16 Sat\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .env_remove("PRJ_CACHE_HOME")
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("task-report")
        .arg("agenda.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize task-report: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let stderr =
        String::from_utf8(output.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("active: 2"), "stdout: {stdout}");
    assert!(stdout.contains("done: 3"), "stdout: {stdout}");
    assert!(stdout.contains("achievements: 2"), "stdout: {stdout}");
    assert!(stdout.contains("archive-candidates: 1"), "stdout: {stdout}");
    assert!(stdout.contains("repeating: 2"), "stdout: {stdout}");
    assert!(stdout.contains("closure-needed: 1"), "stdout: {stdout}");
    assert!(stdout.contains("agent: 5"), "stdout: {stdout}");
    assert!(stdout.contains("achievement: 2"), "stdout: {stdout}");
    assert!(stdout.contains("performance: 2"), "stdout: {stdout}");
    assert!(stdout.contains("Closure Needed: 1"), "stdout: {stdout}");
    assert!(stdout.contains("Archive Candidates: 1"), "stdout: {stdout}");
    assert!(stdout.contains("Achievements: 2"), "stdout: {stdout}");
    assert!(stdout.contains("Repeating Tasks: 2"), "stdout: {stdout}");
    assert!(
        stdout.contains("repeat: scheduled ++1d (catchUp)"),
        "stdout: {stdout}"
    );
}

#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn standalone_orgize_task_report_named_views_limit_summary_scope() {
    let temp = tempdir_or_panic();
    write_task_report_named_views_fixture(temp.path());

    assert_task_report_view(
        temp.path(),
        "archive-candidate",
        &[
            "view: archive-candidate",
            "archive-candidates: 2",
            "Achievements: 1",
        ],
        &["Performance cadence"],
    );
    assert_task_report_view(
        temp.path(),
        "closure-needed",
        &[
            "view: closure-needed",
            "closure-needed: 1",
            "Completed but not closed",
        ],
        &[],
    );
    assert_task_report_view(
        temp.path(),
        "repeating",
        &["view: repeating", "repeating: 1", "Performance cadence"],
        &[],
    );
}

#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn standalone_orgize_task_report_summary_only_omits_detail_sections() {
    let temp = tempdir_or_panic();
    write_task_report_named_views_fixture(temp.path());

    let output = run_orgize(
        temp.path(),
        &["task-report", "--summary-only", "agenda.org"],
        "task-report summary-only",
    );
    assert_cli_success(&output);
    assert!(
        output.stdout.contains("summary-only: true"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output.stdout.contains("archive-candidates: 2"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output.stdout.contains("achievement: 1"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        !output.stdout.contains("Archive Candidates:"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        !output.stdout.contains("[TASK001]"),
        "stdout: {}",
        output.stdout
    );
}

#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn standalone_orgize_task_report_json_outputs_summary_contract() {
    let temp = tempdir_or_panic();
    write_task_report_named_views_fixture(temp.path());

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .env_remove("PRJ_CACHE_HOME")
        .arg("--root")
        .arg(temp.path())
        .arg("--output")
        .arg("json")
        .arg("orgize")
        .arg("task-report")
        .arg("--summary-only")
        .arg("agenda.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize task-report json: {error}"));
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
    assert_eq!(parsed["command"], "orgize task-report");
    assert_eq!(parsed["backend"], "duckdb");
    assert_eq!(parsed["summaryOnly"], true);
    assert_eq!(parsed["rows"], 4);
    assert_eq!(parsed["archiveCandidates"], 2);
    assert_eq!(parsed["tags"]["agent"], 4);
    assert_eq!(parsed["sections"]["archiveCandidates"], 2);
}

fn write_task_report_named_views_fixture(root: &std::path::Path) {
    std::fs::write(
        root.join("agenda.org"),
        concat!(
            "* TODO Performance cadence :agent:performance:\n",
            "SCHEDULED: <2026-05-18 Mon ++1d>\n",
            ":PROPERTIES:\n",
            ":ID: views-performance-cadence\n",
            ":END:\n",
            "* TODO Completed but not closed [3/3] [100%] :agent:\n",
            ":PROPERTIES:\n",
            ":ID: views-completed-but-not-closed\n",
            ":END:\n",
            "- [X] Scope\n",
            "- [X] Implementation\n",
            "- [X] Validation\n",
            "* DONE Completed slice [1/1] [100%] :agent:achievement:\n",
            "CLOSED: [2026-05-17 Sun]\n",
            ":PROPERTIES:\n",
            ":ID: views-completed-slice\n",
            ":END:\n",
            "- [X] Land completed slice.\n",
            "** Reflection\n",
            "- Summary: The completed slice landed.\n",
            "* DONE Archive candidate [1/1] [100%] :agent:\n",
            "CLOSED: [2026-05-17 Sun]\n",
            ":PROPERTIES:\n",
            ":ID: views-archive-candidate\n",
            ":END:\n",
            "- [X] Land archive candidate.\n",
            "** Reflection\n",
            "- Summary: The archive candidate landed.\n",
            "* DONE Archived slice :agent:achievement:ARCHIVE:\n",
            "CLOSED: [2026-05-16 Sat]\n",
            ":PROPERTIES:\n",
            ":ID: views-archived-slice\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));
}

fn assert_task_report_view(root: &std::path::Path, view: &str, expected: &[&str], absent: &[&str]) {
    let output = run_orgize(
        root,
        &["task-report", "--view", view, "agenda.org"],
        &format!("task-report {view} view"),
    );
    assert_cli_success(&output);
    for needle in expected {
        assert!(output.stdout.contains(needle), "stdout: {}", output.stdout);
    }
    for needle in absent {
        assert!(!output.stdout.contains(needle), "stdout: {}", output.stdout);
    }
}
