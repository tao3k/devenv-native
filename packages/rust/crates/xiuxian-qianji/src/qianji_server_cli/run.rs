use super::cli::{QianjiServerCommand, QianjiServerServeCommand, qianji_server_usage};
use super::flowhub::{
    QianjiServerFlowhubState, qianji_server_flowhub_router, resolve_qianji_server_flowhub_root,
};
use super::health::{QianjiServerHealthState, check_valkey_ready, qianji_server_health_router};
use super::security::{
    QianjiInternalServiceSecurity, qianji_internal_service_security,
    with_qianji_internal_service_security,
};
#[cfg(feature = "duckdb")]
use crate::QianjiRunConsoleFlightService;
#[cfg(test)]
use crate::runtime_config::resolve_qianji_runtime_checkpoint_config_with_env;
#[cfg(test)]
use crate::runtime_config::resolve_qianji_runtime_server_config_with_env;
use crate::runtime_config::{
    QianjiRuntimeEnv, resolve_qianji_runtime_checkpoint_config,
    resolve_qianji_runtime_server_config,
};
use crate::{
    QianjiBpmnHostBridge, QianjiBpmnWorkflowControlService, QianjiBpmnWorkflowHttpState,
    qianji_bpmn_workflow_router,
};
#[cfg(feature = "duckdb")]
use arrow_flight::flight_service_server::FlightServiceServer;
use axum::Router;
use std::env;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::net::TcpListener;
#[cfg(feature = "duckdb")]
use tokio_stream::wrappers::TcpListenerStream;
use xiuxian_config_core::resolve_path_from_value;
use xiuxian_qianji_control::ControlLedger;
#[cfg(feature = "duckdb")]
use xiuxian_qianji_control::DuckDbControlLedger;

const DEFAULT_QIANJI_SERVER_CONTROL_LEDGER_RELATIVE_PATH: &str =
    "xiuxian-qianji/duckdb/control-ledger.duckdb";
#[cfg(feature = "valkey")]
use xiuxian_qianji_control::{ValkeyHotStateConfig, ValkeyHotStateStore};

pub(crate) type SharedControlLedger = Arc<dyn ControlLedger>;

pub(crate) async fn run_qianji_server(command: QianjiServerCommand) -> anyhow::Result<()> {
    match command {
        QianjiServerCommand::Serve(command) => serve_qianji_server(command).await,
        QianjiServerCommand::Help => {
            println!("{}", qianji_server_usage());
            Ok(())
        }
    }
}

async fn serve_qianji_server(command: QianjiServerServeCommand) -> anyhow::Result<()> {
    let command = install_default_qianji_server_control_ledger(command)?;
    let bind_addr = resolve_qianji_server_bind_addr(&command)?;
    let flight_bind_addr = resolve_qianji_server_flight_bind_addr(&command)?;
    enforce_qianji_server_startup_readiness(&command).await?;
    let control_ledger = build_qianji_server_control_ledger(&command)?;
    let app = build_qianji_server_router_with_control_ledger(&command, control_ledger.clone())?;
    let listener = TcpListener::bind(bind_addr).await?;
    let local_addr = listener.local_addr()?;
    eprintln!("qianji-server listening on http://{local_addr}");

    if let Some(flight_bind_addr) = flight_bind_addr {
        #[cfg(not(feature = "duckdb"))]
        {
            let _ = flight_bind_addr;
            anyhow::bail!("qianji-server Flight listener requires the `duckdb` feature");
        }
        #[cfg(feature = "duckdb")]
        {
            let flight_service =
                build_qianji_server_flight_service(&command, control_ledger.clone())?;
            let flight_listener = TcpListener::bind(flight_bind_addr).await?;
            let flight_local_addr = flight_listener.local_addr()?;
            eprintln!("qianji-server Arrow Flight listening on grpc://{flight_local_addr}");
            let http_server = axum::serve(listener, app);
            let flight_server = tonic::transport::Server::builder()
                .add_service(FlightServiceServer::new(flight_service))
                .serve_with_incoming(TcpListenerStream::new(flight_listener));
            tokio::select! {
                result = http_server => result?,
                result = flight_server => result?,
            }
            return Ok(());
        }
    }

    axum::serve(listener, app).await?;
    Ok(())
}

fn install_default_qianji_server_control_ledger(
    mut command: QianjiServerServeCommand,
) -> anyhow::Result<QianjiServerServeCommand> {
    if command.control_ledger_path.is_none() {
        command.control_ledger_path = Some(resolve_qianji_server_control_ledger_path(&command)?);
    }
    Ok(command)
}

pub(crate) async fn enforce_qianji_server_startup_readiness(
    command: &QianjiServerServeCommand,
) -> anyhow::Result<()> {
    if !resolve_qianji_server_require_valkey_ready(command)? {
        return Ok(());
    }

    let valkey_url = resolve_qianji_server_valkey_url(command)?;
    check_valkey_ready(&valkey_url).await.map_err(|message| {
        anyhow::anyhow!("qianji-server Valkey readiness check failed: {message}")
    })
}

#[cfg(test)]
pub(crate) fn build_qianji_server_router(
    command: &QianjiServerServeCommand,
) -> anyhow::Result<Router> {
    let control_ledger = build_qianji_server_control_ledger(command)?;
    build_qianji_server_router_with_control_ledger(command, control_ledger)
}

fn build_qianji_server_router_with_control_ledger(
    command: &QianjiServerServeCommand,
    control_ledger: Option<SharedControlLedger>,
) -> anyhow::Result<Router> {
    build_qianji_server_router_with_control_ledger_and_security(
        command,
        control_ledger,
        qianji_internal_service_security(),
    )
}

#[cfg(test)]
pub(crate) fn build_qianji_server_router_with_internal_security(
    command: &QianjiServerServeCommand,
    internal_security: Option<QianjiInternalServiceSecurity>,
) -> anyhow::Result<Router> {
    let control_ledger = build_qianji_server_control_ledger(command)?;
    build_qianji_server_router_with_control_ledger_and_security(
        command,
        control_ledger,
        internal_security,
    )
}

fn build_qianji_server_router_with_control_ledger_and_security(
    command: &QianjiServerServeCommand,
    control_ledger: Option<SharedControlLedger>,
    internal_security: Option<QianjiInternalServiceSecurity>,
) -> anyhow::Result<Router> {
    let valkey_url = resolve_qianji_server_valkey_url(command)?;
    #[cfg(feature = "valkey")]
    let workflow_state = build_workflow_http_state(
        build_workflow_control_service(command),
        QianjiBpmnHostBridge::default(),
        command,
        control_ledger,
    )?;
    #[cfg(not(feature = "valkey"))]
    let workflow_state = build_workflow_http_state(
        build_workflow_control_service(command),
        QianjiBpmnHostBridge::default(),
        command,
        control_ledger,
    );
    let health_state = QianjiServerHealthState::new(valkey_url);
    let flowhub_state = QianjiServerFlowhubState::new(resolve_qianji_server_flowhub_root(
        command.flowhub_root.as_deref(),
    ));
    let business_router = qianji_server_flowhub_router(flowhub_state)
        .merge(qianji_bpmn_workflow_router(workflow_state));
    let business_router = match internal_security {
        Some(security) => with_qianji_internal_service_security(business_router, security),
        None => business_router,
    };
    Ok(qianji_server_health_router(health_state).merge(business_router))
}

#[cfg(feature = "valkey")]
pub(crate) fn build_workflow_http_state(
    service: QianjiBpmnWorkflowControlService,
    host: QianjiBpmnHostBridge,
    command: &QianjiServerServeCommand,
    control_ledger: Option<SharedControlLedger>,
) -> anyhow::Result<QianjiBpmnWorkflowHttpState<QianjiBpmnHostBridge>> {
    let state = QianjiBpmnWorkflowHttpState::new(service, host)
        .with_runtime_env(QianjiRuntimeEnv::default());
    let state = install_recovery_hot_state(state, command)?;
    let Some(control_ledger) = control_ledger else {
        return Ok(state);
    };
    Ok(state.with_activity_evidence_ledger(control_ledger))
}

#[cfg(not(feature = "valkey"))]
pub(crate) fn build_workflow_http_state(
    service: QianjiBpmnWorkflowControlService,
    host: QianjiBpmnHostBridge,
    command: &QianjiServerServeCommand,
    control_ledger: Option<SharedControlLedger>,
) -> QianjiBpmnWorkflowHttpState<QianjiBpmnHostBridge> {
    let state = QianjiBpmnWorkflowHttpState::new(service, host)
        .with_runtime_env(QianjiRuntimeEnv::default());
    let state = install_recovery_hot_state(state, command);
    let Some(control_ledger) = control_ledger else {
        return state;
    };
    state.with_activity_evidence_ledger(control_ledger)
}

#[cfg(feature = "valkey")]
fn install_recovery_hot_state(
    state: QianjiBpmnWorkflowHttpState<QianjiBpmnHostBridge>,
    command: &QianjiServerServeCommand,
) -> anyhow::Result<QianjiBpmnWorkflowHttpState<QianjiBpmnHostBridge>> {
    let config =
        ValkeyHotStateConfig::new(resolve_qianji_server_valkey_url(command)?).map_err(|error| {
            anyhow::anyhow!("invalid qianji-server Valkey hot-state config: {error}")
        })?;
    Ok(state.with_recovery_hot_state(Arc::new(ValkeyHotStateStore::new(config))))
}

#[cfg(not(feature = "valkey"))]
fn install_recovery_hot_state(
    state: QianjiBpmnWorkflowHttpState<QianjiBpmnHostBridge>,
    _command: &QianjiServerServeCommand,
) -> QianjiBpmnWorkflowHttpState<QianjiBpmnHostBridge> {
    state
}

pub(crate) fn resolve_qianji_server_bind_addr(
    command: &QianjiServerServeCommand,
) -> anyhow::Result<SocketAddr> {
    if let Some(bind_addr) = command.bind_addr {
        return Ok(bind_addr);
    }

    let config = resolve_qianji_runtime_server_config()?;
    parse_configured_bind_addr(&config.bind_addr)
}

pub(crate) fn resolve_qianji_server_flight_bind_addr(
    command: &QianjiServerServeCommand,
) -> anyhow::Result<Option<SocketAddr>> {
    if let Some(flight_bind_addr) = command.flight_bind_addr {
        return Ok(Some(flight_bind_addr));
    }

    let config = resolve_qianji_runtime_server_config()?;
    config
        .flight_bind_addr
        .as_deref()
        .map(parse_configured_flight_bind_addr)
        .transpose()
}

#[cfg(test)]
pub(crate) fn resolve_qianji_server_bind_addr_with_env(
    command: &QianjiServerServeCommand,
    runtime_env: &QianjiRuntimeEnv,
) -> anyhow::Result<SocketAddr> {
    if let Some(bind_addr) = command.bind_addr {
        return Ok(bind_addr);
    }

    let config = resolve_qianji_runtime_server_config_with_env(runtime_env)?;
    parse_configured_bind_addr(&config.bind_addr)
}

#[cfg(test)]
pub(crate) fn resolve_qianji_server_flight_bind_addr_with_env(
    command: &QianjiServerServeCommand,
    runtime_env: &QianjiRuntimeEnv,
) -> anyhow::Result<Option<SocketAddr>> {
    if let Some(flight_bind_addr) = command.flight_bind_addr {
        return Ok(Some(flight_bind_addr));
    }

    let config = resolve_qianji_runtime_server_config_with_env(runtime_env)?;
    config
        .flight_bind_addr
        .as_deref()
        .map(parse_configured_flight_bind_addr)
        .transpose()
}

fn parse_configured_bind_addr(value: &str) -> anyhow::Result<SocketAddr> {
    value
        .parse()
        .map_err(|error| anyhow::anyhow!("invalid qianji server bind_addr `{value}`: {error}"))
}

fn parse_configured_flight_bind_addr(value: &str) -> anyhow::Result<SocketAddr> {
    value.parse().map_err(|error| {
        anyhow::anyhow!("invalid qianji server flight_bind_addr `{value}`: {error}")
    })
}

pub(crate) fn resolve_qianji_server_valkey_url(
    command: &QianjiServerServeCommand,
) -> anyhow::Result<String> {
    if let Some(valkey_url) = command.valkey_url.as_ref() {
        return Ok(valkey_url.clone());
    }

    let config = resolve_qianji_runtime_checkpoint_config()?;
    Ok(config.valkey_url)
}

pub(crate) fn resolve_qianji_server_control_ledger_path(
    command: &QianjiServerServeCommand,
) -> anyhow::Result<PathBuf> {
    resolve_qianji_server_control_ledger_path_with_env(command, &QianjiRuntimeEnv::default(), true)
}

pub(crate) fn resolve_qianji_server_control_ledger_path_with_env(
    command: &QianjiServerServeCommand,
    runtime_env: &QianjiRuntimeEnv,
    read_process_env: bool,
) -> anyhow::Result<PathBuf> {
    if let Some(path) = command.control_ledger_path.as_ref() {
        return Ok(path.clone());
    }

    if let Some(path) = resolve_control_ledger_env_path(
        "QIANJI_SERVER_CONTROL_LEDGER",
        runtime_env,
        read_process_env,
    )? {
        return Ok(path);
    }
    if let Some(path) = resolve_control_ledger_env_path(
        "QIANJI_SERVER_CONTROL_LEDGER_PATH",
        runtime_env,
        read_process_env,
    )? {
        return Ok(path);
    }

    Ok(
        resolve_qianji_server_data_home(runtime_env, read_process_env)?
            .join(DEFAULT_QIANJI_SERVER_CONTROL_LEDGER_RELATIVE_PATH),
    )
}

fn resolve_control_ledger_env_path(
    key: &str,
    runtime_env: &QianjiRuntimeEnv,
    read_process_env: bool,
) -> anyhow::Result<Option<PathBuf>> {
    let Some(raw_value) = qianji_server_env_value(runtime_env, key, read_process_env) else {
        return Ok(None);
    };
    Ok(resolve_path_from_value(
        Some(resolve_qianji_server_project_root(runtime_env, read_process_env)?.as_path()),
        Some(raw_value.as_str()),
    ))
}

fn resolve_qianji_server_data_home(
    runtime_env: &QianjiRuntimeEnv,
    read_process_env: bool,
) -> anyhow::Result<PathBuf> {
    let project_root = resolve_qianji_server_project_root(runtime_env, read_process_env)?;
    if let Some(path) = runtime_env.prj_data_home.as_ref() {
        return Ok(path.clone());
    }
    if let Some(raw_data_home) =
        qianji_server_env_value(runtime_env, "PRJ_DATA_HOME", read_process_env)
        && let Some(path) =
            resolve_path_from_value(Some(project_root.as_path()), Some(&raw_data_home))
    {
        return Ok(path);
    }
    Ok(project_root.join(".data"))
}

fn resolve_qianji_server_project_root(
    runtime_env: &QianjiRuntimeEnv,
    read_process_env: bool,
) -> anyhow::Result<PathBuf> {
    if let Some(path) = runtime_env.prj_root.as_ref() {
        return Ok(path.clone());
    }
    if let Some(raw_project_root) =
        qianji_server_env_value(runtime_env, "PRJ_ROOT", read_process_env)
        && let Some(path) = resolve_path_from_value(None::<&Path>, Some(&raw_project_root))
    {
        return Ok(path);
    }
    env::current_dir()
        .map_err(|error| anyhow::anyhow!("failed to resolve qianji-server current dir: {error}"))
}

fn qianji_server_env_value(
    runtime_env: &QianjiRuntimeEnv,
    key: &str,
    read_process_env: bool,
) -> Option<String> {
    runtime_env
        .extra_env
        .iter()
        .find(|(candidate_key, _)| candidate_key == key)
        .map(|(_, value)| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            read_process_env
                .then(|| env::var(key).ok().map(|value| value.trim().to_string()))
                .flatten()
                .filter(|value| !value.is_empty())
        })
}

pub(crate) fn resolve_qianji_server_require_valkey_ready(
    command: &QianjiServerServeCommand,
) -> anyhow::Result<bool> {
    if let Some(require_valkey_ready) = command.require_valkey_ready {
        return Ok(require_valkey_ready);
    }

    let config = resolve_qianji_runtime_server_config()?;
    Ok(config.require_valkey_ready)
}

#[cfg(test)]
pub(crate) fn resolve_qianji_server_valkey_url_with_env(
    command: &QianjiServerServeCommand,
    runtime_env: &QianjiRuntimeEnv,
) -> anyhow::Result<String> {
    if let Some(valkey_url) = command.valkey_url.as_ref() {
        return Ok(valkey_url.clone());
    }

    let config = resolve_qianji_runtime_checkpoint_config_with_env(runtime_env)?;
    Ok(config.valkey_url)
}

#[cfg(test)]
pub(crate) fn resolve_qianji_server_require_valkey_ready_with_env(
    command: &QianjiServerServeCommand,
    runtime_env: &QianjiRuntimeEnv,
) -> anyhow::Result<bool> {
    if let Some(require_valkey_ready) = command.require_valkey_ready {
        return Ok(require_valkey_ready);
    }

    let config = resolve_qianji_runtime_server_config_with_env(runtime_env)?;
    Ok(config.require_valkey_ready)
}

pub(crate) fn build_workflow_control_service(
    command: &QianjiServerServeCommand,
) -> QianjiBpmnWorkflowControlService {
    let Some(valkey_url) = command.valkey_url.as_ref() else {
        return QianjiBpmnWorkflowControlService::new();
    };

    QianjiBpmnWorkflowControlService::new().with_runtime_env(QianjiRuntimeEnv {
        qianji_checkpoint_valkey_url: Some(valkey_url.clone()),
        ..QianjiRuntimeEnv::default()
    })
}

#[cfg(feature = "duckdb")]
pub(crate) fn build_qianji_server_flight_service(
    command: &QianjiServerServeCommand,
    control_ledger: Option<SharedControlLedger>,
) -> anyhow::Result<QianjiRunConsoleFlightService> {
    let control_ledger = match control_ledger {
        Some(control_ledger) => control_ledger,
        None => build_required_qianji_server_control_ledger(command)?,
    };
    let service = QianjiRunConsoleFlightService::new(control_ledger);
    Ok(match qianji_internal_service_security() {
        Some(security) => service.with_internal_security(security),
        None => service,
    })
}

fn build_qianji_server_control_ledger(
    command: &QianjiServerServeCommand,
) -> anyhow::Result<Option<SharedControlLedger>> {
    let Some(_ledger_path) = command.control_ledger_path.as_ref() else {
        return Ok(None);
    };
    Ok(Some(build_required_qianji_server_control_ledger(command)?))
}

fn build_required_qianji_server_control_ledger(
    command: &QianjiServerServeCommand,
) -> anyhow::Result<SharedControlLedger> {
    let ledger_path = resolve_qianji_server_control_ledger_path(command)?;
    #[cfg(not(feature = "duckdb"))]
    {
        let _ = ledger_path;
        anyhow::bail!("qianji-server --control-ledger requires the `duckdb` feature");
    }
    #[cfg(feature = "duckdb")]
    {
        let ledger = DuckDbControlLedger::open(ledger_path.clone()).map_err(|error| {
            anyhow::anyhow!(
                "failed to open qianji-server control ledger {}: {error}",
                ledger_path.display()
            )
        })?;
        Ok(Arc::new(ledger))
    }
}
