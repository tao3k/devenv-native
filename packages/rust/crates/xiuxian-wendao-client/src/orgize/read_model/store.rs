//! `DuckDB` storage and materialization for Org agent tasks.

use std::path::PathBuf;

use anyhow::{Context, Result};
use xiuxian_db_store::duckdb::{
    DuckDbDatabasePath, DuckDbExecutionConfig, DuckDbRuntimeConfig, open_duckdb_connection,
};
use xiuxian_wendao_parsers::{
    OrgizeAgentTaskReadModelRequest, OrgizeAgentTaskRepeater, OrgizeAgentTaskRow,
    collect_agent_task_rows,
};

use crate::ClientContext;

use super::model::{
    AGENT_ORG_TASK_LIST_QUERY, AgentOrgReadModelMaterializationReport, AgentOrgTaskListRow,
    RefreshedAgentOrgReadModel, ResolvedReadModelSettings,
};
use super::settings::{resolve_read_model_settings, resolve_source_paths};

pub(super) fn refresh_agent_org_read_model(
    paths: &[PathBuf],
    context: &ClientContext,
) -> Result<RefreshedAgentOrgReadModel> {
    let settings = resolve_read_model_settings(context)?;
    let source_paths = resolve_source_paths(paths, context, settings.cache_home.as_path());
    let report = collect_agent_task_rows(&OrgizeAgentTaskReadModelRequest {
        paths: source_paths.clone(),
        match_expression: Some("+agent".to_string()),
        include_comments: false,
    })?;
    let materialized = materialize_agent_org_tasks(&settings, &report.rows).with_context(|| {
        format!(
            "failed to materialize Org agent read model at `{}`",
            settings.database_path.display()
        )
    })?;

    Ok(RefreshedAgentOrgReadModel {
        settings,
        source_paths,
        materialized,
    })
}

fn materialize_agent_org_tasks(
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

pub(super) fn open_read_model_connection(
    settings: &ResolvedReadModelSettings,
) -> Result<xiuxian_db_store::duckdb_crate::Connection> {
    let runtime = DuckDbRuntimeConfig {
        enabled: true,
        database_path: DuckDbDatabasePath::File(settings.database_path.clone()),
        temp_directory: settings.temp_directory.clone(),
        threads: settings.threads,
        execution: DuckDbExecutionConfig {
            preserve_insertion_order: true,
            parquet_metadata_cache: true,
            prefer_virtual_arrow: true,
        },
        memory_limit: settings.memory_limit.clone(),
        max_temp_directory_size: settings.max_temp_directory_size.clone(),
        materialize_threshold_rows: settings.materialize_threshold_rows,
    };
    open_duckdb_connection(&runtime).map_err(anyhow::Error::msg)
}

pub(super) fn query_agent_org_task_rows(
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

fn initialize_agent_org_tasks_table(
    connection: &xiuxian_db_store::duckdb_crate::Connection,
) -> Result<()> {
    connection
        .execute_batch(
            r"
DROP TABLE IF EXISTS agent_org_tasks;
CREATE TABLE agent_org_tasks (
    task_id VARCHAR PRIMARY KEY,
    source_path VARCHAR NOT NULL,
    source_line UBIGINT NOT NULL,
    source_column UBIGINT NOT NULL,
    source_range_start UBIGINT NOT NULL,
    source_range_end UBIGINT NOT NULL,
    level UBIGINT NOT NULL,
    outline_path_json VARCHAR NOT NULL,
    title VARCHAR NOT NULL,
    todo_state VARCHAR,
    is_done BOOLEAN NOT NULL,
    tags_json VARCHAR NOT NULL,
    effective_tags_json VARCHAR NOT NULL,
    scheduled VARCHAR,
    scheduled_repeater_json VARCHAR,
    deadline VARCHAR,
    deadline_repeater_json VARCHAR,
    closed VARCHAR,
    archived BOOLEAN NOT NULL,
    archive_location VARCHAR,
    properties_json VARCHAR NOT NULL
);
",
        )
        .with_context(|| "failed to initialize agent_org_tasks DuckDB table")
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
