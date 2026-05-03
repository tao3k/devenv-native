use super::{
    GatewayRuntimeTomlConfig, gateway_bearer_token_with_lookup,
    gateway_flight_concurrency_limit_with_lookup, gateway_flight_grpc_web_enabled_with_lookup,
    gateway_flight_request_timeout_secs_with_lookup, gateway_listen_backlog_with_lookup,
    gateway_studio_concurrency_limit_with_lookup, gateway_studio_request_timeout_secs_with_lookup,
};

#[test]
fn gateway_runtime_knobs_prefer_toml_over_env() {
    let runtime = Some(GatewayRuntimeTomlConfig {
        listen_backlog: Some(4096),
        studio_concurrency_limit: Some(96),
        studio_request_timeout_secs: Some(27),
    });

    assert_eq!(
        gateway_listen_backlog_with_lookup(runtime, &|_| Some("2048".to_string())),
        4096
    );
    assert_eq!(
        gateway_studio_concurrency_limit_with_lookup(
            runtime,
            &|_| Some("48".to_string()),
            Some(12)
        ),
        96
    );
    assert_eq!(
        gateway_studio_request_timeout_secs_with_lookup(runtime, &|_| Some("15".to_string())),
        27
    );
}

#[test]
fn gateway_runtime_knobs_fall_back_to_env_when_toml_is_missing() {
    let runtime = None;

    assert_eq!(
        gateway_listen_backlog_with_lookup(runtime, &|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_LISTEN_BACKLOG" => Some("3072".to_string()),
            _ => None,
        }),
        3072
    );
    assert_eq!(
        gateway_studio_concurrency_limit_with_lookup(
            runtime,
            &|key| match key {
                "XIUXIAN_WENDAO_GATEWAY_STUDIO_CONCURRENCY_LIMIT" => Some("72".to_string()),
                _ => None,
            },
            Some(12)
        ),
        72
    );
    assert_eq!(
        gateway_studio_request_timeout_secs_with_lookup(runtime, &|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_STUDIO_REQUEST_TIMEOUT_SECS" => Some("22".to_string()),
            _ => None,
        }),
        22
    );
}

#[test]
fn gateway_flight_runtime_knobs_default_to_studio_budget() {
    assert_eq!(
        gateway_flight_concurrency_limit_with_lookup(64, &|_| None),
        64
    );
    assert_eq!(
        gateway_flight_request_timeout_secs_with_lookup(22, &|_| None),
        22
    );
}

#[test]
fn gateway_flight_runtime_knobs_accept_env_overrides() {
    assert_eq!(
        gateway_flight_concurrency_limit_with_lookup(64, &|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_FLIGHT_CONCURRENCY_LIMIT" => Some("24".to_string()),
            _ => None,
        }),
        24
    );
    assert_eq!(
        gateway_flight_request_timeout_secs_with_lookup(22, &|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_FLIGHT_REQUEST_TIMEOUT_SECS" => Some("45".to_string()),
            _ => None,
        }),
        45
    );
}

#[test]
fn gateway_flight_runtime_knobs_clamp_invalid_or_out_of_range_values() {
    assert_eq!(
        gateway_flight_concurrency_limit_with_lookup(2, &|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_FLIGHT_CONCURRENCY_LIMIT" => Some("512".to_string()),
            _ => None,
        }),
        128
    );
    assert_eq!(
        gateway_flight_request_timeout_secs_with_lookup(2, &|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_FLIGHT_REQUEST_TIMEOUT_SECS" => Some("0".to_string()),
            _ => None,
        }),
        5
    );
}

#[test]
fn gateway_flight_grpc_web_defaults_to_disabled() {
    assert!(!gateway_flight_grpc_web_enabled_with_lookup(&|_| None));
}

#[test]
fn gateway_flight_grpc_web_accepts_env_override() {
    assert!(!gateway_flight_grpc_web_enabled_with_lookup(
        &|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_FLIGHT_GRPC_WEB_ENABLED" => Some("false".to_string()),
            _ => None,
        }
    ));
    assert!(gateway_flight_grpc_web_enabled_with_lookup(
        &|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_FLIGHT_GRPC_WEB_ENABLED" => Some("yes".to_string()),
            _ => None,
        }
    ));
}

#[test]
fn gateway_flight_grpc_web_ignores_invalid_env_override() {
    assert!(!gateway_flight_grpc_web_enabled_with_lookup(
        &|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_FLIGHT_GRPC_WEB_ENABLED" => Some("sometimes".to_string()),
            _ => None,
        }
    ));
}

#[test]
fn gateway_bearer_token_defaults_to_disabled() {
    assert!(gateway_bearer_token_with_lookup(&|_| None).is_none());
}

#[test]
fn gateway_bearer_token_trims_non_empty_env_value() {
    assert_eq!(
        gateway_bearer_token_with_lookup(&|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_BEARER_TOKEN" => Some("  wd_test  ".to_string()),
            _ => None,
        })
        .as_deref(),
        Some("wd_test")
    );
}

#[test]
fn gateway_bearer_token_ignores_blank_env_value() {
    assert!(
        gateway_bearer_token_with_lookup(&|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_BEARER_TOKEN" => Some("   ".to_string()),
            _ => None,
        })
        .is_none()
    );
}

#[test]
fn gateway_runtime_knobs_clamp_invalid_or_out_of_range_values() {
    let runtime = Some(GatewayRuntimeTomlConfig {
        listen_backlog: Some(32),
        studio_concurrency_limit: Some(512),
        studio_request_timeout_secs: Some(1),
    });

    assert_eq!(
        gateway_listen_backlog_with_lookup(runtime, &|_| Some("0".to_string())),
        128
    );
    assert_eq!(
        gateway_studio_concurrency_limit_with_lookup(
            runtime,
            &|_| Some("bogus".to_string()),
            Some(12)
        ),
        128
    );
    assert_eq!(
        gateway_studio_request_timeout_secs_with_lookup(runtime, &|_| Some("0".to_string())),
        5
    );
}
