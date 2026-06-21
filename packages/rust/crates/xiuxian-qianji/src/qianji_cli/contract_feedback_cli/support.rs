use std::path::{Path, PathBuf};

use crate::executors::QianjiAdvisoryAuditExecutor;
use crate::sovereign::KnowledgeStorageContractFeedbackSink;
use xiuxian_db_store::state::{
    ProjectCacheRootConfig, STATE_STORE_DIR_NAME, project_cache_root_from_config,
};

use super::types::RestDocsCliCommand;
use crate::qianji_cli::input::resolve_path_against_root;

pub(super) fn build_scaffold_advisory_executor() -> QianjiAdvisoryAuditExecutor {
    QianjiAdvisoryAuditExecutor::new()
}

pub(super) fn build_contract_feedback_sink(
    command: &RestDocsCliCommand,
    workspace_root: &Path,
) -> KnowledgeStorageContractFeedbackSink {
    let storage_path = command
        .storage_path
        .clone()
        .unwrap_or_else(|| default_contract_feedback_storage_path(workspace_root));
    let storage_path = resolve_path_against_root(storage_path, workspace_root);

    KnowledgeStorageContractFeedbackSink::new(
        storage_path.display().to_string(),
        command.table_name.clone(),
    )
}

pub(super) fn build_contract_feedback_session_id(openapi_path: &Path) -> String {
    let raw = openapi_path.to_string_lossy();
    let mut normalized = String::with_capacity(raw.len());
    for character in raw.chars() {
        if character.is_ascii_alphanumeric() {
            normalized.push(character.to_ascii_lowercase());
        } else {
            normalized.push('_');
        }
    }
    format!("contract-feedback:rest-docs:{normalized}")
}

fn default_contract_feedback_storage_path(workspace_root: &Path) -> PathBuf {
    project_cache_root_from_config(ProjectCacheRootConfig {
        project_root: Some(workspace_root.to_path_buf()),
        cache_home: None,
        project_namespace: None,
    })
    .join(STATE_STORE_DIR_NAME)
    .join("xiuxian-qianji")
    .join("contract_feedback")
}
