use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::executors::QianjiAdvisoryAuditExecutor;
use crate::sovereign::KnowledgeStorageContractFeedbackSink;
use xiuxian_config_core::resolve_data_home;
use xiuxian_qianhuan::{orchestrator::ThousandFacesOrchestrator, persona::PersonaRegistry};

use super::types::RestDocsCliCommand;
use crate::qianji_cli::input::resolve_path_against_root;

pub(crate) fn normalize_prj_data_home(_workspace_root: &Path, resolved: PathBuf) -> PathBuf {
    resolved
}

pub(super) fn build_scaffold_advisory_executor() -> QianjiAdvisoryAuditExecutor {
    let (orchestrator, registry) = build_contract_feedback_role_runtime();
    QianjiAdvisoryAuditExecutor::new(orchestrator, registry)
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

fn build_contract_feedback_role_runtime() -> (Arc<ThousandFacesOrchestrator>, Arc<PersonaRegistry>)
{
    (
        Arc::new(ThousandFacesOrchestrator::new(
            "Contract Feedback".to_string(),
            None,
        )),
        Arc::new(PersonaRegistry::with_builtins()),
    )
}

fn default_contract_feedback_storage_path(workspace_root: &Path) -> PathBuf {
    let resolved =
        resolve_data_home(Some(workspace_root)).unwrap_or_else(|| workspace_root.join(".data"));
    normalize_prj_data_home(workspace_root, resolved)
        .join("xiuxian-qianji")
        .join("contract_feedback")
}
