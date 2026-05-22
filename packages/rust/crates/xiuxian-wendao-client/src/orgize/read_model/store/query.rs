use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use xiuxian_wendao_parsers::OrgizeAgentTaskRepeater;

use crate::orgize::read_model::model::{
    AGENT_ORG_TASK_LIST_COLUMNS, AGENT_ORG_TASK_LIST_QUERY, AgentOrgTaskListRow,
};

type AgentOrgTaskSqlRow = (
    String,
    u64,
    u64,
    u64,
    String,
    Option<String>,
    bool,
    bool,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    u64,
    String,
    String,
);

pub(in crate::orgize::read_model) struct AgentOrgTaskRowWindow {
    pub(in crate::orgize::read_model) total_rows: usize,
    pub(in crate::orgize::read_model) rows: Vec<AgentOrgTaskListRow>,
}

pub(in crate::orgize::read_model) fn query_agent_org_task_rows(
    connection: &xiuxian_db_store::duckdb_crate::Connection,
) -> Result<Vec<AgentOrgTaskListRow>> {
    query_agent_org_task_rows_with_sql(connection, AGENT_ORG_TASK_LIST_QUERY)
}

pub(in crate::orgize::read_model) fn query_active_agent_org_task_row_window(
    connection: &xiuxian_db_store::duckdb_crate::Connection,
    source_paths: &[PathBuf],
    limit: usize,
) -> Result<AgentOrgTaskRowWindow> {
    let query = format!(
        "SELECT {AGENT_ORG_TASK_LIST_COLUMNS}, COUNT(*) OVER () AS total_rows FROM agent_org_tasks WHERE {} AND is_done = false AND archived = false ORDER BY archived ASC, is_done ASC, source_path ASC, source_line ASC LIMIT {limit}",
        source_path_predicate(source_paths),
    );
    query_agent_org_task_row_window_with_sql(connection, query.as_str())
}

fn query_agent_org_task_rows_with_sql(
    connection: &xiuxian_db_store::duckdb_crate::Connection,
    query: &str,
) -> Result<Vec<AgentOrgTaskListRow>> {
    let mut statement = connection
        .prepare(query)
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
                row.get::<_, u64>(15)?,
                row.get::<_, String>(16)?,
                row.get::<_, String>(17)?,
            ))
        })
        .with_context(|| "failed to query agent task-list rows")?;

    let mut task_rows = Vec::new();
    for row in rows {
        task_rows.push(decode_agent_org_task_row(row?)?);
    }
    Ok(task_rows)
}

fn query_agent_org_task_row_window_with_sql(
    connection: &xiuxian_db_store::duckdb_crate::Connection,
    query: &str,
) -> Result<AgentOrgTaskRowWindow> {
    let mut statement = connection
        .prepare(query)
        .with_context(|| "failed to prepare cached active agent task-list query")?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                (
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
                    row.get::<_, u64>(15)?,
                    row.get::<_, String>(16)?,
                    row.get::<_, String>(17)?,
                ),
                row.get::<_, i64>(18)?,
            ))
        })
        .with_context(|| "failed to query cached active agent task-list rows")?;

    let mut total_rows = 0;
    let mut task_rows = Vec::new();
    for row in rows {
        let (row, row_total) =
            row.with_context(|| "failed to read cached active agent task-list row")?;
        total_rows = usize::try_from(row_total)
            .with_context(|| "cached active agent task row count overflowed usize")?;
        task_rows.push(decode_agent_org_task_row(row)?);
    }
    Ok(AgentOrgTaskRowWindow {
        total_rows,
        rows: task_rows,
    })
}

fn source_path_predicate(source_paths: &[PathBuf]) -> String {
    if source_paths.is_empty() {
        return "true".to_string();
    }
    source_paths
        .iter()
        .map(|path| source_path_condition(path.as_path()))
        .collect::<Vec<_>>()
        .join(" OR ")
}

fn source_path_condition(source_path: &Path) -> String {
    let source = sql_string_literal(source_path.to_string_lossy().as_ref());
    if source_path.is_dir() {
        let mut prefix = source_path.to_string_lossy().to_string();
        if !prefix.ends_with(std::path::MAIN_SEPARATOR) {
            prefix.push(std::path::MAIN_SEPARATOR);
        }
        let prefix = sql_string_literal(prefix.as_str());
        format!("(source_path = {source} OR starts_with(source_path, {prefix}))")
    } else {
        format!("source_path = {source}")
    }
}

fn sql_string_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn decode_agent_org_task_row(row: AgentOrgTaskSqlRow) -> Result<AgentOrgTaskListRow> {
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
        level,
        outline_path_json,
        properties_json,
    ) = row;
    let outline_path = serde_json::from_str(&outline_path_json)
        .with_context(|| "failed to decode agent task-list outline path")?;
    let tags = serde_json::from_str(&tags_json)
        .with_context(|| "failed to decode agent task-list tags")?;
    let effective_tags = serde_json::from_str(&effective_tags_json)
        .with_context(|| "failed to decode agent task-list effective tags")?;
    let properties = serde_json::from_str(&properties_json)
        .with_context(|| "failed to decode agent task-list properties")?;
    let scheduled_repeater = decode_repeater_json(scheduled_repeater_json.as_deref(), "scheduled")?;
    let deadline_repeater = decode_repeater_json(deadline_repeater_json.as_deref(), "deadline")?;
    Ok(AgentOrgTaskListRow {
        source_path,
        source_line,
        source_range_start,
        source_range_end,
        level,
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
    })
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
