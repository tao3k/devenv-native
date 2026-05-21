//! Shared read-model constants and row types.

use std::path::PathBuf;

use xiuxian_wendao_parsers::{OrgizeAgentTaskProperty, OrgizeAgentTaskRepeater};

pub(super) const AGENT_ORG_TASKS_TABLE: &str = "agent_org_tasks";
pub(super) const AGENT_ORG_TASK_LIST_QUERY: &str = r"
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
    level,
    outline_path_json,
    properties_json
FROM agent_org_tasks
ORDER BY archived ASC, is_done ASC, source_path ASC, source_line ASC
";

#[derive(Debug, Clone)]
pub(super) struct ResolvedReadModelSettings {
    pub(super) cache_home: PathBuf,
    pub(super) database_path: PathBuf,
    pub(super) temp_directory: PathBuf,
    pub(super) threads: u64,
    pub(super) memory_limit: Option<String>,
    pub(super) max_temp_directory_size: Option<String>,
    pub(super) materialize_threshold_rows: u64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct AgentOrgReadModelMaterializationReport {
    pub(super) rows: usize,
    pub(super) active_rows: usize,
    pub(super) done_rows: usize,
    pub(super) archived_rows: usize,
}

#[derive(Debug, Clone)]
pub(super) struct RefreshedAgentOrgReadModel {
    pub(super) settings: ResolvedReadModelSettings,
    pub(super) source_paths: Vec<PathBuf>,
    pub(super) materialized: AgentOrgReadModelMaterializationReport,
}

#[derive(Debug, Clone)]
pub(super) struct AgentOrgTaskListRow {
    pub(super) source_path: String,
    pub(super) source_line: u64,
    pub(super) source_range_start: u64,
    pub(super) source_range_end: u64,
    pub(super) level: u64,
    pub(super) title: String,
    pub(super) todo_state: Option<String>,
    pub(super) is_done: bool,
    pub(super) archived: bool,
    pub(super) tags: Vec<String>,
    pub(super) effective_tags: Vec<String>,
    pub(super) scheduled: Option<String>,
    pub(super) scheduled_repeater: Option<OrgizeAgentTaskRepeater>,
    pub(super) deadline: Option<String>,
    pub(super) deadline_repeater: Option<OrgizeAgentTaskRepeater>,
    pub(super) closed: Option<String>,
    pub(super) outline_path: Vec<String>,
    pub(super) properties: Vec<OrgizeAgentTaskProperty>,
}
