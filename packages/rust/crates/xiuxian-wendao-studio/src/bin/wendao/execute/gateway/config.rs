//! Gateway configuration resolution.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use crate::studio::studio_effective_wendao_toml_path;
use log::info;
use serde::Deserialize;
#[cfg(test)]
use xiuxian_config_core::resolve_project_root_or_cwd_from_value;
use xiuxian_config_core::{
    first_non_empty_lookup, first_non_empty_named_lookup, load_toml_value_with_imports,
    resolve_project_root,
};
use xiuxian_zhenfa::WebhookConfig;

use crate::bin_support::wendao::execute::gateway::shared::{DEFAULT_BIND_ADDR, DEFAULT_PORT};

/// Resolve the effective config file from CLI override, local project file, or
/// `PRJ_ROOT`.
pub(crate) fn resolve_config_path(cli_config: Option<&Path>) -> Option<PathBuf> {
    let project_root = resolve_project_root();
    resolve_config_path_with_project_root(cli_config, project_root.as_deref())
}

fn resolve_config_path_with_project_root(
    cli_config: Option<&Path>,
    project_root: Option<&Path>,
) -> Option<PathBuf> {
    if let Some(path) = cli_config {
        return Some(resolve_effective_config_path(path));
    }

    let local_config = Path::new("wendao.toml");
    if local_config.exists() {
        return Some(resolve_effective_config_path(local_config));
    }

    let config_path = project_root.map(|root| root.join("wendao.toml"))?;
    config_path
        .exists()
        .then(|| resolve_effective_config_path(config_path.as_path()))
}

#[cfg(test)]
fn resolve_config_path_with_project_root_value(
    cli_config: Option<&Path>,
    project_root_value: Option<&str>,
    current_dir: Option<&Path>,
) -> Option<PathBuf> {
    let project_root = resolve_project_root_or_cwd_from_value(project_root_value, current_dir);
    resolve_config_path_with_project_root(cli_config, Some(project_root.as_path()))
}

/// Resolve the bind address from config file or default (`127.0.0.1`).
///
/// Reads `[gateway].bind` which may be a bare IP (`"0.0.0.0"`) or an
/// `ip:port` pair (`"0.0.0.0:9517"`).
pub(crate) fn resolve_bind_addr(config_path: Option<&Path>) -> [u8; 4] {
    if let Some(addr) = get_bind_addr_from_config(config_path) {
        return addr;
    }

    DEFAULT_BIND_ADDR
}

/// Get bind address from wendao.toml config file.
fn get_bind_addr_from_config(config_path: Option<&Path>) -> Option<[u8; 4]> {
    let config = load_gateway_toml(config_path?)?;
    config.gateway.bind_addr()
}

/// Resolve the port from CLI arg, config file, or default.
pub(crate) fn resolve_port(cli_port: Option<u16>, config_path: Option<&Path>) -> u16 {
    if let Some(port) = cli_port {
        return port;
    }

    if let Some(config_port) = get_port_from_config(config_path) {
        return config_port;
    }

    DEFAULT_PORT
}

/// Get port from wendao.toml config file.
pub(crate) fn get_port_from_config(config_path: Option<&Path>) -> Option<u16> {
    parse_port_from_toml(config_path?)
}

/// Parse port from a TOML config file.
pub(crate) fn parse_port_from_toml(path: &Path) -> Option<u16> {
    load_gateway_toml(path).and_then(|config| config.gateway.port())
}

/// Resolve webhook config with priority: TOML > env var > defaults.
pub(crate) fn resolve_webhook_config(config_path: Option<&Path>) -> WebhookConfig {
    resolve_webhook_config_with_lookup(config_path, &|name| std::env::var(name).ok())
}

pub(crate) fn resolve_webhook_config_with_lookup(
    config_path: Option<&Path>,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> WebhookConfig {
    if let Some(config) = get_webhook_from_config(config_path) {
        info!("Gateway: Using webhook config from wendao.toml");
        return config;
    }

    let url = first_non_empty_named_lookup(&["WENDAO_WEBHOOK_URL"], lookup);
    if let Some(candidate) = url.as_ref() {
        info!(
            "Gateway: Using webhook config from {} env var",
            candidate.source_name
        );
    }

    WebhookConfig {
        url: url.map(|candidate| candidate.value).unwrap_or_default(),
        secret: first_non_empty_lookup(&["WENDAO_WEBHOOK_SECRET"], lookup),
        timeout_secs: 10,
        retry_on_failure: true,
    }
}

/// Get webhook config from wendao.toml config file.
pub(crate) fn get_webhook_from_config(config_path: Option<&Path>) -> Option<WebhookConfig> {
    parse_webhook_from_toml(config_path?)
}

/// Get gateway runtime knob overrides from wendao.toml config file.
pub(crate) fn get_gateway_runtime_from_config(
    config_path: Option<&Path>,
) -> Option<GatewayRuntimeTomlConfig> {
    parse_gateway_runtime_from_toml(config_path?)
}

/// Parse webhook config from a TOML config file.
pub(crate) fn parse_webhook_from_toml(path: &Path) -> Option<WebhookConfig> {
    load_gateway_toml(path).and_then(|config| config.gateway.webhook_config())
}

/// Parse gateway runtime knobs from a TOML config file.
pub(crate) fn parse_gateway_runtime_from_toml(path: &Path) -> Option<GatewayRuntimeTomlConfig> {
    load_gateway_toml(path).and_then(|config| config.gateway.runtime_config())
}

fn resolve_effective_config_path(path: &Path) -> PathBuf {
    let Some(file_name) = path.file_name() else {
        return path.to_path_buf();
    };
    if file_name != "wendao.toml" {
        return path.to_path_buf();
    }

    let Some(config_root) = path.parent() else {
        return path.to_path_buf();
    };
    let effective_path = studio_effective_wendao_toml_path(config_root);
    if effective_path.is_file() {
        effective_path
    } else {
        path.to_path_buf()
    }
}

fn load_gateway_toml(path: &Path) -> Option<GatewayTomlConfig> {
    let merged = load_toml_value_with_imports(path).ok()?;
    merged.try_into().ok()
}

#[derive(Debug, Default, Deserialize)]
struct GatewayTomlConfig {
    #[serde(default)]
    gateway: GatewayTomlSection,
}

#[derive(Debug, Default, Deserialize)]
struct GatewayTomlSection {
    #[serde(default)]
    bind: Option<String>,
    #[serde(default)]
    port: Option<u16>,
    #[serde(default)]
    webhook_url: Option<String>,
    #[serde(default)]
    webhook_secret: Option<String>,
    #[serde(default)]
    webhook_enabled: Option<bool>,
    #[serde(default)]
    runtime: GatewayRuntimeTomlSection,
}

#[derive(Debug, Default, Deserialize, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GatewayRuntimeTomlConfig {
    pub(crate) listen_backlog: Option<u32>,
    pub(crate) studio_concurrency_limit: Option<usize>,
    pub(crate) studio_request_timeout_secs: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
struct GatewayRuntimeTomlSection {
    #[serde(default)]
    listen_backlog: Option<u32>,
    #[serde(default)]
    studio_concurrency_limit: Option<usize>,
    #[serde(default)]
    studio_request_timeout_secs: Option<u64>,
}

impl GatewayTomlSection {
    fn port(&self) -> Option<u16> {
        self.port
            .or_else(|| self.bind.as_deref().and_then(parse_bind_port))
    }

    fn bind_addr(&self) -> Option<[u8; 4]> {
        self.bind.as_deref().and_then(parse_bind_addr)
    }

    fn webhook_config(&self) -> Option<WebhookConfig> {
        if self.webhook_enabled == Some(false) {
            return None;
        }

        self.webhook_url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|url| WebhookConfig {
                url: url.to_string(),
                secret: self
                    .webhook_secret
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string),
                timeout_secs: 10,
                retry_on_failure: true,
            })
    }

    fn runtime_config(&self) -> Option<GatewayRuntimeTomlConfig> {
        let config = GatewayRuntimeTomlConfig {
            listen_backlog: self.runtime.listen_backlog,
            studio_concurrency_limit: self.runtime.studio_concurrency_limit,
            studio_request_timeout_secs: self.runtime.studio_request_timeout_secs,
        };
        (config.listen_backlog.is_some()
            || config.studio_concurrency_limit.is_some()
            || config.studio_request_timeout_secs.is_some())
        .then_some(config)
    }
}

fn parse_bind_addr(bind: &str) -> Option<[u8; 4]> {
    // Try as full SocketAddr (e.g. "0.0.0.0:9517")
    if let Ok(addr) = bind.parse::<SocketAddr>()
        && let std::net::IpAddr::V4(v4) = addr.ip()
    {
        return Some(v4.octets());
    }
    // Try as bare IPv4 (e.g. "0.0.0.0")
    if let Ok(v4) = bind.parse::<std::net::Ipv4Addr>() {
        return Some(v4.octets());
    }
    None
}

fn parse_bind_port(bind: &str) -> Option<u16> {
    bind.parse::<SocketAddr>()
        .ok()
        .map(|address| address.port())
        .or_else(|| {
            bind.rsplit_once(':')
                .and_then(|(_, port)| port.trim().parse::<u16>().ok())
        })
}

#[cfg(test)]
#[path = "../../../../../tests/unit/bin/wendao/execute/gateway/config.rs"]
mod tests;
