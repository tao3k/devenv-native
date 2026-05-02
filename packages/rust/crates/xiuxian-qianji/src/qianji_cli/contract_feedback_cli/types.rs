use std::path::PathBuf;

use serde::Serialize;

pub(crate) const DEFAULT_CONTRACT_FEEDBACK_TABLE_NAME: &str = "contract_feedback";
pub(crate) const REST_DOCS_PACK_ID: &str = "rest_docs";

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ContractFeedbackCliCommand {
    RestDocs(RestDocsCliCommand),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RestDocsCliCommand {
    pub(crate) openapi_path: PathBuf,
    pub(crate) workspace_root: Option<PathBuf>,
    pub(crate) storage_path: Option<PathBuf>,
    pub(crate) table_name: String,
    pub(crate) no_persist: bool,
    pub(crate) live_advisory: bool,
    pub(crate) roles: Vec<String>,
    pub(crate) model: Option<String>,
    pub(crate) temperature: Option<f32>,
    pub(crate) cognitive_early_halt_threshold: Option<f32>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ContractFeedbackStorageOutput {
    pub(crate) storage_path: String,
    pub(crate) table_name: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ContractFeedbackCliOutput {
    pub(crate) openapi_path: PathBuf,
    pub(crate) workspace_root: PathBuf,
    pub(crate) live_advisory: bool,
    pub(crate) advisory_roles: Vec<String>,
    pub(crate) report: serde_json::Value,
    pub(crate) knowledge_entry_ids: Vec<String>,
    pub(crate) persisted_entry_ids: Vec<String>,
    pub(crate) storage: Option<ContractFeedbackStorageOutput>,
}
