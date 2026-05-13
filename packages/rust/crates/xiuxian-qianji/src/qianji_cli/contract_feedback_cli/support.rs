use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::contract_feedback::{
    QianjiLiveContractFeedbackOptions, QianjiLiveContractFeedbackRuntime,
};
use crate::executors::QianjiAdvisoryAuditExecutor;
use crate::runtime_config::resolve_qianji_runtime_llm_config;
use crate::sovereign::KnowledgeStorageContractFeedbackSink;
use xiuxian_config_core::resolve_cache_home;
use xiuxian_llm::llm::{LlmClient, OpenAICompatibleClient, OpenAIWireApi};
use xiuxian_qianhuan::{orchestrator::ThousandFacesOrchestrator, persona::PersonaRegistry};

use super::types::RestDocsCliCommand;
use crate::qianji_cli::input::resolve_path_against_root;

pub(crate) fn sanitize_prj_cache_home(workspace_root: &Path, resolved: PathBuf) -> PathBuf {
    if resolved.is_absolute() && !resolved.starts_with(workspace_root) {
        workspace_root.join(".cache")
    } else {
        resolved
    }
}

pub(super) fn build_scaffold_advisory_executor() -> QianjiAdvisoryAuditExecutor {
    let (orchestrator, registry) = build_contract_feedback_role_runtime();
    QianjiAdvisoryAuditExecutor::new(orchestrator, registry)
}

pub(super) fn build_live_contract_feedback_runtime()
-> Result<QianjiLiveContractFeedbackRuntime, Box<dyn std::error::Error>> {
    let llm_runtime = resolve_qianji_runtime_llm_config()?;
    let (orchestrator, registry) = build_contract_feedback_role_runtime();
    let client: Arc<dyn LlmClient> = Arc::new(OpenAICompatibleClient {
        api_key: llm_runtime.api_key,
        base_url: llm_runtime.base_url,
        wire_api: OpenAIWireApi::parse(Some(llm_runtime.wire_api.as_str())),
        http: reqwest::Client::new(),
    });

    Ok(QianjiLiveContractFeedbackRuntime::new(
        orchestrator,
        registry,
        client,
    ))
}

pub(super) fn build_live_contract_feedback_options(
    command: &RestDocsCliCommand,
) -> Result<QianjiLiveContractFeedbackOptions, Box<dyn std::error::Error>> {
    let mut options = QianjiLiveContractFeedbackOptions::default();
    let resolved = resolve_qianji_runtime_llm_config()?;
    options.model = command.model.clone().unwrap_or(resolved.model);
    if let Some(temperature) = command.temperature {
        options.temperature = temperature;
    }
    options.cognitive_early_halt_threshold = command.cognitive_early_halt_threshold;
    Ok(options)
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
        resolve_cache_home(Some(workspace_root)).unwrap_or_else(|| workspace_root.join(".cache"));
    sanitize_prj_cache_home(workspace_root, resolved)
        .join("wendao")
        .join("contract_feedback")
}
