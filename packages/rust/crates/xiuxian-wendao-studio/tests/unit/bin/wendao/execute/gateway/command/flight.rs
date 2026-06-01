use super::{
    gateway_flight_concurrency_limit_with_lookup, gateway_flight_grpc_web_enabled_with_lookup,
    gateway_flight_request_timeout_secs_with_lookup,
};

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
    assert_eq!(
        gateway_flight_request_timeout_secs_with_lookup(22, &|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_FLIGHT_REQUEST_TIMEOUT_SECS" => Some("1200".to_string()),
            _ => None,
        }),
        900
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
