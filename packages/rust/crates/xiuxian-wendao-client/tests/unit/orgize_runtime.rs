use std::process::Command;

#[test]
fn standalone_orgize_agent_planning_renders_org_cards() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Agent task :agent:\n",
            "SCHEDULED: <2026-05-17 Sun>\n",
            "* DONE Closed task :agent:\n",
            "SCHEDULED: <2026-05-17 Sun>\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("agent-planning")
        .arg("--date")
        .arg("2026-05-17")
        .arg("agenda.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize agent-planning: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    assert_eq!(output.status.code(), Some(0));
    assert!(stdout.contains("task: Agent task"), "stdout: {stdout}");
    assert!(!stdout.contains("Closed task"), "stdout: {stdout}");
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
            "\n",
            "* TODO Agent slice [0/4] [0%] :agent:\n",
            ":PROPERTIES:\n",
            ":BLUEPRINT: <blueprint-path-or-none>\n",
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

#[test]
fn standalone_orgize_sparse_tree_finds_done_achievements_without_include_flags() {
    let temp = tempdir_or_panic();
    std::fs::write(
        temp.path().join("achievement.org"),
        concat!(
            "#+TITLE: Achievement Ledger\n",
            "#+FILETAGS: :agent:achievement:\n",
            "\n",
            "* DONE Completed slice [2/2] [100%] :agent:achievement:\n",
            "SCHEDULED: <2026-05-17 Sun>\n",
            "CLOSED: [2026-05-17 Sun]\n",
            "- [X] Implementation complete.\n",
            "- [X] Validation complete.\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write achievement org: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("sparse-tree")
        .arg("--match")
        .arg("+agent+achievement")
        .arg("achievement.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize sparse-tree: {error}"));

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
        stdout.contains("[SPARSE001] Match: Completed slice [2/2] [100%]"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("state: DONE"), "stdout: {stdout}");
}

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

#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn standalone_orgize_read_model_materializes_default_duckdb_snapshot() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Agent task :agent:\n",
            "SCHEDULED: <2026-05-17 Sun>\n",
            ":PROPERTIES:\n",
            ":NEXT_ACTION: Verify DuckDB snapshot\n",
            ":END:\n",
            "** Evidence\n",
            "This inherited agent-tag section should not become a task row.\n",
            "** TODO Task Checklist\n",
            "This inherited agent-tag TODO section should not become a task row.\n",
            "* DONE Closed task :agent:\n",
            "CLOSED: [2026-05-16 Sat]\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .env_remove("PRJ_CACHE_HOME")
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("read-model")
        .arg("agenda.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize read-model: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    assert_eq!(output.status.code(), Some(0), "stdout: {stdout}");
    assert!(stdout.contains("backend: duckdb"), "stdout: {stdout}");
    assert!(stdout.contains("rows: 2"), "stdout: {stdout}");
    let database_path = temp
        .path()
        .join(".cache")
        .join("agent")
        .join("readmodels")
        .join("org_agent_tasks.duckdb");
    assert!(database_path.is_file());
    assert_agent_task_row_count(&database_path, 2);
}

#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn standalone_orgize_task_list_lists_active_rows_from_duckdb_snapshot() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Agent task :agent:performance:\n",
            "SCHEDULED: <2026-05-17 Sun ++1w> DEADLINE: <2026-05-20 Wed .+2d>\n",
            ":PROPERTIES:\n",
            ":NEXT_ACTION: Verify DuckDB snapshot\n",
            ":RESUME_QUERY: wendao-client orgize task-list --text 'Agent task'\n",
            ":END:\n",
            "** Evidence\n",
            "This inherited agent-tag section should not become a task row.\n",
            "** TODO Task Checklist\n",
            "This inherited agent-tag TODO section should not become a task row.\n",
            "* DONE Closed task :agent:\n",
            "CLOSED: [2026-05-16 Sat]\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .env_remove("PRJ_CACHE_HOME")
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("task-list")
        .arg("--text")
        .arg("++1w")
        .arg("--tag")
        .arg("performance")
        .arg("agenda.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize task-list: {error}"));

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
        stdout.contains("orgize agent task-list"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("rows: 1"), "stdout: {stdout}");
    assert!(stdout.contains("[TASK001] Agent task"), "stdout: {stdout}");
    assert!(
        stdout.contains("tags: agent:performance"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("scheduled: <2026-05-17 Sun ++1w>"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("deadline: <2026-05-20 Wed .+2d>"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("repeat: scheduled ++1w (catchUp), deadline .+2d (restart)"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("next: Verify DuckDB snapshot"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("resume: wendao-client orgize task-list --text 'Agent task'"),
        "stdout: {stdout}"
    );
    assert!(!stdout.contains("Closed task"), "stdout: {stdout}");
    assert!(!stdout.contains("Evidence"), "stdout: {stdout}");
}

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
            "* DONE Completed slice :agent:achievement:\n",
            "CLOSED: [2026-05-17 Sun]\n",
            ":PROPERTIES:\n",
            ":NEXT_ACTION: Archive after review\n",
            ":END:\n",
            "* DONE Archived slice :agent:achievement:\n",
            "CLOSED: [2026-05-16 Sat]\n",
            ":PROPERTIES:\n",
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
    assert!(
        stdout.contains("orgize agent task-report"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("rows: 3"), "stdout: {stdout}");
    assert!(stdout.contains("active: 1"), "stdout: {stdout}");
    assert!(stdout.contains("done: 2"), "stdout: {stdout}");
    assert!(stdout.contains("achievements: 2"), "stdout: {stdout}");
    assert!(stdout.contains("archive-candidates: 2"), "stdout: {stdout}");
    assert!(stdout.contains("repeating: 1"), "stdout: {stdout}");
    assert!(stdout.contains("agent: 3"), "stdout: {stdout}");
    assert!(stdout.contains("achievement: 2"), "stdout: {stdout}");
    assert!(stdout.contains("performance: 1"), "stdout: {stdout}");
    assert!(stdout.contains("Archive Candidates: 2"), "stdout: {stdout}");
    assert!(stdout.contains("Achievements: 2"), "stdout: {stdout}");
    assert!(stdout.contains("Repeating Tasks: 1"), "stdout: {stdout}");
    assert!(
        stdout.contains("repeat: scheduled ++1d (catchUp)"),
        "stdout: {stdout}"
    );
}

#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn standalone_orgize_task_archive_plans_and_applies_completed_rows() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    let archive_path = temp
        .path()
        .join(".cache")
        .join("agent")
        .join("org")
        .join("archives")
        .join("completed.org");
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
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let plan = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .env_remove("PRJ_CACHE_HOME")
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("task-archive")
        .arg("agenda.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize task-archive plan: {error}"));

    let plan_stdout =
        String::from_utf8(plan.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let plan_stderr =
        String::from_utf8(plan.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    assert_eq!(
        plan.status.code(),
        Some(0),
        "stdout: {plan_stdout}\nstderr: {plan_stderr}"
    );
    assert!(plan_stdout.contains("mode: plan"), "stdout: {plan_stdout}");
    assert!(
        plan_stdout.contains("candidates: 1"),
        "stdout: {plan_stdout}"
    );
    assert!(
        plan_stdout.contains("[ARCHIVE001] Completed slice"),
        "stdout: {plan_stdout}"
    );
    assert!(!archive_path.exists());
    assert!(
        std::fs::read_to_string(&agenda)
            .unwrap_or_else(|error| panic!("read agenda after plan: {error}"))
            .contains("Completed slice")
    );

    let apply = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .env_remove("PRJ_CACHE_HOME")
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("task-archive")
        .arg("--apply")
        .arg("agenda.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize task-archive apply: {error}"));

    let apply_stdout =
        String::from_utf8(apply.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let apply_stderr =
        String::from_utf8(apply.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    assert_eq!(
        apply.status.code(),
        Some(0),
        "stdout: {apply_stdout}\nstderr: {apply_stderr}"
    );
    assert!(
        apply_stdout.contains("mode: apply"),
        "stdout: {apply_stdout}"
    );
    assert!(
        apply_stdout.contains("applied: 1"),
        "stdout: {apply_stdout}"
    );
    let agenda_after =
        std::fs::read_to_string(&agenda).unwrap_or_else(|error| panic!("read agenda: {error}"));
    assert!(!agenda_after.contains("Completed slice"));
    assert!(agenda_after.contains("Active task"));
    let archive_after = std::fs::read_to_string(&archive_path)
        .unwrap_or_else(|error| panic!("read archive: {error}"));
    assert!(archive_after.contains("#+FILETAGS: :ARCHIVE:"));
    assert!(
        archive_after.contains("* DONE Completed slice :agent:achievement:ARCHIVE:"),
        "archive: {archive_after}"
    );
    assert!(archive_after.contains("Evidence remains with the archived subtree."));

    let report = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .env_remove("PRJ_CACHE_HOME")
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("task-report")
        .arg("--include-archived")
        .arg(".")
        .output()
        .unwrap_or_else(|error| panic!("run orgize task-report after archive: {error}"));

    let report_stdout =
        String::from_utf8(report.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let report_stderr =
        String::from_utf8(report.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    assert_eq!(
        report.status.code(),
        Some(0),
        "stdout: {report_stdout}\nstderr: {report_stderr}"
    );
    assert!(
        report_stdout.contains("archived: 1"),
        "stdout: {report_stdout}"
    );
    assert!(
        report_stdout.contains("archive-candidates: 0"),
        "stdout: {report_stdout}"
    );
}

#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn standalone_orgize_read_model_uses_wendao_toml_path_overrides() {
    let temp = tempdir_or_panic();
    std::fs::write(
        temp.path().join("wendao.toml"),
        concat!(
            "[agent.org_read_model]\n",
            "database_path = \".cache/custom-agent/tasks.duckdb\"\n",
            "temp_directory = \".cache/custom-agent/tmp\"\n",
            "threads = 1\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write wendao.toml: {error}"));
    std::fs::write(
        temp.path().join("agenda.org"),
        "* TODO Agent task :agent:\n",
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .env_remove("PRJ_CACHE_HOME")
        .arg("--root")
        .arg(temp.path())
        .arg("orgize")
        .arg("read-model")
        .arg("agenda.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize read-model: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    assert_eq!(output.status.code(), Some(0), "stdout: {stdout}");
    assert!(stdout.contains("rows: 1"), "stdout: {stdout}");
    let database_path = temp
        .path()
        .join(".cache")
        .join("custom-agent")
        .join("tasks.duckdb");
    assert!(database_path.is_file());
    assert!(
        stdout.contains(database_path.to_string_lossy().as_ref()),
        "stdout: {stdout}"
    );
    assert_agent_task_row_count(&database_path, 1);
}

#[cfg(feature = "orgize-agent-read-model")]
fn assert_agent_task_row_count(database_path: &std::path::Path, expected: i64) {
    let connection = xiuxian_db_store::duckdb_crate::Connection::open(database_path)
        .unwrap_or_else(|error| panic!("open read-model duckdb: {error}"));
    let count = connection
        .query_row("SELECT COUNT(*) FROM agent_org_tasks", [], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap_or_else(|error| panic!("query read-model row count: {error}"));
    assert_eq!(count, expected);
}

fn tempdir_or_panic() -> tempfile::TempDir {
    tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"))
}
