use std::path::PathBuf;

use crate::contract_feedback::{QianjiContractFeedbackRun, QianjiPersistedContractFeedbackRun};
use crate::sovereign::KnowledgeStorageContractFeedbackSink;

use super::types::{ContractFeedbackCliOutput, ContractFeedbackStorageOutput};

pub(super) fn print_contract_feedback_output(
    output: &ContractFeedbackCliOutput,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

pub(super) fn build_contract_feedback_output(
    openapi_path: PathBuf,
    workspace_root: PathBuf,
    live_advisory: bool,
    advisory_roles: Vec<String>,
    run: QianjiContractFeedbackRun,
    persisted_entry_ids: Vec<String>,
    storage: Option<ContractFeedbackStorageOutput>,
) -> ContractFeedbackCliOutput {
    let report = serde_json::to_value(run.report)
        .unwrap_or_else(|error| serde_json::json!({ "serialization_error": error.to_string() }));
    ContractFeedbackCliOutput {
        openapi_path,
        workspace_root,
        live_advisory,
        advisory_roles,
        report,
        knowledge_entry_ids: run
            .knowledge_entries
            .into_iter()
            .map(|entry| entry.id)
            .collect(),
        persisted_entry_ids,
        storage,
    }
}

pub(super) fn build_persisted_contract_feedback_output(
    openapi_path: impl Into<PathBuf>,
    workspace_root: impl Into<PathBuf>,
    live_advisory: bool,
    advisory_roles: Vec<String>,
    persisted: QianjiPersistedContractFeedbackRun,
    storage: ContractFeedbackStorageOutput,
) -> ContractFeedbackCliOutput {
    build_contract_feedback_output(
        openapi_path.into(),
        workspace_root.into(),
        live_advisory,
        advisory_roles,
        persisted.run,
        persisted.persisted_entry_ids,
        Some(storage),
    )
}

pub(super) fn storage_output_from_sink(
    sink: &KnowledgeStorageContractFeedbackSink,
) -> ContractFeedbackStorageOutput {
    ContractFeedbackStorageOutput {
        storage_path: sink.storage_path().to_string(),
        table_name: sink.table_name().to_string(),
    }
}
