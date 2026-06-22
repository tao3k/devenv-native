use super::{
    GatewayRuntimeTomlConfig, gateway_listen_backlog_with_lookup,
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
