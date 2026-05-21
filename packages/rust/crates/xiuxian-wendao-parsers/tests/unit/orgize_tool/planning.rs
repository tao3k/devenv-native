use xiuxian_wendao_parsers::{OrgizeAgentPlanningRequest, render_agent_planning};

use super::support::tempdir_or_panic;

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
