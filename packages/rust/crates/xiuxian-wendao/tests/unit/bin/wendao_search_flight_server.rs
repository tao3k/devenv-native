use super::search_flight_grpc_web_enabled_with_lookup;

#[test]
fn search_flight_grpc_web_defaults_to_disabled() {
    assert!(!search_flight_grpc_web_enabled_with_lookup(&|_| None));
}

#[test]
fn search_flight_grpc_web_accepts_explicit_override() {
    assert!(search_flight_grpc_web_enabled_with_lookup(
        &|key| match key {
            "XIUXIAN_WENDAO_SEARCH_FLIGHT_GRPC_WEB_ENABLED" => Some("true".to_string()),
            _ => None,
        }
    ));
    assert!(!search_flight_grpc_web_enabled_with_lookup(
        &|key| match key {
            "XIUXIAN_WENDAO_SEARCH_FLIGHT_GRPC_WEB_ENABLED" => Some("false".to_string()),
            _ => None,
        }
    ));
}
