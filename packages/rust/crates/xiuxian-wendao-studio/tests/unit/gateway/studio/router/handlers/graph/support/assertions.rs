use xiuxian_wendao::search::contracts::GraphNeighborsResponse;

pub(crate) fn assert_graph_neighbors_include_path(response: &GraphNeighborsResponse, suffix: &str) {
    assert!(
        response.nodes.iter().any(|node| node.path.contains(suffix)),
        "expected {suffix} to be present in graph neighbors",
    );
}

pub(crate) fn assert_graph_neighbors_include_link_target(
    response: &GraphNeighborsResponse,
    suffix: &str,
) {
    assert!(
        response
            .links
            .iter()
            .any(|link| link.target.contains(suffix)),
        "expected graph links to point at {suffix}",
    );
}
