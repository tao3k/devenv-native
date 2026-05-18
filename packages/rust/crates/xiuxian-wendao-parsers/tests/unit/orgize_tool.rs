use xiuxian_wendao_parsers::{
    OrgizeAgentPlanningRequest, OrgizeAgentTaskReadModelRequest, OrgizeSddStatusRequest,
    OrgizeSparseTreeRenderOptions, OrgizeSparseTreeRequest, OrgizeSparseTreeVisibility,
    collect_agent_task_rows, render_agent_planning, render_sdd_status, render_sparse_tree,
};

#[test]
fn render_agent_planning_uses_org_agenda_semantics() {
    let temp = tempdir_or_panic();
    let path = temp.path().join("agenda.org");
    std::fs::write(
        &path,
        concat!(
            "* TODO Agent task :agent:\n",
            "SCHEDULED: <2026-05-17 Sun>\n",
            "* DONE Closed task :agent:\n",
            "SCHEDULED: <2026-05-17 Sun>\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let rendered = render_agent_planning(&OrgizeAgentPlanningRequest {
        paths: vec![path],
        start_date: "2026-05-17".to_string(),
        end_date: None,
        include_done: false,
        include_archived: false,
        include_comments: false,
        match_expression: Some("+agent".to_string()),
    })
    .unwrap_or_else(|error| panic!("render planning: {error}"));

    assert!(
        rendered.contains("task: Agent task"),
        "rendered: {rendered}"
    );
    assert!(
        rendered.contains("contract: Derived from official Org agenda syntax"),
        "rendered: {rendered}"
    );
    assert!(!rendered.contains("Closed task"), "rendered: {rendered}");
}

#[test]
fn render_sparse_tree_can_filter_done_tasks() {
    let temp = tempdir_or_panic();
    let path = temp.path().join("memory.org");
    std::fs::write(
        &path,
        concat!(
            "* TODO Active memory :agent:\n",
            "The active agent memory mentions sparse tree cards.\n",
            "* DONE Retired memory :agent:\n",
            "The retired agent memory should be filtered.\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write memory: {error}"));

    let rendered = render_sparse_tree(&OrgizeSparseTreeRequest {
        paths: vec![path],
        text: Some("agent memory".to_string()),
        match_expression: Some("+agent".to_string()),
        visibility: OrgizeSparseTreeVisibility {
            exclude_done: true,
            exclude_archived: false,
        },
        include_comments: false,
        render: OrgizeSparseTreeRenderOptions {
            explain_skips: false,
        },
    })
    .unwrap_or_else(|error| panic!("render sparse tree: {error}"));

    assert!(
        rendered.contains("[SPARSE001] Match: Active memory"),
        "rendered: {rendered}"
    );
    assert!(!rendered.contains("Retired memory"), "rendered: {rendered}");
}

#[test]
fn render_sdd_status_uses_org_native_parent_edges() {
    let temp = tempdir_or_panic();
    let path = temp.path().join("sdd.org");
    std::fs::write(
        &path,
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

    let rendered = render_sdd_status(&OrgizeSddStatusRequest { paths: vec![path] })
        .unwrap_or_else(|error| panic!("render sdd status: {error}"));

    assert!(rendered.contains("[SDD]"), "rendered: {rendered}");
    assert!(
        rendered.contains("architecture nodes: 2"),
        "rendered: {rendered}"
    );
    assert!(
        rendered.contains("- view review: Runtime View"),
        "rendered: {rendered}"
    );
    assert!(
        rendered.contains("parent: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11 (System SDD)"),
        "rendered: {rendered}"
    );
    assert!(
        rendered.contains("viewpoint: runtime"),
        "rendered: {rendered}"
    );
}

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
            ":NEXT_ACTION: Materialize DuckDB snapshot\n",
            ":END:\n",
            "The active task should be indexed.\n",
            "** Evidence\n",
            "This inherited agent-tag section should not become a task row.\n",
            "** TODO Task Checklist\n",
            "This inherited agent-tag TODO section should not become a task row.\n",
            "* DONE Finished task :agent:\n",
            "CLOSED: [2026-05-16 Sat]\n",
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
    assert!(done.is_done);
    assert_eq!(done.closed.as_deref(), Some("[2026-05-16 Sat]"));
}

fn tempdir_or_panic() -> tempfile::TempDir {
    tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"))
}
