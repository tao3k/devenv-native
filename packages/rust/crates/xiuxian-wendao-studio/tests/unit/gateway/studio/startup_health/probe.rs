use std::time::Duration;

use crate::studio::GatewayStartupDependencyStatus;
use crate::studio::startup_health::probe::{
    BUILTIN_PLUGIN_REGISTRY_DEPENDENCY, LINK_GRAPH_CACHE_VALKEY_DEPENDENCY,
    SEARCH_CACHE_VALKEY_DEPENDENCY, probe_link_graph_cache_valkey_with,
    probe_plugin_registry_with_ids, probe_search_cache_valkey_with,
};

#[test]
fn plugin_registry_probe_fails_when_no_plugins_are_registered() {
    let check = probe_plugin_registry_with_ids(Vec::<String>::new());

    assert_eq!(check.dependency, BUILTIN_PLUGIN_REGISTRY_DEPENDENCY);
    assert_eq!(check.status, GatewayStartupDependencyStatus::Failed);
    assert_eq!(
        check.detail,
        "no builtin repo-intelligence plugins registered"
    );
}

#[test]
fn plugin_registry_probe_reports_registered_plugin_ids() {
    let check = probe_plugin_registry_with_ids(vec![
        "julia-code-parser".to_string(),
        "modelica".to_string(),
    ]);

    assert_eq!(check.dependency, BUILTIN_PLUGIN_REGISTRY_DEPENDENCY);
    assert_eq!(check.status, GatewayStartupDependencyStatus::Connected);
    assert_eq!(check.detail, "plugins=julia-code-parser,modelica");
}

#[test]
fn search_cache_probe_reports_missing_configuration_failures() {
    let check = probe_search_cache_valkey_with(
        Err("missing search cache valkey url".to_string()),
        &|_, _, _| Ok("PONG".to_string()),
    );

    assert_eq!(check.dependency, SEARCH_CACHE_VALKEY_DEPENDENCY);
    assert_eq!(check.status, GatewayStartupDependencyStatus::Failed);
    assert_eq!(check.detail, "missing search cache valkey url");
}

#[test]
fn search_cache_probe_reports_ping_success_with_url() {
    let check = probe_search_cache_valkey_with(
        Ok((
            "redis://127.0.0.1:6379/0".to_string(),
            Duration::from_millis(40),
            Duration::from_millis(50),
        )),
        &|_, _, _| Ok("PONG".to_string()),
    );

    assert_eq!(check.dependency, SEARCH_CACHE_VALKEY_DEPENDENCY);
    assert_eq!(check.status, GatewayStartupDependencyStatus::Connected);
    assert_eq!(check.detail, "url=redis://127.0.0.1:6379/0 ping=PONG");
}

#[test]
fn link_graph_probe_reports_ping_failures_with_url() {
    let check = probe_link_graph_cache_valkey_with(
        Ok((
            "redis://127.0.0.1:6379/1".to_string(),
            "xiuxian:link_graph".to_string(),
        )),
        &|_| Err("connection failed".to_string()),
    );

    assert_eq!(check.dependency, LINK_GRAPH_CACHE_VALKEY_DEPENDENCY);
    assert_eq!(check.status, GatewayStartupDependencyStatus::Failed);
    assert_eq!(
        check.detail,
        "url=redis://127.0.0.1:6379/1 connection failed"
    );
}

#[test]
fn link_graph_probe_reports_success_with_key_prefix() {
    let check = probe_link_graph_cache_valkey_with(
        Ok((
            "redis://127.0.0.1:6379/1".to_string(),
            "xiuxian:link_graph".to_string(),
        )),
        &|_| Ok("PONG".to_string()),
    );

    assert_eq!(check.dependency, LINK_GRAPH_CACHE_VALKEY_DEPENDENCY);
    assert_eq!(check.status, GatewayStartupDependencyStatus::Connected);
    assert_eq!(
        check.detail,
        "url=redis://127.0.0.1:6379/1 ping=PONG key_prefix=xiuxian:link_graph"
    );
}
