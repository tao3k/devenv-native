use crate::studio::{GatewayStartupDependencyCheck, GatewayStartupHealthReport};

use crate::bin_support::wendao::execute::gateway::command::{
    ensure_gateway_startup_health, gateway_bearer_token_with_lookup,
    gateway_flight_concurrency_limit_with_lookup, gateway_flight_grpc_web_enabled_with_lookup,
    gateway_flight_request_timeout_secs_with_lookup, gateway_listen_backlog_with_lookup,
    gateway_studio_concurrency_limit_with_lookup, gateway_studio_request_timeout_secs_with_lookup,
};
use crate::bin_support::wendao::execute::gateway::config::{
    GatewayRuntimeTomlConfig, get_gateway_runtime_from_config,
};
use crate::bin_support::wendao::execute::gateway::shared::DEFAULT_PORT;

use super::support::{
    bootstrap_builtin_registry, remove_temp_gateway_config, write_temp_gateway_config,
};

#[test]
fn test_default_port() {
    assert_eq!(DEFAULT_PORT, 9517);
}

#[test]
fn test_gateway_listen_backlog_defaults_when_env_missing() {
    let backlog = gateway_listen_backlog_with_lookup(None, &|_| None);
    assert_eq!(backlog, 2048);
}

#[test]
fn test_gateway_listen_backlog_accepts_positive_override() {
    let backlog = gateway_listen_backlog_with_lookup(None, &|key| {
        if key == "XIUXIAN_WENDAO_GATEWAY_LISTEN_BACKLOG" {
            Some("4096".to_string())
        } else {
            None
        }
    });
    assert_eq!(backlog, 4096);
}

#[test]
fn test_gateway_listen_backlog_clamps_invalid_override() {
    let backlog = gateway_listen_backlog_with_lookup(None, &|key| {
        if key == "XIUXIAN_WENDAO_GATEWAY_LISTEN_BACKLOG" {
            Some("0".to_string())
        } else {
            None
        }
    });
    assert_eq!(backlog, 2048);
}

#[test]
fn test_gateway_studio_concurrency_limit_defaults_from_parallelism() {
    let limit = gateway_studio_concurrency_limit_with_lookup(None, &|_| None, Some(8));
    assert_eq!(limit, 32);
}

#[test]
fn test_gateway_studio_concurrency_limit_accepts_positive_override() {
    let limit = gateway_studio_concurrency_limit_with_lookup(
        None,
        &|key| {
            if key == "XIUXIAN_WENDAO_GATEWAY_STUDIO_CONCURRENCY_LIMIT" {
                Some("96".to_string())
            } else {
                None
            }
        },
        Some(8),
    );
    assert_eq!(limit, 96);
}

#[test]
fn test_gateway_studio_concurrency_limit_ignores_invalid_override() {
    let limit = gateway_studio_concurrency_limit_with_lookup(
        None,
        &|key| {
            if key == "XIUXIAN_WENDAO_GATEWAY_STUDIO_CONCURRENCY_LIMIT" {
                Some("-1".to_string())
            } else {
                None
            }
        },
        Some(8),
    );
    assert_eq!(limit, 32);
}

#[test]
fn test_gateway_studio_concurrency_limit_clamps_large_override() {
    let limit = gateway_studio_concurrency_limit_with_lookup(
        None,
        &|key| {
            if key == "XIUXIAN_WENDAO_GATEWAY_STUDIO_CONCURRENCY_LIMIT" {
                Some("320".to_string())
            } else {
                None
            }
        },
        Some(8),
    );
    assert_eq!(limit, 128);
}

#[test]
fn test_gateway_studio_request_timeout_defaults_when_env_missing() {
    let timeout = gateway_studio_request_timeout_secs_with_lookup(None, &|_| None);
    assert_eq!(timeout, 15);
}

#[test]
fn test_gateway_studio_request_timeout_accepts_positive_override() {
    let timeout = gateway_studio_request_timeout_secs_with_lookup(None, &|key| {
        if key == "XIUXIAN_WENDAO_GATEWAY_STUDIO_REQUEST_TIMEOUT_SECS" {
            Some("25".to_string())
        } else {
            None
        }
    });
    assert_eq!(timeout, 25);
}

#[test]
fn test_gateway_studio_request_timeout_clamps_invalid_override() {
    let timeout = gateway_studio_request_timeout_secs_with_lookup(None, &|key| {
        if key == "XIUXIAN_WENDAO_GATEWAY_STUDIO_REQUEST_TIMEOUT_SECS" {
            Some("0".to_string())
        } else {
            None
        }
    });
    assert_eq!(timeout, 15);
}

#[test]
fn test_gateway_flight_concurrency_limit_defaults_to_studio_budget() {
    let limit = gateway_flight_concurrency_limit_with_lookup(48, &|_| None);
    assert_eq!(limit, 48);
}

#[test]
fn test_gateway_flight_concurrency_limit_accepts_positive_override() {
    let limit = gateway_flight_concurrency_limit_with_lookup(48, &|key| {
        if key == "XIUXIAN_WENDAO_GATEWAY_FLIGHT_CONCURRENCY_LIMIT" {
            Some("24".to_string())
        } else {
            None
        }
    });
    assert_eq!(limit, 24);
}

#[test]
fn test_gateway_flight_request_timeout_defaults_to_studio_budget() {
    let timeout = gateway_flight_request_timeout_secs_with_lookup(18, &|_| None);
    assert_eq!(timeout, 18);
}

#[test]
fn test_gateway_flight_request_timeout_accepts_positive_override() {
    let timeout = gateway_flight_request_timeout_secs_with_lookup(18, &|key| {
        if key == "XIUXIAN_WENDAO_GATEWAY_FLIGHT_REQUEST_TIMEOUT_SECS" {
            Some("45".to_string())
        } else {
            None
        }
    });
    assert_eq!(timeout, 45);
}

#[test]
fn test_gateway_flight_grpc_web_defaults_to_disabled() {
    assert!(!gateway_flight_grpc_web_enabled_with_lookup(&|_| None));
}

#[test]
fn test_gateway_flight_grpc_web_accepts_false_override() {
    let enabled = gateway_flight_grpc_web_enabled_with_lookup(&|key| {
        if key == "XIUXIAN_WENDAO_GATEWAY_FLIGHT_GRPC_WEB_ENABLED" {
            Some("off".to_string())
        } else {
            None
        }
    });
    assert!(!enabled);
}

#[test]
fn test_gateway_bearer_token_accepts_non_empty_override() {
    let token = gateway_bearer_token_with_lookup(&|key| {
        if key == "XIUXIAN_WENDAO_GATEWAY_BEARER_TOKEN" {
            Some("wd_runtime".to_string())
        } else {
            None
        }
    });
    assert_eq!(token.as_deref(), Some("wd_runtime"));
}

#[test]
fn test_get_gateway_runtime_from_config_reads_runtime_knobs() {
    let config_path = write_temp_gateway_config(
        r"
[gateway.runtime]
listen_backlog = 3072
studio_concurrency_limit = 72
studio_request_timeout_secs = 18
",
    );

    let runtime = get_gateway_runtime_from_config(Some(config_path.as_path()));
    remove_temp_gateway_config(config_path.as_path());

    assert_eq!(
        runtime,
        Some(GatewayRuntimeTomlConfig {
            listen_backlog: Some(3072),
            studio_concurrency_limit: Some(72),
            studio_request_timeout_secs: Some(18),
        })
    );
}

#[test]
fn test_gateway_startup_health_rejects_failed_dependencies() {
    let report = GatewayStartupHealthReport::new(vec![
        GatewayStartupDependencyCheck::connected("builtin_plugin_registry", "plugins=julia"),
        GatewayStartupDependencyCheck::failed("search_cache_valkey", "connection failed"),
    ]);

    let error = match ensure_gateway_startup_health(&report) {
        Ok(()) => panic!("failed startup health should abort gateway startup"),
        Err(error) => error,
    };

    assert_eq!(
        error.to_string(),
        "gateway startup health checks failed: search_cache_valkey (connection failed)"
    );
}

#[test]
fn test_gateway_startup_health_accepts_ready_dependencies() {
    let report = GatewayStartupHealthReport::new(vec![
        GatewayStartupDependencyCheck::connected("builtin_plugin_registry", "plugins=julia"),
        GatewayStartupDependencyCheck::connected(
            "search_cache_valkey",
            "url=redis://127.0.0.1:6379/0 ping=PONG",
        ),
        GatewayStartupDependencyCheck::connected(
            "link_graph_cache_valkey",
            "url=redis://127.0.0.1:6379/0 ping=PONG key_prefix=xiuxian:link_graph",
        ),
    ]);

    if let Err(error) = ensure_gateway_startup_health(&report) {
        panic!("healthy startup dependencies should allow gateway startup: {error}");
    }
}

#[test]
fn test_build_plugin_registry_bootstraps_builtin_plugins() {
    let registry = bootstrap_builtin_registry();
    assert!(registry.plugin_ids().contains(&"julia"));
    assert!(registry.plugin_ids().contains(&"modelica"));
}
