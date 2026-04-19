use std::path::{Path, PathBuf};
use std::sync::Arc;

use tempfile::Builder;
use tokio::sync::mpsc;
use xiuxian_wendao::analyzers::PluginRegistry;
use xiuxian_zhenfa::ZhenfaSignal;

use crate::execute::gateway::registry::build_plugin_registry;
use crate::execute::gateway::shared::AppState;

pub(crate) fn write_temp_gateway_config(contents: &str) -> PathBuf {
    let (mut file, path) = Builder::new()
        .prefix("wendao-gateway-config-")
        .suffix(".toml")
        .tempfile()
        .unwrap_or_else(|error| panic!("failed to allocate temp gateway config: {error}"))
        .keep()
        .unwrap_or_else(|error| panic!("failed to persist temp gateway config: {error}"));
    if let Err(err) = std::io::Write::write_all(&mut file, contents.as_bytes()) {
        panic!("failed to write temp config at {}: {err}", path.display());
    }
    path
}

pub(crate) fn remove_temp_gateway_config(path: &Path) {
    if let Err(err) = std::fs::remove_file(path)
        && path.exists()
    {
        panic!("failed to remove temp config at {}: {err}", path.display());
    }
}

pub(super) fn write_temp_gateway_pidfile(contents: &str) -> PathBuf {
    let (mut file, path) = Builder::new()
        .prefix("wendao-gateway-pidfile-")
        .suffix(".pid")
        .tempfile()
        .unwrap_or_else(|error| panic!("failed to allocate temp gateway pidfile: {error}"))
        .keep()
        .unwrap_or_else(|error| panic!("failed to persist temp gateway pidfile: {error}"));
    if let Err(err) = std::io::Write::write_all(&mut file, contents.as_bytes()) {
        panic!("failed to write temp pidfile at {}: {err}", path.display());
    }
    path
}

pub(super) fn remove_temp_gateway_pidfile(path: &Path) {
    if let Err(err) = std::fs::remove_file(path)
        && path.exists()
    {
        panic!("failed to remove temp pidfile at {}: {err}", path.display());
    }
}

pub(super) fn mismatched_pid() -> u32 {
    let current = std::process::id();
    if current == u32::MAX {
        current - 1
    } else {
        current + 1
    }
}

pub(super) fn app_state(signal_tx: Option<mpsc::UnboundedSender<ZhenfaSignal>>) -> Arc<AppState> {
    app_state_with_webhook_url(signal_tx, None)
}

pub(super) fn app_state_with_webhook_url(
    signal_tx: Option<mpsc::UnboundedSender<ZhenfaSignal>>,
    webhook_url: Option<String>,
) -> Arc<AppState> {
    Arc::new(AppState::new_with_webhook_url(
        None,
        signal_tx,
        webhook_url,
        bootstrap_builtin_registry(),
    ))
}

pub(super) fn bootstrap_builtin_registry() -> Arc<PluginRegistry> {
    build_plugin_registry().unwrap_or_else(|error| panic!("bootstrap builtin registry: {error}"))
}
