use anyhow::{Context, Result};
use xiuxian_wendao_parsers::{OrgizeAgentTaskRow, OrgizeOrgElementRow};

use crate::orgize::read_model::memory::org_inferred_memory_objects_for_row;
use crate::orgize::read_model::model::{
    AgentOrgReadModelMaterializationReport, ResolvedReadModelSettings,
};

use super::connection::open_read_model_connection;
use super::schema::{
    initialize_agent_org_elements_table, initialize_agent_org_memory_objects_table,
    initialize_agent_org_tasks_table,
};

pub(super) fn materialize_agent_org_tasks(
    settings: &ResolvedReadModelSettings,
    rows: &[OrgizeAgentTaskRow],
    element_rows: &[OrgizeOrgElementRow],
) -> Result<AgentOrgReadModelMaterializationReport> {
    let connection = open_read_model_connection(settings)?;
    replace_agent_org_task_rows_in_transaction(&connection, rows, element_rows)?;

    let done_rows = rows.iter().filter(|row| row.is_done).count();
    let archived_rows = rows.iter().filter(|row| row.archived).count();
    let active_rows = rows
        .iter()
        .filter(|row| !row.is_done && !row.archived)
        .count();
    let memory_object_rows = rows
        .iter()
        .map(agent_task_list_row_from_parser_row)
        .map(|row| org_inferred_memory_objects_for_row(&row).len())
        .sum();
    Ok(AgentOrgReadModelMaterializationReport {
        rows: rows.len(),
        active_rows,
        done_rows,
        archived_rows,
        memory_object_rows,
        org_element_rows: element_rows.len(),
    })
}

fn replace_agent_org_task_rows_in_transaction(
    connection: &xiuxian_db_store::duckdb_crate::Connection,
    rows: &[OrgizeAgentTaskRow],
    element_rows: &[OrgizeOrgElementRow],
) -> Result<()> {
    connection
        .execute_batch("BEGIN TRANSACTION;")
        .with_context(|| "failed to begin agent_org_tasks refresh transaction")?;
    let result = (|| {
        initialize_agent_org_tasks_table(connection)?;
        initialize_agent_org_memory_objects_table(connection)?;
        initialize_agent_org_elements_table(connection)?;
        replace_agent_org_task_rows(connection, rows)
            .and_then(|()| replace_agent_org_memory_object_rows(connection, rows))
            .and_then(|()| replace_agent_org_element_rows(connection, element_rows))
    })();
    match result {
        Ok(()) => connection
            .execute_batch("COMMIT;")
            .with_context(|| "failed to commit agent_org_tasks refresh transaction"),
        Err(error) => {
            let _ = connection.execute_batch("ROLLBACK;");
            Err(error)
        }
    }
}

fn replace_agent_org_element_rows(
    connection: &xiuxian_db_store::duckdb_crate::Connection,
    rows: &[OrgizeOrgElementRow],
) -> Result<()> {
    let mut appender = connection
        .appender("agent_org_elements")
        .with_context(|| "failed to open agent_org_elements DuckDB appender")?;

    for row in rows {
        appender
            .append_row(xiuxian_db_store::duckdb_crate::params![
                row.source_path.as_str(),
                row.source_modified_unix_ms,
                row.ordinal,
                row.category.as_str(),
                row.kind.as_str(),
                row.affiliated_name.as_deref(),
                row.outline_path_json.as_str(),
                row.context.as_str(),
                row.summary_json.as_str(),
                row.language.as_deref(),
                row.source_start_line,
                row.source_start_column,
                row.source_end_line,
                row.source_end_column,
                row.source_range_start,
                row.source_range_end,
                row.source_raw.as_str(),
            ])
            .with_context(|| {
                format!(
                    "failed to insert Org element `{}` from `{}`",
                    row.kind, row.source_path
                )
            })?;
    }
    appender
        .flush()
        .with_context(|| "failed to flush agent_org_elements DuckDB appender")?;
    Ok(())
}

fn replace_agent_org_task_rows(
    connection: &xiuxian_db_store::duckdb_crate::Connection,
    rows: &[OrgizeAgentTaskRow],
) -> Result<()> {
    let mut appender = connection
        .appender("agent_org_tasks")
        .with_context(|| "failed to open agent_org_tasks DuckDB appender")?;

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
        appender
            .append_row(xiuxian_db_store::duckdb_crate::params![
                row.orgid.as_str(),
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
    appender
        .flush()
        .with_context(|| "failed to flush agent_org_tasks DuckDB appender")?;
    Ok(())
}

fn replace_agent_org_memory_object_rows(
    connection: &xiuxian_db_store::duckdb_crate::Connection,
    rows: &[OrgizeAgentTaskRow],
) -> Result<()> {
    let task_rows = rows
        .iter()
        .map(agent_task_list_row_from_parser_row)
        .collect::<Vec<_>>();
    let mut appender = connection
        .appender("agent_org_memory_objects")
        .with_context(|| "failed to open agent_org_memory_objects DuckDB appender")?;

    for row in &task_rows {
        for (index, projection) in org_inferred_memory_objects_for_row(row)
            .into_iter()
            .enumerate()
        {
            let object_index =
                u64::try_from(index + 1).with_context(|| "memory object index overflowed u64")?;
            appender
                .append_row(xiuxian_db_store::duckdb_crate::params![
                    row.orgid.as_str(),
                    row.source_path.as_str(),
                    row.source_line,
                    row.source_range_start,
                    row.source_range_end,
                    object_index,
                    projection.object.kind.name(),
                    projection.object.kind.facet_label(),
                    projection.source_kind.as_str(),
                    projection.source_key.as_str(),
                    projection.object.value.as_str()
                ])
                .with_context(|| {
                    format!(
                        "failed to insert memory object for Org task `{}` from `{}`",
                        row.title, row.source_path
                    )
                })?;
        }
    }
    appender
        .flush()
        .with_context(|| "failed to flush agent_org_memory_objects DuckDB appender")?;
    Ok(())
}

fn agent_task_list_row_from_parser_row(
    row: &OrgizeAgentTaskRow,
) -> crate::orgize::read_model::model::AgentOrgTaskListRow {
    crate::orgize::read_model::model::AgentOrgTaskListRow {
        orgid: row.orgid.clone(),
        source_path: row.source_path.clone(),
        source_modified_unix_ms: 0,
        source_line: row.source_line,
        source_range_start: row.source_range_start,
        source_range_end: row.source_range_end,
        level: row.level,
        title: row.title.clone(),
        todo_state: row.todo_state.clone(),
        is_done: row.is_done,
        archived: row.archived,
        tags: row.tags.clone(),
        effective_tags: row.effective_tags.clone(),
        scheduled: row.scheduled.clone(),
        scheduled_repeater: row.scheduled_repeater.clone(),
        deadline: row.deadline.clone(),
        deadline_repeater: row.deadline_repeater.clone(),
        closed: row.closed.clone(),
        outline_path: row.outline_path.clone(),
        properties: row.properties.clone(),
        matched_org_elements: Vec::new(),
    }
}
