//! Shared read-model constants and row types.

use std::path::PathBuf;

use xiuxian_wendao_parsers::{OrgizeAgentTaskProperty, OrgizeAgentTaskRepeater};

pub(super) const AGENT_ORG_TASKS_TABLE: &str = "agent_org_tasks";
pub(super) const AGENT_ORG_MEMORY_OBJECTS_TABLE: &str = "agent_org_memory_objects";
pub(super) const AGENT_ORG_ELEMENTS_TABLE: &str = "agent_org_elements";

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
    pub(super) memory_object_rows: usize,
    pub(super) org_element_rows: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct TaskQuerySnapshot {
    pub(super) settings: ResolvedReadModelSettings,
    pub(super) source_paths: Vec<PathBuf>,
    pub(super) materialized: Option<AgentOrgReadModelMaterializationReport>,
    pub(super) snapshot_label: &'static str,
    pub(super) refresh_warning: Option<String>,
    pub(super) rows: Vec<AgentOrgTaskListRow>,
}

#[derive(Debug, Clone)]
pub(super) struct RefreshedAgentOrgReadModel {
    pub(super) settings: ResolvedReadModelSettings,
    pub(super) source_paths: Vec<PathBuf>,
    pub(super) materialized: AgentOrgReadModelMaterializationReport,
}

#[derive(Debug, Clone)]
pub(super) struct AgentOrgTaskListRow {
    pub(super) orgid: String,
    pub(super) source_path: String,
    pub(super) source_modified_unix_ms: u64,
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
    pub(super) matched_org_elements: Vec<AgentOrgElementMatch>,
}

#[derive(Debug, Clone)]
pub(super) struct AgentOrgElementMatch {
    pub(super) ordinal: u64,
    pub(super) category: String,
    pub(super) kind: String,
    pub(super) affiliated_name: Option<String>,
    pub(super) context: String,
    pub(super) summary_json: String,
    pub(super) language: Option<String>,
    pub(super) source_start_line: u64,
    pub(super) source_range_start: u64,
    pub(super) source_range_end: u64,
    pub(super) source_raw: String,
}
