use anyhow::{Context, Result};
use xiuxian_wendao_parsers::OrgizeAgentTaskRow;

use crate::orgize::read_model::model::{
    AgentOrgReadModelMaterializationReport, ResolvedReadModelSettings,
};

use super::connection::open_read_model_connection;
use super::schema::initialize_agent_org_tasks_table;

pub(super) fn materialize_agent_org_tasks(
    settings: &ResolvedReadModelSettings,
    rows: &[OrgizeAgentTaskRow],
) -> Result<AgentOrgReadModelMaterializationReport> {
    let connection = open_read_model_connection(settings)?;
    initialize_agent_org_tasks_table(&connection)?;
    replace_agent_org_task_rows(&connection, rows)?;

    let done_rows = rows.iter().filter(|row| row.is_done).count();
    let archived_rows = rows.iter().filter(|row| row.archived).count();
    let active_rows = rows
        .iter()
        .filter(|row| !row.is_done && !row.archived)
        .count();
    Ok(AgentOrgReadModelMaterializationReport {
        rows: rows.len(),
        active_rows,
        done_rows,
        archived_rows,
    })
}

fn replace_agent_org_task_rows(
    connection: &xiuxian_db_store::duckdb_crate::Connection,
    rows: &[OrgizeAgentTaskRow],
) -> Result<()> {
    let mut statement = connection
        .prepare(
            r"
INSERT INTO agent_org_tasks (
    task_id,
    source_path,
    source_line,
    source_column,
    source_range_start,
    source_range_end,
    level,
    outline_path_json,
    title,
    todo_state,
    is_done,
    tags_json,
    effective_tags_json,
    scheduled,
    scheduled_repeater_json,
    deadline,
    deadline_repeater_json,
    closed,
    archived,
    archive_location,
    properties_json
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)
",
        )
        .with_context(|| "failed to prepare agent_org_tasks insert statement")?;

    for row in rows {
        let outline_path_json = serde_json::to_string(&row.outline_path)
            .with_context(|| "failed to serialize Org task outline path")?;
        let tags_json = serde_json::to_string(&row.tags)
            .with_context(|| "failed to serialize Org task tags")?;
        let effective_tags_json = serde_json::to_string(&row.effective_tags)
            .with_context(|| "failed to serialize Org task effective tags")?;
        let properties_json = serde_json::to_string(&row.properties)
            .with_context(|| "failed to serialize Org task properties")?;
        let scheduled_repeater_json = row
            .scheduled_repeater
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .with_context(|| "failed to serialize Org task scheduled repeater")?;
        let deadline_repeater_json = row
            .deadline_repeater
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .with_context(|| "failed to serialize Org task deadline repeater")?;
        statement
            .execute(xiuxian_db_store::duckdb_crate::params![
                row.task_id.as_str(),
                row.source_path.as_str(),
                row.source_line,
                row.source_column,
                row.source_range_start,
                row.source_range_end,
                row.level,
                outline_path_json.as_str(),
                row.title.as_str(),
                row.todo_state.as_deref(),
                row.is_done,
                tags_json.as_str(),
                effective_tags_json.as_str(),
                row.scheduled.as_deref(),
                scheduled_repeater_json.as_deref(),
                row.deadline.as_deref(),
                deadline_repeater_json.as_deref(),
                row.closed.as_deref(),
                row.archived,
                row.archive_location.as_deref(),
                properties_json.as_str()
            ])
            .with_context(|| {
                format!(
                    "failed to insert Org task `{}` from `{}`",
                    row.title, row.source_path
                )
            })?;
    }
    Ok(())
}
