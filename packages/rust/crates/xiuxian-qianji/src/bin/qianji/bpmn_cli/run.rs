use super::deps::{
    Arc, QianjiBpmnCheckpointStore, QianjiBpmnExecutionFacade, QianjiBpmnExecutionRequest,
    QianjiRuntimeEnv, SchedulerAgentIdentity, invalid_input, io, load_bpmn_package_from_files,
    resolve_cli_path, resolve_qianji_runtime_checkpoint_config,
    resolve_qianji_runtime_checkpoint_config_with_env, unix_millis_now,
};
use super::types::{
    BpmnCliCheckpointBackend, BpmnCliCommand, BpmnCliOutput, BpmnRunCliCommand,
    BpmnRunRenderContext,
};
use super::{host, render};

pub(crate) async fn handle_bpmn_command(
    command: BpmnCliCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = run_bpmn_command(command).await?;
    println!("{}", output.rendered);
    if output.exit_code == 0 {
        Ok(())
    } else {
        std::process::exit(output.exit_code);
    }
}

pub(crate) async fn run_bpmn_command(
    command: BpmnCliCommand,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    match command {
        BpmnCliCommand::Run(command) => run_bpmn_run_command(&command).await,
    }
}

async fn run_bpmn_run_command(
    command: &BpmnRunCliCommand,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    let scheduler_identity = SchedulerAgentIdentity::from_env();
    run_bpmn_run_command_with_runtime_env(command, None, Some(&scheduler_identity)).await
}

pub(crate) async fn run_bpmn_run_command_with_runtime_env(
    command: &BpmnRunCliCommand,
    runtime_env: Option<&QianjiRuntimeEnv>,
    scheduler_identity: Option<&SchedulerAgentIdentity>,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    let resolved_bpmn_path = resolve_cli_path(command.bpmn_path.as_path())?;
    let resolved_dmn_paths = command
        .dmn_paths
        .iter()
        .map(|path| resolve_cli_path(path.as_path()))
        .collect::<io::Result<Vec<_>>>()?;
    let package = load_bpmn_package_from_files(&resolved_bpmn_path, &resolved_dmn_paths)?;
    let checkpoint_store =
        resolve_bpmn_checkpoint_store_with_env(command.checkpoint_backend.as_ref(), runtime_env)?;
    let host_context = host::build_bpmn_cli_host_bridge(&package, command)?;
    let request = QianjiBpmnExecutionRequest::new(
        &command.process_id,
        &command.instance_id,
        parse_bpmn_cli_initial_variables(command.context_json.as_deref())?,
        unix_millis_now(),
    );
    let mut execution_facade =
        QianjiBpmnExecutionFacade::new(Arc::clone(&package), checkpoint_store.clone());
    if let Some(scheduler_identity) = scheduler_identity.cloned() {
        execution_facade = execution_facade.with_scheduler_identity(scheduler_identity);
    }
    let execution = execution_facade.run(&request, &host_context.host).await?;

    Ok(render::render_bpmn_run_output(
        command,
        &execution.session,
        &execution.outcome,
        &BpmnRunRenderContext {
            resolved_bpmn_path: resolved_bpmn_path.as_path(),
            resolved_dmn_paths: &resolved_dmn_paths,
            checkpoint_store: checkpoint_store.as_ref(),
            resolved_host_fixture_path: host_context.resolved_host_fixture_path.as_deref(),
            resolved_event_fixture_path: host_context.resolved_event_fixture_path.as_deref(),
            resumed_from_checkpoint: execution.resumed_from_checkpoint,
            checkpoint_saved: execution.checkpoint_saved,
            checkpoint_deleted: execution.checkpoint_deleted,
        },
    ))
}

pub(crate) fn resolve_bpmn_checkpoint_store_with_env(
    backend: Option<&BpmnCliCheckpointBackend>,
    runtime_env: Option<&QianjiRuntimeEnv>,
) -> Result<Option<QianjiBpmnCheckpointStore>, Box<dyn std::error::Error>> {
    match backend {
        None => Ok(None),
        Some(BpmnCliCheckpointBackend::RuntimeValkey) => {
            let runtime = match runtime_env {
                Some(runtime_env) => resolve_qianji_runtime_checkpoint_config_with_env(runtime_env),
                None => resolve_qianji_runtime_checkpoint_config(),
            }
            .map_err(|error| {
                io::Error::other(format!(
                    "failed to resolve Qianji checkpoint runtime config for `bpmn run`: {error}"
                ))
            })?;
            Ok(Some(
                QianjiBpmnCheckpointStore::from_runtime_checkpoint_config(&runtime),
            ))
        }
        #[cfg(feature = "sqlite")]
        Some(BpmnCliCheckpointBackend::Sqlite(path)) => Ok(Some(
            QianjiBpmnCheckpointStore::sqlite(resolve_cli_path(path.as_path())?),
        )),
    }
}

fn parse_bpmn_cli_initial_variables(
    raw_context: Option<&str>,
) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
    raw_context
        .map(|raw| {
            serde_json::from_str(raw).map_err(|error| {
                invalid_input(format!(
                    "failed to parse `--context-json` as valid JSON: {error}"
                ))
            })
        })
        .transpose()
        .map_err(Into::into)
}
