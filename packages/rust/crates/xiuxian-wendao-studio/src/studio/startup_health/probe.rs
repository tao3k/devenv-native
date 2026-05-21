//! Owns the Studio studio startup health probe surface.

use std::time::Duration;

use crate::studio::startup_health::types::{
    GatewayStartupDependencyCheck, GatewayStartupHealthReport,
};
use xiuxian_wendao::analyzers::PluginRegistry;
use xiuxian_wendao::link_graph::runtime_config::resolve_link_graph_cache_runtime;
use xiuxian_wendao::search::resolve_search_plane_cache_connection_target;
use xiuxian_wendao::valkey_common::{open_client, ping_client, ping_valkey_url};

const BUILTIN_PLUGIN_REGISTRY_DEPENDENCY: &str = "builtin_plugin_registry";
const SEARCH_CACHE_VALKEY_DEPENDENCY: &str = "search_cache_valkey";
const LINK_GRAPH_CACHE_VALKEY_DEPENDENCY: &str = "link_graph_cache_valkey";
const STARTUP_VALKEY_CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const STARTUP_VALKEY_IO_TIMEOUT: Duration = Duration::from_secs(2);

/// Probe required gateway startup dependencies before listener bind.
#[must_use]
pub fn probe_gateway_startup_health(
    plugin_registry: &PluginRegistry,
) -> GatewayStartupHealthReport {
    GatewayStartupHealthReport::new(vec![
        probe_plugin_registry(plugin_registry),
        probe_search_cache_valkey(),
        probe_link_graph_cache_valkey(),
    ])
}

/// Render stable one-line summaries for startup logs.
#[must_use]
pub fn describe_gateway_startup_health(report: &GatewayStartupHealthReport) -> Vec<String> {
    report
        .checks()
        .iter()
        .map(|check| {
            format!(
                "{}={} {}",
                check.dependency,
                check.status.label(),
                check.detail
            )
        })
        .collect()
}

fn probe_plugin_registry(plugin_registry: &PluginRegistry) -> GatewayStartupDependencyCheck {
    probe_plugin_registry_with_ids(plugin_registry.plugin_ids())
}

fn probe_plugin_registry_with_ids<I, S>(plugin_ids: I) -> GatewayStartupDependencyCheck
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let plugin_ids = plugin_ids
        .into_iter()
        .map(|plugin_id| plugin_id.as_ref().to_string())
        .collect::<Vec<_>>();

    if plugin_ids.is_empty() {
        return GatewayStartupDependencyCheck::failed(
            BUILTIN_PLUGIN_REGISTRY_DEPENDENCY,
            "no builtin repo-intelligence plugins registered",
        );
    }

    GatewayStartupDependencyCheck::connected(
        BUILTIN_PLUGIN_REGISTRY_DEPENDENCY,
        format!("plugins={}", plugin_ids.join(",")),
    )
}

fn probe_search_cache_valkey() -> GatewayStartupDependencyCheck {
    let target = match resolve_search_plane_cache_connection_target() {
        Ok(target) => target,
        Err(error) => {
            return GatewayStartupDependencyCheck::failed(SEARCH_CACHE_VALKEY_DEPENDENCY, error);
        }
    };

    probe_search_cache_valkey_with(
        Ok((
            target.valkey_url,
            target.config.connection_timeout,
            target.config.response_timeout,
        )),
        &|valkey_url, connection_timeout, response_timeout| {
            let client =
                open_client(valkey_url).map_err(|error| format!("invalid valkey url: {error}"))?;
            ping_client(&client, connection_timeout, response_timeout)
        },
    )
}

fn probe_search_cache_valkey_with(
    target: Result<(String, Duration, Duration), String>,
    ping_probe: &dyn Fn(&str, Duration, Duration) -> Result<String, String>,
) -> GatewayStartupDependencyCheck {
    let (valkey_url, connection_timeout, response_timeout) = match target {
        Ok(target) => target,
        Err(error) => {
            return GatewayStartupDependencyCheck::failed(SEARCH_CACHE_VALKEY_DEPENDENCY, error);
        }
    };

    match ping_probe(valkey_url.as_str(), connection_timeout, response_timeout) {
        Ok(ping_reply) => GatewayStartupDependencyCheck::connected(
            SEARCH_CACHE_VALKEY_DEPENDENCY,
            format!("url={valkey_url} ping={ping_reply}"),
        ),
        Err(error) => GatewayStartupDependencyCheck::failed(
            SEARCH_CACHE_VALKEY_DEPENDENCY,
            format!("url={valkey_url} {error}"),
        ),
    }
}

fn probe_link_graph_cache_valkey() -> GatewayStartupDependencyCheck {
    let runtime = match resolve_link_graph_cache_runtime() {
        Ok(runtime) => runtime,
        Err(error) => {
            return GatewayStartupDependencyCheck::failed(
                LINK_GRAPH_CACHE_VALKEY_DEPENDENCY,
                error,
            );
        }
    };

    probe_link_graph_cache_valkey_with(
        Ok((runtime.valkey_url, runtime.key_prefix)),
        &|valkey_url| {
            ping_valkey_url(
                valkey_url,
                STARTUP_VALKEY_CONNECT_TIMEOUT,
                STARTUP_VALKEY_IO_TIMEOUT,
            )
        },
    )
}

fn probe_link_graph_cache_valkey_with(
    runtime: Result<(String, String), String>,
    ping_probe: &dyn Fn(&str) -> Result<String, String>,
) -> GatewayStartupDependencyCheck {
    let (valkey_url, key_prefix) = match runtime {
        Ok(runtime) => runtime,
        Err(error) => {
            return GatewayStartupDependencyCheck::failed(
                LINK_GRAPH_CACHE_VALKEY_DEPENDENCY,
                error,
            );
        }
    };

    match ping_probe(valkey_url.as_str()) {
        Ok(ping_reply) => GatewayStartupDependencyCheck::connected(
            LINK_GRAPH_CACHE_VALKEY_DEPENDENCY,
            format!("url={valkey_url} ping={ping_reply} key_prefix={key_prefix}"),
        ),
        Err(error) => GatewayStartupDependencyCheck::failed(
            LINK_GRAPH_CACHE_VALKEY_DEPENDENCY,
            format!("url={valkey_url} {error}"),
        ),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/gateway/studio/startup_health/probe.rs"]
mod tests;
