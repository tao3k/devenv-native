use super::is_optional_link_graph_cache_failure;

#[test]
fn cache_connection_failure_is_optional_for_cli_index_build() {
    let error = "failed to connect valkey for link-graph cache: Connection refused (os error 61)";

    assert!(is_optional_link_graph_cache_failure(error));
}

#[test]
fn missing_cache_runtime_is_optional_for_cli_index_build() {
    let error = "link_graph cache valkey url is required (set link_graph.cache.valkey_url or XIUXIAN_WENDAO_LINK_GRAPH_VALKEY_URL)";

    assert!(is_optional_link_graph_cache_failure(error));
}

#[test]
fn invalid_cache_url_remains_authoritative() {
    let error = "invalid valkey url for link-graph cache: Redis URL did not parse";

    assert!(!is_optional_link_graph_cache_failure(error));
}

#[test]
fn index_build_errors_remain_authoritative() {
    let error = "failed to parse Markdown link graph";

    assert!(!is_optional_link_graph_cache_failure(error));
}
