use super::cli::{QianjiServerCommand, QianjiServerServeCommand, qianji_server_usage};
use super::flowhub::{
    QianjiServerFlowhubState, qianji_server_flowhub_router, resolve_qianji_server_flowhub_root,
};
use super::health::{QianjiServerHealthState, check_valkey_ready, qianji_server_health_router};
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
use axum::Router;
use std::net::SocketAddr;
#[cfg(feature = "duckdb")]
use std::sync::Arc;
use tokio::net::TcpListener;
#[cfg(feature = "duckdb")]
use xiuxian_qianji_control::DuckDbControlLedger;

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
    let bind_addr = resolve_qianji_server_bind_addr(&command)?;
    enforce_qianji_server_startup_readiness(&command).await?;
    let app = build_qianji_server_router(&command)?;
    let listener = TcpListener::bind(bind_addr).await?;
    let local_addr = listener.local_addr()?;
    eprintln!("qianji-server listening on http://{local_addr}");

    axum::serve(listener, app).await?;
    Ok(())
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

pub(crate) fn build_qianji_server_router(
    command: &QianjiServerServeCommand,
) -> anyhow::Result<Router> {
    let valkey_url = resolve_qianji_server_valkey_url(command)?;
    let workflow_state = build_workflow_http_state(
        build_workflow_control_service(command),
        QianjiBpmnHostBridge::default(),
        command,
    )?;
    let health_state = QianjiServerHealthState::new(valkey_url);
    let flowhub_state = QianjiServerFlowhubState::new(resolve_qianji_server_flowhub_root(
        command.flowhub_root.as_deref(),
    ));
    Ok(qianji_server_health_router(health_state)
        .merge(qianji_server_flowhub_router(flowhub_state))
        .merge(qianji_bpmn_workflow_router(workflow_state)))
}

fn build_workflow_http_state(
    service: QianjiBpmnWorkflowControlService,
    host: QianjiBpmnHostBridge,
    command: &QianjiServerServeCommand,
) -> anyhow::Result<QianjiBpmnWorkflowHttpState<QianjiBpmnHostBridge>> {
    let state = QianjiBpmnWorkflowHttpState::new(service, host);
    let Some(ledger_path) = command.control_ledger_path.as_ref() else {
        return Ok(state);
    };
    #[cfg(not(feature = "duckdb"))]
    {
        let _ = ledger_path;
        anyhow::bail!("qianji-server --control-ledger requires the `duckdb` feature");
    }
    #[cfg(feature = "duckdb")]
    {
        let ledger = DuckDbControlLedger::open(ledger_path).map_err(|error| {
            anyhow::anyhow!(
                "failed to open qianji-server control ledger {}: {error}",
                ledger_path.display()
            )
        })?;
        Ok(state.with_activity_evidence_ledger(Arc::new(ledger)))
    }
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

fn parse_configured_bind_addr(value: &str) -> anyhow::Result<SocketAddr> {
    value
        .parse()
        .map_err(|error| anyhow::anyhow!("invalid qianji server bind_addr `{value}`: {error}"))
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
