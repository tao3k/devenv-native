use xiuxian_wendao_parsers::{OrgizeAgentTaskReadModelRequest, collect_agent_task_rows};

use super::support::tempdir_or_panic;

#[test]
fn collect_agent_task_rows_uses_orgize_sparse_tree_semantics() {
    let temp = tempdir_or_panic();
    let path = temp.path().join("agent.org");
    std::fs::write(
        &path,
        concat!(
            "* TODO Agent read model :agent:checkpoint:\n",
            "SCHEDULED: <2026-05-17 Sun ++1w> DEADLINE: <2026-05-20 Wed .+2d>\n",
            ":PROPERTIES:\n",
            ":ID: agent-read-model\n",
            ":NEXT_ACTION: Materialize DuckDB snapshot\n",
            ":END:\n",
            "The active task should be indexed.\n",
            "** Evidence\n",
            "This inherited agent-tag section should not become a task row.\n",
            "** TODO Task Checklist\n",
            "This inherited agent-tag TODO section should not become a task row.\n",
            "* DONE Finished task :agent:\n",
            "CLOSED: [2026-05-16 Sat]\n",
            ":PROPERTIES:\n",
            ":ID: finished-task\n",
            ":END:\n",
            "* TODO Non-agent task\n",
            "This task should not match the default agent query.\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agent org: {error}"));

    let report = collect_agent_task_rows(&OrgizeAgentTaskReadModelRequest {
        paths: vec![path.clone()],
        match_expression: None,
        include_comments: false,
    })
    .unwrap_or_else(|error| panic!("collect agent rows: {error}"));

    assert_eq!(report.rows.len(), 2);
    let active = report
        .rows
        .iter()
        .find(|row| row.title == "Agent read model")
        .unwrap_or_else(|| panic!("active row missing: {report:?}"));
    assert_eq!(active.source_path, path.display().to_string());
    assert_eq!(active.orgid, "agent-read-model");
    assert_eq!(active.source_range_start, 0);
    assert!(active.source_range_end > active.source_range_start);
    assert_eq!(active.todo_state.as_deref(), Some("TODO"));
    assert!(!active.is_done);
    assert_eq!(active.scheduled.as_deref(), Some("<2026-05-17 Sun ++1w>"));
    assert_eq!(
        active
            .scheduled_repeater
            .as_ref()
            .map(|repeater| repeater.cookie.as_str()),
        Some("++1w")
    );
    assert_eq!(active.deadline.as_deref(), Some("<2026-05-20 Wed .+2d>"));
    assert_eq!(
        active
            .deadline_repeater
            .as_ref()
            .map(|repeater| repeater.cookie.as_str()),
        Some(".+2d")
    );
    assert!(
        active.effective_tags.iter().any(|tag| tag == "checkpoint"),
        "active row: {active:?}"
    );
    assert!(
        active
            .properties
            .iter()
            .any(|property| property.key == "NEXT_ACTION"
                && property.value == "Materialize DuckDB snapshot"),
        "active row: {active:?}"
    );

    let done = report
        .rows
        .iter()
        .find(|row| row.title == "Finished task")
        .unwrap_or_else(|| panic!("done row missing: {report:?}"));
    assert_eq!(done.orgid, "finished-task");
    assert!(done.is_done);
    assert_eq!(done.closed.as_deref(), Some("[2026-05-16 Sat]"));
}

#[test]
fn collect_agent_task_rows_accepts_agent_doing_keyword_without_file_declaration() {
    let temp = tempdir_or_panic();
    let path = temp.path().join("agent.org");
    std::fs::write(
        &path,
        concat!(
            "* DOING Agent implementation <2026-05-23 Sat> :agent:\n",
            ":PROPERTIES:\n",
            ":ID: doing-task\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agent org: {error}"));

    let report = collect_agent_task_rows(&OrgizeAgentTaskReadModelRequest {
        paths: vec![path],
        match_expression: None,
        include_comments: false,
    })
    .unwrap_or_else(|error| panic!("collect agent rows: {error}"));

    assert_eq!(report.rows.len(), 1);
    assert_eq!(report.rows[0].orgid, "doing-task");
    assert_eq!(report.rows[0].todo_state.as_deref(), Some("DOING"));
    assert_eq!(
        report.rows[0].title,
        "Agent implementation <2026-05-23 Sat>"
    );
    assert!(!report.rows[0].is_done);
}

#[test]
fn collect_agent_task_rows_requires_org_section_id() {
    let temp = tempdir_or_panic();
    let path = temp.path().join("agent.org");
    std::fs::write(&path, "* TODO Agent read model :agent:\n")
        .unwrap_or_else(|error| panic!("write agent org: {error}"));

    let error = match collect_agent_task_rows(&OrgizeAgentTaskReadModelRequest {
        paths: vec![path],
        match_expression: None,
        include_comments: false,
    }) {
        Ok(report) => panic!("missing ID should fail, got {report:?}"),
        Err(error) => error,
    };

    assert!(
        error.to_string().contains("missing required :ID: property"),
        "error: {error}"
    );
}

#[test]
fn collect_agent_task_rows_reads_id_from_legacy_multiline_planning_drawer() {
    let temp = tempdir_or_panic();
    let path = temp.path().join("agent.org");
    std::fs::write(
        &path,
        concat!(
            "* Parent\n",
            "** DONE Legacy task :agent:\n",
            "CLOSED: [2026-05-22 Fri]\n",
            "SCHEDULED: <2026-05-22 Fri>\n",
            ":PROPERTIES:\n",
            ":ID: legacy-task-id\n",
            ":END:\n",
            "Body.\n",
            "*** DONE Child :agent:\n",
            ":PROPERTIES:\n",
            ":ID: child-task-id\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agent org: {error}"));

    let report = collect_agent_task_rows(&OrgizeAgentTaskReadModelRequest {
        paths: vec![path],
        match_expression: None,
        include_comments: false,
    })
    .unwrap_or_else(|error| panic!("collect agent rows: {error}"));

    let legacy = report
        .rows
        .iter()
        .find(|row| row.title == "Legacy task")
        .unwrap_or_else(|| panic!("legacy row missing: {report:?}"));
    assert_eq!(legacy.orgid, "legacy-task-id");
}
