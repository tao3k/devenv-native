use super::search_flightsql_grpc_web_enabled_with_lookup;

#[test]
fn search_flightsql_grpc_web_defaults_to_disabled() {
    assert!(!search_flightsql_grpc_web_enabled_with_lookup(&|_| None));
}

#[test]
fn search_flightsql_grpc_web_accepts_explicit_override() {
    assert!(search_flightsql_grpc_web_enabled_with_lookup(
        &|key| match key {
            "XIUXIAN_WENDAO_SEARCH_FLIGHTSQL_GRPC_WEB_ENABLED" => Some("yes".to_string()),
            _ => None,
        }
    ));
    assert!(!search_flightsql_grpc_web_enabled_with_lookup(
        &|key| match key {
            "XIUXIAN_WENDAO_SEARCH_FLIGHTSQL_GRPC_WEB_ENABLED" => Some("off".to_string()),
            _ => None,
        }
    ));
}
