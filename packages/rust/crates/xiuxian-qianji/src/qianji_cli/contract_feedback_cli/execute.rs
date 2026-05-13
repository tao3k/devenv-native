use std::path::Path;

use crate::contract_feedback::{CollectionContext, ContractRunConfig, NoopAdvisoryAuditExecutor};
use crate::contract_feedback::{
    build_rest_docs_collection_context, persist_contract_feedback_run,
    run_and_persist_rest_docs_contract_feedback,
    run_and_persist_rest_docs_contract_feedback_with_live_advisory,
    run_rest_docs_contract_feedback, run_rest_docs_contract_feedback_with_live_advisory,
};

use super::config::build_contract_feedback_config;
use super::output::{
    build_contract_feedback_output, build_persisted_contract_feedback_output,
    print_contract_feedback_output, storage_output_from_sink,
};
use super::support::{
    build_contract_feedback_session_id, build_contract_feedback_sink,
    build_live_contract_feedback_options, build_live_contract_feedback_runtime,
    build_scaffold_advisory_executor,
};
use super::types::{
    ContractFeedbackCliCommand, ContractFeedbackCliOutput, REST_DOCS_PACK_ID, RestDocsCliCommand,
};
use crate::qianji_cli::input::resolve_cli_path;
use crate::qianji_cli::workspace::resolve_workspace_root;

pub(crate) async fn handle_contract_feedback_command(
    command: ContractFeedbackCliCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ContractFeedbackCliCommand::RestDocs(command) => {
            handle_rest_docs_contract_feedback(command).await
        }
    }
}

pub(crate) async fn run_deterministic_rest_docs_contract_feedback(
    command: &RestDocsCliCommand,
    openapi_path: &Path,
    workspace_root: &Path,
    collection_context: CollectionContext,
    config: &ContractRunConfig,
    advisory_roles: Vec<String>,
) -> Result<ContractFeedbackCliOutput, Box<dyn std::error::Error>> {
    if command.no_persist {
        let run = run_rest_docs_contract_feedback(
            openapi_path,
            collection_context,
            config,
            &NoopAdvisoryAuditExecutor,
        )
        .await?;
        return Ok(build_contract_feedback_output(
            openapi_path.to_path_buf(),
            workspace_root.to_path_buf(),
            false,
            advisory_roles,
            run,
            Vec::new(),
            None,
        ));
    }

    let sink = build_contract_feedback_sink(command, workspace_root);
    let persisted = run_and_persist_rest_docs_contract_feedback(
        openapi_path,
        collection_context,
        config,
        &sink,
    )
    .await?;

    Ok(build_persisted_contract_feedback_output(
        openapi_path.to_path_buf(),
        workspace_root.to_path_buf(),
        false,
        advisory_roles,
        persisted,
        storage_output_from_sink(&sink),
    ))
}

pub(crate) async fn run_scaffold_rest_docs_contract_feedback(
    command: &RestDocsCliCommand,
    openapi_path: &Path,
    workspace_root: &Path,
    collection_context: CollectionContext,
    config: &ContractRunConfig,
    advisory_roles: Vec<String>,
) -> Result<ContractFeedbackCliOutput, Box<dyn std::error::Error>> {
    let executor = build_scaffold_advisory_executor();

    if command.no_persist {
        let run =
            run_rest_docs_contract_feedback(openapi_path, collection_context, config, &executor)
                .await?;
        return Ok(build_contract_feedback_output(
            openapi_path.to_path_buf(),
            workspace_root.to_path_buf(),
            false,
            advisory_roles,
            run,
            Vec::new(),
            None,
        ));
    }

    let sink = build_contract_feedback_sink(command, workspace_root);
    let run = run_rest_docs_contract_feedback(openapi_path, collection_context, config, &executor)
        .await?;
    let persisted = persist_contract_feedback_run(run, &sink).await?;

    Ok(build_persisted_contract_feedback_output(
        openapi_path.to_path_buf(),
        workspace_root.to_path_buf(),
        false,
        advisory_roles,
        persisted,
        storage_output_from_sink(&sink),
    ))
}

async fn handle_rest_docs_contract_feedback(
    command: RestDocsCliCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    let openapi_path = resolve_cli_path(command.openapi_path.as_path())?;
    let workspace_root = resolve_workspace_root(command.workspace_root.as_deref())?;
    let mut collection_context =
        build_rest_docs_collection_context(&openapi_path, Some(workspace_root.clone()));
    collection_context.labels.insert(
        "invocation".to_string(),
        "qianji_contract_feedback_rest_docs".to_string(),
    );
    collection_context.labels.insert(
        "session_id".to_string(),
        build_contract_feedback_session_id(&openapi_path),
    );
    if let Some(model) = command.model.as_ref() {
        collection_context
            .labels
            .insert("llm_model".to_string(), model.clone());
    }

    let config = build_contract_feedback_config(&command);
    let advisory_roles = config
        .advisory_policy_for_pack(REST_DOCS_PACK_ID)
        .requested_roles;

    let output = if command.live_advisory {
        run_live_rest_docs_contract_feedback(
            &command,
            &openapi_path,
            &workspace_root,
            collection_context,
            &config,
            advisory_roles,
        )
        .await?
    } else if advisory_roles.is_empty() {
        run_deterministic_rest_docs_contract_feedback(
            &command,
            &openapi_path,
            &workspace_root,
            collection_context,
            &config,
            advisory_roles,
        )
        .await?
    } else {
        run_scaffold_rest_docs_contract_feedback(
            &command,
            &openapi_path,
            &workspace_root,
            collection_context,
            &config,
            advisory_roles,
        )
        .await?
    };

    print_contract_feedback_output(&output)
}

async fn run_live_rest_docs_contract_feedback(
    command: &RestDocsCliCommand,
    openapi_path: &Path,
    workspace_root: &Path,
    collection_context: CollectionContext,
    config: &ContractRunConfig,
    advisory_roles: Vec<String>,
) -> Result<ContractFeedbackCliOutput, Box<dyn std::error::Error>> {
    let runtime = build_live_contract_feedback_runtime()?;
    let options = build_live_contract_feedback_options(command)?;

    if command.no_persist {
        let run = run_rest_docs_contract_feedback_with_live_advisory(
            openapi_path,
            collection_context,
            config,
            runtime,
            options,
        )
        .await?;
        return Ok(build_contract_feedback_output(
            openapi_path.to_path_buf(),
            workspace_root.to_path_buf(),
            true,
            advisory_roles,
            run,
            Vec::new(),
            None,
        ));
    }

    let sink = build_contract_feedback_sink(command, workspace_root);
    let persisted = run_and_persist_rest_docs_contract_feedback_with_live_advisory(
        openapi_path,
        collection_context,
        config,
        runtime,
        options,
        &sink,
    )
    .await?;

    Ok(build_persisted_contract_feedback_output(
        openapi_path.to_path_buf(),
        workspace_root.to_path_buf(),
        true,
        advisory_roles,
        persisted,
        storage_output_from_sink(&sink),
    ))
}
