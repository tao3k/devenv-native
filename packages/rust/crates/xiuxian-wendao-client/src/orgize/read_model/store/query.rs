use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::{Context, Result};
use xiuxian_wendao_parsers::OrgizeAgentTaskRepeater;

use crate::orgize::read_model::model::AgentOrgTaskListRow;

type AgentOrgTaskSqlRow = (
    String,
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

pub(in crate::orgize::read_model) fn query_agent_org_task_rows_matching(
    connection: &xiuxian_db_store::duckdb_crate::Connection,
    source_paths: &[PathBuf],
    text: Option<&str>,
    tags: &[String],
) -> Result<Vec<AgentOrgTaskListRow>> {
    let query = format!(
        "SELECT {} FROM agent_org_tasks WHERE {} ORDER BY archived ASC, is_done ASC, source_path ASC, source_line ASC",
        agent_org_task_list_columns(),
        task_match_predicate(source_paths, text, tags),
    );
    query_agent_org_task_rows_with_sql(connection, query.as_str())
}

pub(in crate::orgize::read_model) fn query_active_agent_org_task_row_window(
    connection: &xiuxian_db_store::duckdb_crate::Connection,
    source_paths: &[PathBuf],
    limit: usize,
) -> Result<AgentOrgTaskRowWindow> {
    let query = format!(
        "SELECT {}, COUNT(*) OVER () AS total_rows FROM agent_org_tasks WHERE {} AND is_done = false AND archived = false ORDER BY archived ASC, is_done ASC, source_path ASC, source_line ASC LIMIT {limit}",
        agent_org_task_list_columns(),
        source_path_predicate(source_paths),
    );
    query_agent_org_task_row_window_with_sql(connection, query.as_str())
}

pub(in crate::orgize::read_model) fn query_agent_org_task_rows_by_orgid(
    connection: &xiuxian_db_store::duckdb_crate::Connection,
    source_paths: &[PathBuf],
    orgid: &str,
) -> Result<Vec<AgentOrgTaskListRow>> {
    let query = format!(
        "SELECT {} FROM agent_org_tasks WHERE ({}) AND orgid = {} ORDER BY source_path ASC, source_line ASC LIMIT 2",
        agent_org_task_list_columns(),
        source_path_predicate(source_paths),
        sql_string_literal(orgid),
    );
    query_agent_org_task_rows_with_sql(connection, query.as_str())
}

fn agent_org_task_list_columns() -> &'static str {
    r"
    orgid,
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
    properties_json
"
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
                row.get::<_, String>(1)?,
                row.get::<_, u64>(2)?,
                row.get::<_, u64>(3)?,
                row.get::<_, u64>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, bool>(7)?,
                row.get::<_, bool>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, String>(10)?,
                row.get::<_, Option<String>>(11)?,
                row.get::<_, Option<String>>(12)?,
                row.get::<_, Option<String>>(13)?,
                row.get::<_, Option<String>>(14)?,
                row.get::<_, Option<String>>(15)?,
                row.get::<_, u64>(16)?,
                row.get::<_, String>(17)?,
                row.get::<_, String>(18)?,
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
                    row.get::<_, String>(1)?,
                    row.get::<_, u64>(2)?,
                    row.get::<_, u64>(3)?,
                    row.get::<_, u64>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, bool>(7)?,
                    row.get::<_, bool>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, String>(10)?,
                    row.get::<_, Option<String>>(11)?,
                    row.get::<_, Option<String>>(12)?,
                    row.get::<_, Option<String>>(13)?,
                    row.get::<_, Option<String>>(14)?,
                    row.get::<_, Option<String>>(15)?,
                    row.get::<_, u64>(16)?,
                    row.get::<_, String>(17)?,
                    row.get::<_, String>(18)?,
                ),
                row.get::<_, i64>(19)?,
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

fn task_match_predicate(source_paths: &[PathBuf], text: Option<&str>, tags: &[String]) -> String {
    let mut predicates = vec![format!("({})", source_path_predicate(source_paths))];
    if let Some(text_predicate) = text.and_then(text_match_predicate) {
        predicates.push(format!("({text_predicate})"));
    }
    predicates.extend(tags.iter().filter_map(|tag| tag_match_predicate(tag)));
    predicates.join(" AND ")
}

fn text_match_predicate(text: &str) -> Option<String> {
    let needle = text.trim().to_lowercase();
    if needle.is_empty() {
        return None;
    }
    let needle = sql_string_literal(needle.as_str());
    Some(
        [
            "orgid",
            "source_path",
            "title",
            "todo_state",
            "outline_path_json",
            "tags_json",
            "effective_tags_json",
            "scheduled",
            "scheduled_repeater_json",
            "deadline",
            "deadline_repeater_json",
            "closed",
            "properties_json",
        ]
        .into_iter()
        .map(|column| format!("contains(lower(coalesce({column}, '')), {needle})"))
        .collect::<Vec<_>>()
        .join(" OR "),
    )
}

fn tag_match_predicate(tag: &str) -> Option<String> {
    let tag = tag.trim().trim_matches(':').to_lowercase();
    if tag.is_empty() {
        return None;
    }
    let json_tag = serde_json::to_string(&tag).ok()?;
    let needle = sql_string_literal(json_tag.as_str());
    Some(format!(
        "(contains(lower(tags_json), {needle}) OR contains(lower(effective_tags_json), {needle}))"
    ))
}

fn sql_string_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn source_modified_unix_ms(source_path: &str) -> u64 {
    std::fs::metadata(source_path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis().try_into().unwrap_or(u64::MAX))
        .unwrap_or_default()
}

fn decode_agent_org_task_row(row: AgentOrgTaskSqlRow) -> Result<AgentOrgTaskListRow> {
    let (
        orgid,
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
    let source_modified_unix_ms = source_modified_unix_ms(&source_path);
    Ok(AgentOrgTaskListRow {
        orgid,
        source_path,
        source_modified_unix_ms,
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
