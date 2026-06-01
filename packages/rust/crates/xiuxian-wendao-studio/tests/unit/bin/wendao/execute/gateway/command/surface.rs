use super::{
    gateway_flight_rate_limit_per_second_with_lookup,
    gateway_flight_stream_budget_bytes_with_lookup,
    gateway_https_rate_limit_per_second_with_lookup, gateway_https_stream_budget_bytes_with_lookup,
};

#[test]
fn gateway_surface_rate_limits_default_from_surface_concurrency() {
    assert_eq!(
        gateway_https_rate_limit_per_second_with_lookup(64, &|_| None),
        512
    );
    assert_eq!(
        gateway_flight_rate_limit_per_second_with_lookup(24, &|_| None),
        192
    );
}

#[test]
fn gateway_surface_rate_limits_accept_env_overrides() {
    assert_eq!(
        gateway_https_rate_limit_per_second_with_lookup(64, &|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_HTTPS_RATE_LIMIT_PER_SECOND" => Some("128".to_string()),
            _ => None,
        }),
        128
    );
    assert_eq!(
        gateway_flight_rate_limit_per_second_with_lookup(24, &|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_FLIGHT_RATE_LIMIT_PER_SECOND" => Some("48".to_string()),
            _ => None,
        }),
        48
    );
}

#[test]
fn gateway_surface_rate_limits_clamp_out_of_range_values() {
    assert_eq!(
        gateway_https_rate_limit_per_second_with_lookup(64, &|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_HTTPS_RATE_LIMIT_PER_SECOND" => Some("0".to_string()),
            _ => None,
        }),
        512
    );
    assert_eq!(
        gateway_https_rate_limit_per_second_with_lookup(64, &|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_HTTPS_RATE_LIMIT_PER_SECOND" => Some("999999".to_string()),
            _ => None,
        }),
        4096
    );
    assert_eq!(
        gateway_flight_rate_limit_per_second_with_lookup(24, &|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_FLIGHT_RATE_LIMIT_PER_SECOND" => Some("999999".to_string()),
            _ => None,
        }),
        2048
    );
}

#[test]
fn gateway_surface_stream_budgets_default_and_accept_env_overrides() {
    assert_eq!(
        gateway_https_stream_budget_bytes_with_lookup(&|_| None),
        64 * 1024 * 1024
    );
    assert_eq!(
        gateway_flight_stream_budget_bytes_with_lookup(&|_| None),
        1024 * 1024 * 1024
    );
    assert_eq!(
        gateway_https_stream_budget_bytes_with_lookup(&|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_HTTPS_STREAM_BUDGET_BYTES" => Some("1048576".to_string()),
            _ => None,
        }),
        1024 * 1024
    );
    assert_eq!(
        gateway_flight_stream_budget_bytes_with_lookup(&|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_FLIGHT_STREAM_BUDGET_BYTES" => Some("2097152".to_string()),
            _ => None,
        }),
        2 * 1024 * 1024
    );
}

#[test]
fn gateway_surface_stream_budgets_clamp_out_of_range_values() {
    assert_eq!(
        gateway_https_stream_budget_bytes_with_lookup(&|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_HTTPS_STREAM_BUDGET_BYTES" => Some("1".to_string()),
            _ => None,
        }),
        1024
    );
    assert_eq!(
        gateway_https_stream_budget_bytes_with_lookup(&|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_HTTPS_STREAM_BUDGET_BYTES" => {
                Some("999999999999".to_string())
            }
            _ => None,
        }),
        512 * 1024 * 1024
    );
    assert_eq!(
        gateway_flight_stream_budget_bytes_with_lookup(&|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_FLIGHT_STREAM_BUDGET_BYTES" => {
                Some("999999999999".to_string())
            }
            _ => None,
        }),
        4 * 1024 * 1024 * 1024
    );
}
