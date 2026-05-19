use anyhow::{Context, Result};
use xiuxian_wendao_parsers::OrgizeAgentTaskRepeater;

use crate::orgize::read_model::model::{AGENT_ORG_TASK_LIST_QUERY, AgentOrgTaskListRow};

pub(in crate::orgize::read_model) fn query_agent_org_task_rows(
    connection: &xiuxian_db_store::duckdb_crate::Connection,
) -> Result<Vec<AgentOrgTaskListRow>> {
    let mut statement = connection
        .prepare(AGENT_ORG_TASK_LIST_QUERY)
        .with_context(|| "failed to prepare agent task-list query")?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, u64>(1)?,
                row.get::<_, u64>(2)?,
                row.get::<_, u64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, bool>(6)?,
                row.get::<_, bool>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, Option<String>>(10)?,
                row.get::<_, Option<String>>(11)?,
                row.get::<_, Option<String>>(12)?,
                row.get::<_, Option<String>>(13)?,
                row.get::<_, Option<String>>(14)?,
                row.get::<_, String>(15)?,
                row.get::<_, String>(16)?,
            ))
        })
        .with_context(|| "failed to query agent task-list rows")?;

    let mut task_rows = Vec::new();
    for row in rows {
        let (
            source_path,
            source_line,
            source_range_start,
            source_range_end,
            title,
            todo_state,
            is_done,
            archived,
            tags_json,
            effective_tags_json,
            scheduled,
            scheduled_repeater_json,
            deadline,
            deadline_repeater_json,
            closed,
            outline_path_json,
            properties_json,
        ) = row.with_context(|| "failed to read agent task-list row")?;
        let outline_path = serde_json::from_str(&outline_path_json)
            .with_context(|| "failed to decode agent task-list outline path")?;
        let tags = serde_json::from_str(&tags_json)
            .with_context(|| "failed to decode agent task-list tags")?;
        let effective_tags = serde_json::from_str(&effective_tags_json)
            .with_context(|| "failed to decode agent task-list effective tags")?;
        let properties = serde_json::from_str(&properties_json)
            .with_context(|| "failed to decode agent task-list properties")?;
        let scheduled_repeater =
            decode_repeater_json(scheduled_repeater_json.as_deref(), "scheduled")?;
        let deadline_repeater =
            decode_repeater_json(deadline_repeater_json.as_deref(), "deadline")?;
        task_rows.push(AgentOrgTaskListRow {
            source_path,
            source_line,
            source_range_start,
            source_range_end,
            title,
            todo_state,
            is_done,
            archived,
            tags,
            effective_tags,
            scheduled,
            scheduled_repeater,
            deadline,
            deadline_repeater,
            closed,
            outline_path,
            properties,
        });
    }
    Ok(task_rows)
}

fn decode_repeater_json(
    value: Option<&str>,
    planning_kind: &str,
) -> Result<Option<OrgizeAgentTaskRepeater>> {
    value
        .map(|value| {
            serde_json::from_str(value).with_context(|| {
                format!("failed to decode agent task-list {planning_kind} repeater")
            })
        })
        .transpose()
}
