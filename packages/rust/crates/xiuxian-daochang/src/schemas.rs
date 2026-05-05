//! Agent and Router schemas bundled as static strings.

/// Agent route-trace schema.
pub const AGENT_ROUTE_TRACE_V1: &str =
    include_str!("../resources/xiuxian.runtime.route_trace.v1.schema.json");
/// Agent server-info schema.
pub const AGENT_SERVER_INFO_V1: &str =
    include_str!("../resources/xiuxian.runtime.server_info.v1.schema.json");
/// Agent session-closed schema.
pub const AGENT_SESSION_CLOSED_V1: &str =
    include_str!("../resources/xiuxian.runtime.session_closed.v1.schema.json");
/// Router route-test schema.
pub const ROUTER_ROUTE_TEST_V1: &str =
    include_str!("../resources/xiuxian.router.route_test.v1.schema.json");
/// Router routing-search schema.
pub const ROUTER_ROUTING_SEARCH_V1: &str =
    include_str!("../resources/xiuxian.router.routing_search.v1.schema.json");
/// Router search-config schema.
pub const ROUTER_SEARCH_CONFIG_V1: &str =
    include_str!("../resources/xiuxian.router.search_config.v1.schema.json");
/// Discover match schema.
pub const DISCOVER_MATCH_V1: &str =
    include_str!("../resources/xiuxian.discover.match.v1.schema.json");

/// Returns a bundled schema by canonical schema name.
#[must_use]
pub fn get_schema(name: &str) -> Option<&'static str> {
    match name {
        "xiuxian.runtime.route_trace.v1" => Some(AGENT_ROUTE_TRACE_V1),
        "xiuxian.runtime.server_info.v1" => Some(AGENT_SERVER_INFO_V1),
        "xiuxian.runtime.session_closed.v1" => Some(AGENT_SESSION_CLOSED_V1),
        "xiuxian.router.route_test.v1" => Some(ROUTER_ROUTE_TEST_V1),
        "xiuxian.router.routing_search.v1" => Some(ROUTER_ROUTING_SEARCH_V1),
        "xiuxian.router.search_config.v1" => Some(ROUTER_SEARCH_CONFIG_V1),
        "xiuxian.discover.match.v1" => Some(DISCOVER_MATCH_V1),
        _ => None,
    }
}
