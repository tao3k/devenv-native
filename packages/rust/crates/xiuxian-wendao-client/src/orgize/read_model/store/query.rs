use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::{Context, Result};
use xiuxian_wendao_parsers::OrgizeAgentTaskRepeater;

use crate::orgize::read_model::model::{AgentOrgElementMatch, AgentOrgTaskListRow};

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

type AgentOrgElementMatchSqlRow = (
    u64,
    String,
    String,
    Option<String>,
    String,
    String,
    Option<String>,
    u64,
    u64,
    u64,
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
        "SELECT DISTINCT {} FROM agent_org_tasks AS task LEFT JOIN agent_org_memory_objects AS memory ON memory.orgid = task.orgid LEFT JOIN agent_org_elements AS element ON element.source_path = task.source_path AND element.source_range_start >= task.source_range_start AND element.source_range_start < task.source_range_end WHERE {} ORDER BY task.archived ASC, task.is_done ASC, task.source_path ASC, task.source_line ASC",
        agent_org_task_list_columns(),
        task_match_predicate(source_paths, text, tags),
    );
    let mut rows = query_agent_org_task_rows_with_sql(connection, query.as_str())?;
    if let Some(needle) = normalized_text_filter(text) {
        attach_org_element_matches(connection, &mut rows, &needle)?;
    }
    Ok(rows)
}

pub(in crate::orgize::read_model) fn query_active_agent_org_task_row_window(
    connection: &xiuxian_db_store::duckdb_crate::Connection,
    source_paths: &[PathBuf],
    limit: usize,
) -> Result<AgentOrgTaskRowWindow> {
    let query = format!(
        "SELECT {}, COUNT(*) OVER () AS total_rows FROM agent_org_tasks AS task WHERE {} AND task.is_done = false AND task.archived = false ORDER BY task.archived ASC, task.is_done ASC, task.source_path ASC, task.source_line ASC LIMIT {limit}",
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
        "SELECT {} FROM agent_org_tasks AS task WHERE ({}) AND task.orgid = {} ORDER BY task.source_path ASC, task.source_line ASC LIMIT 2",
        agent_org_task_list_columns(),
        source_path_predicate(source_paths),
        sql_string_literal(orgid),
    );
    query_agent_org_task_rows_with_sql(connection, query.as_str())
}

fn agent_org_task_list_columns() -> &'static str {
    r"
    task.orgid,
    task.source_path,
    task.source_line,
    task.source_range_start,
    task.source_range_end,
    task.title,
    task.todo_state,
    task.is_done,
    task.archived,
    task.tags_json,
    task.effective_tags_json,
    task.scheduled,
    task.scheduled_repeater_json,
    task.deadline,
    task.deadline_repeater_json,
    task.closed,
    task.level,
    task.outline_path_json,
    task.properties_json
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

fn attach_org_element_matches(
    connection: &xiuxian_db_store::duckdb_crate::Connection,
    task_rows: &mut [AgentOrgTaskListRow],
    needle: &str,
) -> Result<()> {
    for row in task_rows {
        row.matched_org_elements = query_org_element_matches_for_task(connection, row, needle)?;
    }
    Ok(())
}

fn query_org_element_matches_for_task(
    connection: &xiuxian_db_store::duckdb_crate::Connection,
    task: &AgentOrgTaskListRow,
    needle: &str,
) -> Result<Vec<AgentOrgElementMatch>> {
    let needle = sql_string_literal(needle);
    let query = format!(
        "SELECT ordinal, category, kind, affiliated_name, context, summary_json, language, source_start_line, source_range_start, source_range_end, source_raw \
         FROM agent_org_elements AS element \
         WHERE element.source_path = {} \
           AND element.source_range_start >= {} \
           AND element.source_range_start < {} \
           AND element.kind NOT IN ('org-data', 'target-definition', 'headline') \
           AND ({}) \
         ORDER BY CASE \
             WHEN element.kind = 'paragraph' THEN 0 \
             WHEN element.context = 'paragraph' THEN 1 \
             WHEN element.category = 'property' THEN 2 \
             WHEN element.context IN ('headlineTitle', 'targetAlias') THEN 3 \
             ELSE 4 \
           END ASC, \
           (element.source_range_end - element.source_range_start) ASC, \
           element.source_range_start ASC, \
           element.ordinal ASC \
         LIMIT 8",
        sql_string_literal(&task.source_path),
        task.source_range_start,
        task.source_range_end,
        org_element_direct_text_match_predicates(&needle)
            .collect::<Vec<_>>()
            .join(" OR "),
    );
    let mut statement = connection
        .prepare(query.as_str())
        .with_context(|| "failed to prepare agent org-element match query")?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, u64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, u64>(7)?,
                row.get::<_, u64>(8)?,
                row.get::<_, u64>(9)?,
                row.get::<_, String>(10)?,
            ))
        })
        .with_context(|| "failed to query agent org-element matches")?;

    let mut matches = Vec::new();
    for row in rows {
        matches.push(decode_agent_org_element_match(row?));
    }
    Ok(matches)
}

fn decode_agent_org_element_match(row: AgentOrgElementMatchSqlRow) -> AgentOrgElementMatch {
    let (
        ordinal,
        category,
        kind,
        affiliated_name,
        context,
        summary_json,
        language,
        source_start_line,
        source_range_start,
        source_range_end,
        source_raw,
    ) = row;
    AgentOrgElementMatch {
        ordinal,
        category,
        kind,
        affiliated_name,
        context,
        summary_json,
        language,
        source_start_line,
        source_range_start,
        source_range_end,
        source_raw,
    }
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
        format!("(task.source_path = {source} OR starts_with(task.source_path, {prefix}))")
    } else {
        format!("task.source_path = {source}")
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
    let needle = normalized_text_filter(Some(text))?;
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
        .map(|column| format!("contains(lower(coalesce(task.{column}, '')), {needle})"))
        .chain(memory_object_text_match_predicates(&needle))
        .chain(org_element_text_match_predicates(&needle))
        .collect::<Vec<_>>()
        .join(" OR "),
    )
}

fn normalized_text_filter(text: Option<&str>) -> Option<String> {
    let needle = text?.trim().to_lowercase();
    (!needle.is_empty()).then_some(needle)
}

fn memory_object_text_match_predicates(needle: &str) -> impl Iterator<Item = String> + '_ {
    ["kind", "facet", "source_kind", "source_key", "value"]
        .into_iter()
        .map(move |column| format!("contains(lower(coalesce(memory.{column}, '')), {needle})"))
}

fn org_element_text_match_predicates(needle: &str) -> impl Iterator<Item = String> + '_ {
    [
        "category",
        "kind",
        "affiliated_name",
        "outline_path_json",
        "context",
        "summary_json",
        "language",
        "source_raw",
    ]
    .into_iter()
    .map(move |column| format!("contains(lower(coalesce(element.{column}, '')), {needle})"))
}

fn org_element_direct_text_match_predicates(needle: &str) -> impl Iterator<Item = String> + '_ {
    [
        "category",
        "kind",
        "affiliated_name",
        "context",
        "language",
        "source_raw",
    ]
    .into_iter()
    .map(move |column| format!("contains(lower(coalesce(element.{column}, '')), {needle})"))
}

fn tag_match_predicate(tag: &str) -> Option<String> {
    let tag = tag.trim().trim_matches(':').to_lowercase();
    if tag.is_empty() {
        return None;
    }
    let json_tag = serde_json::to_string(&tag).ok()?;
    let needle = sql_string_literal(json_tag.as_str());
    Some(format!(
        "(contains(lower(task.tags_json), {needle}) OR contains(lower(task.effective_tags_json), {needle}))"
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
        matched_org_elements: Vec::new(),
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
