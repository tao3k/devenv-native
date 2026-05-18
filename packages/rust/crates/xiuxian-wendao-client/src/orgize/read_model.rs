//! DuckDB-backed read-model materialization for Orgize agent tasks.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;
use xiuxian_config_core::{
    load_toml_value_with_imports_and_paths, resolve_cache_home, resolve_config_home,
    resolve_path_from_value,
};
use xiuxian_db_store::duckdb::{
    DuckDbDatabasePath, DuckDbExecutionConfig, DuckDbRuntimeConfig, open_duckdb_connection,
};
use xiuxian_wendao_parsers::{
    OrgizeAgentTaskProperty, OrgizeAgentTaskReadModelRequest, OrgizeAgentTaskRepeater,
    OrgizeAgentTaskRow, collect_agent_task_rows,
};

use crate::orgize::{
    OrgizeReadModelArgs, OrgizeTaskArchiveArgs, OrgizeTaskListArgs, OrgizeTaskReportArgs,
};
use crate::{ClientContext, CommandOutcome};

const AGENT_ORG_TASKS_TABLE: &str = "agent_org_tasks";
const AGENT_ORG_TASK_LIST_QUERY: &str = r"
SELECT
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
    properties_json
FROM agent_org_tasks
ORDER BY archived ASC, is_done ASC, source_path ASC, source_line ASC
";

pub(crate) fn run_read_model(
    args: &OrgizeReadModelArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let refreshed = refresh_agent_org_read_model(&args.paths, context)?;

    println!("orgize agent read-model materialized");
    println!("backend: duckdb");
    println!("database: {}", refreshed.settings.database_path.display());
    println!("table: {AGENT_ORG_TASKS_TABLE}");
    println!("sources: {}", refreshed.source_paths.len());
    println!("rows: {}", refreshed.materialized.rows);
    println!("active: {}", refreshed.materialized.active_rows);
    println!("done: {}", refreshed.materialized.done_rows);
    println!("archived: {}", refreshed.materialized.archived_rows);
    Ok(CommandOutcome::success())
}

pub(crate) fn run_task_list(
    args: &OrgizeTaskListArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let refreshed = refresh_agent_org_read_model(&args.paths, context)?;
    let connection = open_read_model_connection(&refreshed.settings)?;
    let rows = query_agent_org_task_rows(&connection)?;
    let filtered = filter_task_rows(&rows, args);
    let limit = args.limit;
    let shown = filtered.iter().take(limit).collect::<Vec<_>>();

    println!("orgize agent task-list");
    println!("backend: duckdb");
    println!("database: {}", refreshed.settings.database_path.display());
    println!("table: {AGENT_ORG_TASKS_TABLE}");
    println!("sources: {}", refreshed.source_paths.len());
    println!("rows: {}", filtered.len());
    println!("showing: {}", shown.len());
    println!(
        "active: {}",
        filtered
            .iter()
            .filter(|row| !row.is_done && !row.archived)
            .count()
    );
    println!(
        "done: {}",
        filtered.iter().filter(|row| row.is_done).count()
    );
    println!(
        "archived: {}",
        filtered.iter().filter(|row| row.archived).count()
    );
    for (index, row) in shown.iter().enumerate() {
        render_task_list_row(index + 1, row, context);
    }
    Ok(CommandOutcome::success())
}

pub(crate) fn run_task_report(
    args: &OrgizeTaskReportArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let refreshed = refresh_agent_org_read_model(&args.paths, context)?;
    let connection = open_read_model_connection(&refreshed.settings)?;
    let rows = query_agent_org_task_rows(&connection)?;
    let filtered = filter_report_rows(&rows, args);
    let limit = args.limit;
    let active_rows = filtered
        .iter()
        .filter(|row| !row.is_done && !row.archived)
        .copied()
        .collect::<Vec<_>>();
    let done_rows = filtered
        .iter()
        .filter(|row| row.is_done)
        .copied()
        .collect::<Vec<_>>();
    let archived_rows = filtered
        .iter()
        .filter(|row| row.archived)
        .copied()
        .collect::<Vec<_>>();
    let archive_candidates = filtered
        .iter()
        .filter(|row| row.is_done && !row.archived)
        .copied()
        .collect::<Vec<_>>();
    let achievements = filtered
        .iter()
        .filter(|row| task_row_has_tag(row, "achievement"))
        .copied()
        .collect::<Vec<_>>();
    let repeating_rows = filtered
        .iter()
        .filter(|row| row.scheduled_repeater.is_some() || row.deadline_repeater.is_some())
        .copied()
        .collect::<Vec<_>>();

    println!("orgize agent task-report");
    println!("backend: duckdb");
    println!("database: {}", refreshed.settings.database_path.display());
    println!("table: {AGENT_ORG_TASKS_TABLE}");
    println!("sources: {}", refreshed.source_paths.len());
    println!("rows: {}", filtered.len());
    println!("active: {}", active_rows.len());
    println!("done: {}", done_rows.len());
    println!("archived: {}", archived_rows.len());
    println!("achievements: {}", achievements.len());
    println!("archive-candidates: {}", archive_candidates.len());
    println!("repeating: {}", repeating_rows.len());
    render_tag_counts(&filtered);
    render_report_section("Archive Candidates", &archive_candidates, limit, context);
    render_report_section("Achievements", &achievements, limit, context);
    render_report_section("Repeating Tasks", &repeating_rows, limit, context);
    Ok(CommandOutcome::success())
}

pub(crate) fn run_task_archive(
    args: &OrgizeTaskArchiveArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let refreshed = refresh_agent_org_read_model(&args.paths, context)?;
    let connection = open_read_model_connection(&refreshed.settings)?;
    let rows = query_agent_org_task_rows(&connection)?;
    let candidates = filter_archive_rows(&rows, args);
    let selected = candidates
        .iter()
        .take(args.limit)
        .copied()
        .collect::<Vec<_>>();

    println!("orgize agent task-archive");
    println!("backend: duckdb");
    println!("mode: {}", if args.apply { "apply" } else { "plan" });
    println!("database: {}", refreshed.settings.database_path.display());
    println!("table: {AGENT_ORG_TASKS_TABLE}");
    println!("sources: {}", refreshed.source_paths.len());
    println!("candidates: {}", candidates.len());
    println!("selected: {}", selected.len());
    for (index, row) in selected.iter().enumerate() {
        render_archive_plan_row(index + 1, row, &refreshed.settings, context);
    }

    if args.apply {
        apply_archive_plan(&selected, &refreshed.settings, context)?;
        println!("applied: {}", selected.len());
    }

    Ok(CommandOutcome::success())
}

fn refresh_agent_org_read_model(
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

#[derive(Debug, Clone)]
struct ResolvedReadModelSettings {
    cache_home: PathBuf,
    database_path: PathBuf,
    temp_directory: PathBuf,
    threads: u64,
    memory_limit: Option<String>,
    max_temp_directory_size: Option<String>,
    materialize_threshold_rows: u64,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct WendaoTomlConfig {
    #[serde(default)]
    agent: AgentTomlConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct AgentTomlConfig {
    #[serde(default)]
    org_read_model: AgentOrgReadModelTomlConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct AgentOrgReadModelTomlConfig {
    database_path: Option<String>,
    temp_directory: Option<String>,
    threads: Option<u64>,
    memory_limit: Option<String>,
    max_temp_directory_size: Option<String>,
    materialize_threshold_rows: Option<u64>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct AgentOrgReadModelMaterializationReport {
    rows: usize,
    active_rows: usize,
    done_rows: usize,
    archived_rows: usize,
}

#[derive(Debug, Clone)]
struct RefreshedAgentOrgReadModel {
    settings: ResolvedReadModelSettings,
    source_paths: Vec<PathBuf>,
    materialized: AgentOrgReadModelMaterializationReport,
}

#[derive(Debug, Clone)]
struct AgentOrgTaskListRow {
    source_path: String,
    source_line: u64,
    source_range_start: u64,
    source_range_end: u64,
    title: String,
    todo_state: Option<String>,
    is_done: bool,
    archived: bool,
    tags: Vec<String>,
    effective_tags: Vec<String>,
    scheduled: Option<String>,
    scheduled_repeater: Option<OrgizeAgentTaskRepeater>,
    deadline: Option<String>,
    deadline_repeater: Option<OrgizeAgentTaskRepeater>,
    closed: Option<String>,
    outline_path: Vec<String>,
    properties: Vec<OrgizeAgentTaskProperty>,
}

fn resolve_read_model_settings(context: &ClientContext) -> Result<ResolvedReadModelSettings> {
    let cache_home = resolve_cache_home(Some(context.root()))
        .with_context(|| "failed to resolve PRJ_CACHE_HOME for agent read model")?;
    let default_database_path = cache_home
        .join("agent")
        .join("readmodels")
        .join("org_agent_tasks.duckdb");
    let default_temp_directory = cache_home
        .join("agent")
        .join("readmodels")
        .join("duckdb-tmp");
    let default_threads =
        std::thread::available_parallelism().map_or(1, |count| count.get() as u64);

    let toml_config = load_agent_org_read_model_config(context)?;
    let database_path = toml_config
        .database_path
        .as_deref()
        .map_or(default_database_path, |value| {
            resolve_config_path_value(value, context.root(), cache_home.as_path())
        });
    let temp_directory = toml_config
        .temp_directory
        .as_deref()
        .map_or(default_temp_directory, |value| {
            resolve_config_path_value(value, context.root(), cache_home.as_path())
        });

    Ok(ResolvedReadModelSettings {
        cache_home,
        database_path,
        temp_directory,
        threads: toml_config.threads.unwrap_or(default_threads).max(1),
        memory_limit: normalized_optional_string(toml_config.memory_limit),
        max_temp_directory_size: normalized_optional_string(toml_config.max_temp_directory_size),
        materialize_threshold_rows: toml_config.materialize_threshold_rows.unwrap_or(1),
    })
}

fn load_agent_org_read_model_config(
    context: &ClientContext,
) -> Result<AgentOrgReadModelTomlConfig> {
    let Some(config_path) = resolve_config_path(context)? else {
        return Ok(AgentOrgReadModelTomlConfig::default());
    };
    let config_home = resolve_config_home(Some(context.root()));
    let merged = load_toml_value_with_imports_and_paths(
        config_path.as_path(),
        Some(context.root()),
        config_home.as_deref(),
    )
    .with_context(|| {
        format!(
            "failed to load agent read-model config from `{}`",
            config_path.display()
        )
    })?;
    let parsed: WendaoTomlConfig = merged.try_into().with_context(|| {
        format!(
            "failed to parse agent read-model config from `{}`",
            config_path.display()
        )
    })?;
    Ok(parsed.agent.org_read_model)
}

fn resolve_config_path(context: &ClientContext) -> Result<Option<PathBuf>> {
    if let Some(config_path) = context.config_file() {
        let config_path = config_path.to_path_buf();
        if !config_path.is_file() {
            anyhow::bail!(
                "configured agent read-model config `{}` does not exist or is not a file",
                config_path.display()
            );
        }
        return Ok(Some(config_path));
    }

    let default_path = context.root().join("wendao.toml");
    if default_path.is_file() {
        Ok(Some(default_path))
    } else {
        Ok(None)
    }
}

fn resolve_source_paths(
    paths: &[PathBuf],
    context: &ClientContext,
    cache_home: &Path,
) -> Vec<PathBuf> {
    if paths.is_empty() {
        return vec![cache_home.join("agent").join("org")];
    }
    paths
        .iter()
        .map(|path| {
            if path.is_absolute() {
                path.clone()
            } else {
                context.root().join(path)
            }
        })
        .collect()
}

fn resolve_config_path_value(value: &str, project_root: &Path, cache_home: &Path) -> PathBuf {
    let expanded = expand_project_path_variables(value, project_root, cache_home);
    resolve_path_from_value(Some(project_root), Some(expanded.as_str()))
        .unwrap_or_else(|| project_root.to_path_buf())
}

fn expand_project_path_variables(value: &str, project_root: &Path, cache_home: &Path) -> String {
    let trimmed = value.trim();
    let mut expanded = trimmed.to_string();
    let replacements = [
        ("${PRJ_CACHE_HOME}", cache_home),
        ("$PRJ_CACHE_HOME", cache_home),
        ("${PRJ_ROOT}", project_root),
        ("$PRJ_ROOT", project_root),
    ];
    for (token, path) in replacements {
        let path = path.to_string_lossy();
        if expanded == token {
            expanded = path.into_owned();
        } else if let Some(rest) = expanded.strip_prefix(&format!("{token}/")) {
            expanded = format!("{path}/{rest}");
        }
    }
    expanded
}

fn normalized_optional_string(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
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

fn open_read_model_connection(
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

fn query_agent_org_task_rows(
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

fn filter_task_rows<'a>(
    rows: &'a [AgentOrgTaskListRow],
    args: &OrgizeTaskListArgs,
) -> Vec<&'a AgentOrgTaskListRow> {
    let text = args.text.as_ref().map(|text| text.to_lowercase());
    let tags = args
        .tags
        .iter()
        .map(|tag| normalize_tag_filter(tag))
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();
    rows.iter()
        .filter(|row| args.include_done || !row.is_done)
        .filter(|row| args.include_archived || !row.archived)
        .filter(|row| tags.iter().all(|tag| task_row_has_tag(row, tag)))
        .filter(|row| {
            text.as_ref()
                .is_none_or(|text| task_row_matches_text(row, text))
        })
        .collect()
}

fn filter_report_rows<'a>(
    rows: &'a [AgentOrgTaskListRow],
    args: &OrgizeTaskReportArgs,
) -> Vec<&'a AgentOrgTaskListRow> {
    let text = args.text.as_ref().map(|text| text.to_lowercase());
    let tags = args
        .tags
        .iter()
        .map(|tag| normalize_tag_filter(tag))
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();
    rows.iter()
        .filter(|row| args.include_archived || !row.archived)
        .filter(|row| tags.iter().all(|tag| task_row_has_tag(row, tag)))
        .filter(|row| {
            text.as_ref()
                .is_none_or(|text| task_row_matches_text(row, text))
        })
        .collect()
}

fn filter_archive_rows<'a>(
    rows: &'a [AgentOrgTaskListRow],
    args: &OrgizeTaskArchiveArgs,
) -> Vec<&'a AgentOrgTaskListRow> {
    let text = args.text.as_ref().map(|text| text.to_lowercase());
    let tags = args
        .tags
        .iter()
        .map(|tag| normalize_tag_filter(tag))
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();
    rows.iter()
        .filter(|row| row.is_done && !row.archived)
        .filter(|row| tags.iter().all(|tag| task_row_has_tag(row, tag)))
        .filter(|row| {
            text.as_ref()
                .is_none_or(|text| task_row_matches_text(row, text))
        })
        .collect()
}

fn task_row_matches_text(row: &AgentOrgTaskListRow, text: &str) -> bool {
    let property_match = row.properties.iter().any(|property| {
        property.key.to_lowercase().contains(text) || property.value.to_lowercase().contains(text)
    });
    let repeater_match = row
        .scheduled_repeater
        .iter()
        .chain(row.deadline_repeater.iter())
        .any(|repeater| repeater_matches_text(repeater, text));
    row.title.to_lowercase().contains(text)
        || row.source_path.to_lowercase().contains(text)
        || row.outline_path.join(" / ").to_lowercase().contains(text)
        || row.tags.iter().any(|tag| tag.to_lowercase().contains(text))
        || row
            .effective_tags
            .iter()
            .any(|tag| tag.to_lowercase().contains(text))
        || property_match
        || repeater_match
}

fn normalize_tag_filter(tag: &str) -> String {
    tag.trim().trim_matches(':').to_lowercase()
}

fn task_row_has_tag(row: &AgentOrgTaskListRow, tag: &str) -> bool {
    row.tags
        .iter()
        .chain(row.effective_tags.iter())
        .any(|candidate| candidate.to_lowercase() == tag)
}

fn repeater_matches_text(repeater: &OrgizeAgentTaskRepeater, text: &str) -> bool {
    repeater.kind.to_lowercase().contains(text)
        || repeater.unit.to_lowercase().contains(text)
        || repeater.cookie.to_lowercase().contains(text)
}

fn render_task_list_row(index: usize, row: &AgentOrgTaskListRow, context: &ClientContext) {
    println!();
    println!("[TASK{index:03}] {}", row.title);
    if let Some(todo_state) = row.todo_state.as_deref() {
        println!("state: {todo_state}");
    }
    if !row.effective_tags.is_empty() {
        println!("tags: {}", row.effective_tags.join(":"));
    }
    println!(
        "source: {}:{}",
        display_source_path(&row.source_path, context),
        row.source_line
    );
    if let Some(scheduled) = row.scheduled.as_deref() {
        println!("scheduled: {scheduled}");
    }
    if let Some(deadline) = row.deadline.as_deref() {
        println!("deadline: {deadline}");
    }
    let repeaters = task_repeater_labels(row);
    if !repeaters.is_empty() {
        println!("repeat: {}", repeaters.join(", "));
    }
    if let Some(closed) = row.closed.as_deref() {
        println!("closed: {closed}");
    }
    if let Some(next_action) = property_value(row, "NEXT_ACTION") {
        println!("next: {next_action}");
    }
    if let Some(resume_query) = property_value(row, "RESUME_QUERY") {
        println!("resume: {resume_query}");
    }
}

fn render_tag_counts(rows: &[&AgentOrgTaskListRow]) {
    let mut counts = BTreeMap::<String, usize>::new();
    for row in rows {
        for tag in &row.effective_tags {
            *counts.entry(tag.clone()).or_default() += 1;
        }
    }
    if counts.is_empty() {
        return;
    }

    println!();
    println!("Tags");
    for (tag, count) in counts {
        println!("{tag}: {count}");
    }
}

fn render_report_section(
    title: &str,
    rows: &[&AgentOrgTaskListRow],
    limit: usize,
    context: &ClientContext,
) {
    println!();
    println!("{title}: {}", rows.len());
    for (index, row) in rows.iter().take(limit).enumerate() {
        render_task_list_row(index + 1, row, context);
    }
}

fn render_archive_plan_row(
    index: usize,
    row: &AgentOrgTaskListRow,
    settings: &ResolvedReadModelSettings,
    context: &ClientContext,
) {
    println!();
    println!("[ARCHIVE{index:03}] {}", row.title);
    println!(
        "source: {}:{}",
        display_source_path(&row.source_path, context),
        row.source_line
    );
    println!(
        "range: {}..{}",
        row.source_range_start, row.source_range_end
    );
    println!(
        "target: {}",
        display_source_path(
            archive_target_for_row(row, settings, context)
                .to_string_lossy()
                .as_ref(),
            context
        )
    );
    if !row.effective_tags.is_empty() {
        println!("tags: {}", row.effective_tags.join(":"));
    }
}

fn apply_archive_plan(
    rows: &[&AgentOrgTaskListRow],
    settings: &ResolvedReadModelSettings,
    context: &ClientContext,
) -> Result<()> {
    let mut rows_by_source = BTreeMap::<PathBuf, Vec<&AgentOrgTaskListRow>>::new();
    for row in rows {
        rows_by_source
            .entry(PathBuf::from(&row.source_path))
            .or_default()
            .push(*row);
    }

    let mut appends = BTreeMap::<PathBuf, Vec<String>>::new();
    for (source_path, mut source_rows) in rows_by_source {
        source_rows.sort_by_key(|row| std::cmp::Reverse(row.source_range_start));
        let original = fs::read_to_string(&source_path)
            .with_context(|| format!("failed to read `{}`", source_path.display()))?;
        let mut updated = original.clone();
        for row in source_rows {
            let start = usize::try_from(row.source_range_start)
                .with_context(|| "archive source range start overflowed usize")?;
            let end = usize::try_from(row.source_range_end)
                .with_context(|| "archive source range end overflowed usize")?;
            if start >= end || end > original.len() {
                anyhow::bail!(
                    "invalid archive range {}..{} for `{}`",
                    start,
                    end,
                    source_path.display()
                );
            }
            let target = archive_target_for_row(row, settings, context);
            if target == source_path {
                anyhow::bail!(
                    "archive target for `{}` resolves to the source file",
                    source_path.display()
                );
            }
            let subtree = original[start..end].trim_end_matches('\n').to_string();
            appends
                .entry(target)
                .or_default()
                .push(mark_subtree_archived(&subtree));
            updated.replace_range(start..end, "");
        }
        fs::write(&source_path, updated)
            .with_context(|| format!("failed to write `{}`", source_path.display()))?;
    }

    for (target, subtrees) in appends {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create archive directory `{}`", parent.display())
            })?;
        }
        let mut archive_content = if target.is_file() {
            fs::read_to_string(&target)
                .with_context(|| format!("failed to read `{}`", target.display()))?
        } else {
            archive_file_header()
        };
        if !archive_content.ends_with('\n') {
            archive_content.push('\n');
        }
        for subtree in subtrees {
            archive_content.push('\n');
            archive_content.push_str(subtree.trim_end_matches('\n'));
            archive_content.push('\n');
        }
        fs::write(&target, archive_content)
            .with_context(|| format!("failed to write `{}`", target.display()))?;
    }

    Ok(())
}

fn archive_target_for_row(
    row: &AgentOrgTaskListRow,
    settings: &ResolvedReadModelSettings,
    context: &ClientContext,
) -> PathBuf {
    property_value(row, "ARCHIVE_TARGET").map_or_else(
        || {
            settings
                .cache_home
                .join("agent")
                .join("org")
                .join("archives")
                .join("agent_tasks.org")
        },
        |value| resolve_config_path_value(value, context.root(), settings.cache_home.as_path()),
    )
}

fn archive_file_header() -> String {
    concat!(
        "#+TITLE: Agent Org Archive\n",
        "#+AUTHOR: CyberXiuXian Artisan workshop\n",
        "#+FILETAGS: :ARCHIVE:\n"
    )
    .to_string()
}

fn mark_subtree_archived(subtree: &str) -> String {
    let Some((heading, rest)) = subtree.split_once('\n') else {
        return mark_heading_archived(subtree);
    };
    format!("{}\n{}", mark_heading_archived(heading), rest)
}

fn mark_heading_archived(heading: &str) -> String {
    let trimmed = heading.trim_end();
    if trimmed.contains(":ARCHIVE:") {
        return trimmed.to_string();
    }
    if trimmed.ends_with(':') && trimmed.rfind(" :").is_some() {
        let without_final_colon = &trimmed[..trimmed.len() - 1];
        format!("{without_final_colon}:ARCHIVE:")
    } else {
        format!("{trimmed} :ARCHIVE:")
    }
}

fn task_repeater_labels(row: &AgentOrgTaskListRow) -> Vec<String> {
    let mut labels = Vec::new();
    if let Some(repeater) = &row.scheduled_repeater {
        labels.push(format!("scheduled {} ({})", repeater.cookie, repeater.kind));
    }
    if let Some(repeater) = &row.deadline_repeater {
        labels.push(format!("deadline {} ({})", repeater.cookie, repeater.kind));
    }
    labels
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

fn display_source_path(source_path: &str, context: &ClientContext) -> String {
    let path = Path::new(source_path);
    path.strip_prefix(context.root()).map_or_else(
        |_| source_path.to_string(),
        |path| path.display().to_string(),
    )
}

fn property_value<'a>(row: &'a AgentOrgTaskListRow, key: &str) -> Option<&'a str> {
    row.properties
        .iter()
        .find(|property| property.key == key)
        .map(|property| property.value.as_str())
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
