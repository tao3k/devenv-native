//! Synthetic performance fixtures for Org agent read-model flows.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};

use crate::{ClientContext, OutputFormat};

use super::settings::{resolve_read_model_settings, resolve_source_paths};
use super::store::{
    open_read_model_read_only_connection, query_active_agent_org_task_row_window,
    query_agent_org_task_rows_matching, refresh_agent_org_read_model,
};

/// Number of Org task rows in the default agent read-model benchmark fixture.
pub const ORGIZE_AGENT_BENCH_TASK_COUNT: usize = 1_024;

/// Number of source Org files in the default agent read-model benchmark fixture.
pub const ORGIZE_AGENT_BENCH_FILE_COUNT: usize = 16;

const TASKS_PER_FILE: usize = ORGIZE_AGENT_BENCH_TASK_COUNT / ORGIZE_AGENT_BENCH_FILE_COUNT;

/// Summary returned by the agent Org read-model benchmark path.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct OrgizeAgentReadModelBenchmarkSummary {
    /// Number of source files resolved for the refresh.
    pub source_count: usize,
    /// Number of rows materialized into `DuckDB`.
    pub materialized_rows: usize,
    /// Number of active rows reported by materialization.
    pub active_rows: usize,
    /// Number of done rows reported by materialization.
    pub done_rows: usize,
    /// Number of archived rows reported by materialization.
    pub archived_rows: usize,
    /// Number of rows read back from the cached `DuckDB` snapshot.
    pub cached_rows: usize,
}

/// Summary returned by the cached active recovery query benchmark path.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct OrgizeAgentCachedActiveBenchmarkSummary {
    /// Number of active rows matching the source scope.
    pub active_rows: usize,
    /// Number of rows decoded for the requested recovery window.
    pub shown_rows: usize,
}

/// Write a deterministic Org agent task corpus for read-model benchmarks.
///
/// # Panics
///
/// Panics when fixture directories or files cannot be written.
#[must_use]
pub fn write_agent_org_benchmark_fixture(root: &Path) -> PathBuf {
    let org_dir = root.join("agent-org");
    fs::create_dir_all(org_dir.as_path())
        .unwrap_or_else(|error| panic!("create agent Org benchmark directory: {error}"));

    for file_index in 0..ORGIZE_AGENT_BENCH_FILE_COUNT {
        write_agent_org_benchmark_file(org_dir.as_path(), file_index);
    }

    org_dir
}

fn write_agent_org_benchmark_file(org_dir: &Path, file_index: usize) {
    let mut body = String::from("#+title: Agent benchmark fixture\n\n");
    for local_index in 0..TASKS_PER_FILE {
        body.push_str(agent_org_benchmark_task(file_index, local_index).as_str());
    }
    fs::write(org_dir.join(format!("agenda_{file_index:02}.org")), body)
        .unwrap_or_else(|error| panic!("write agent Org benchmark file: {error}"));
}

fn agent_org_benchmark_task(file_index: usize, local_index: usize) -> String {
    let index = file_index * TASKS_PER_FILE + local_index;
    let state = agent_org_benchmark_task_state(index);
    let closed = agent_org_benchmark_closed_line(index, state);
    let schedule = agent_org_benchmark_schedule_line(index);
    let archive_tag = agent_org_benchmark_archive_tag(index);
    format!(
        "* {state} Agent benchmark task {index:04} :agent:performance:{archive_tag}\n\
         {schedule}{closed}\
         :PROPERTIES:\n\
         :NEXT_ACTION: Continue benchmark task {index:04}\n\
         :RESUME_QUERY: wendao-client orgize task-list --text 'task {index:04}'\n\
         :END:\n\
         - [X] Parse source\n\
         - [ ] Refresh read model\n\n",
    )
}

fn agent_org_benchmark_task_state(index: usize) -> &'static str {
    if index.is_multiple_of(5) {
        "DONE"
    } else {
        "TODO"
    }
}

fn agent_org_benchmark_closed_line(index: usize, state: &str) -> String {
    if state == "DONE" {
        format!("CLOSED: [2026-05-{:02} Mon]\n", 1 + (index % 18))
    } else {
        String::new()
    }
}

fn agent_org_benchmark_schedule_line(index: usize) -> &'static str {
    if index.is_multiple_of(17) {
        "SCHEDULED: <2026-05-18 Mon ++1d>\n"
    } else {
        ""
    }
}

fn agent_org_benchmark_archive_tag(index: usize) -> &'static str {
    if index.is_multiple_of(29) {
        ":ARCHIVE:"
    } else {
        ""
    }
}

/// Refresh and query the agent Org read model for the benchmark fixture.
///
/// # Errors
///
/// Returns an error when source resolution, `DuckDB` refresh, or cached query fails.
pub fn benchmark_agent_org_read_model(root: &Path) -> Result<OrgizeAgentReadModelBenchmarkSummary> {
    let context = ClientContext::new(root, OutputFormat::Text);
    let org_dir = root.join("agent-org");
    let paths = vec![org_dir];
    let settings = resolve_read_model_settings(&context)?;
    let source_paths = resolve_source_paths(&paths, &context, settings.cache_home.as_path());
    let refreshed = refresh_agent_org_read_model(&paths, &context)?;
    let connection = open_read_model_read_only_connection(&refreshed.settings)?
        .context("agent Org read-model benchmark expected a cached DuckDB snapshot")?;
    let cached_rows =
        query_agent_org_task_rows_matching(&connection, &source_paths, None, &[])?.len();

    Ok(OrgizeAgentReadModelBenchmarkSummary {
        source_count: source_paths.len(),
        materialized_rows: refreshed.materialized.rows,
        active_rows: refreshed.materialized.active_rows,
        done_rows: refreshed.materialized.done_rows,
        archived_rows: refreshed.materialized.archived_rows,
        cached_rows,
    })
}

/// Query the cached active recovery window for the benchmark fixture.
///
/// # Errors
///
/// Returns an error when the cached `DuckDB` snapshot is unavailable or cannot
/// be queried.
pub fn benchmark_agent_org_cached_active_query(
    root: &Path,
    limit: usize,
) -> Result<OrgizeAgentCachedActiveBenchmarkSummary> {
    let context = ClientContext::new(root, OutputFormat::Text);
    let paths = vec![root.join("agent-org")];
    let settings = resolve_read_model_settings(&context)?;
    let source_paths = resolve_source_paths(&paths, &context, settings.cache_home.as_path());
    let connection = open_read_model_read_only_connection(&settings)?
        .context("agent Org cached-active benchmark expected a cached DuckDB snapshot")?;
    let window = query_active_agent_org_task_row_window(&connection, &source_paths, limit)?;
    Ok(OrgizeAgentCachedActiveBenchmarkSummary {
        active_rows: window.total_rows,
        shown_rows: window.rows.len(),
    })
}
