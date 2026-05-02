//! Graph-neighbors route contract and metadata validation.

/// Canonical graph-neighbors node identifier metadata header for Wendao Flight
/// requests.
pub const WENDAO_GRAPH_NODE_ID_HEADER: &str = "x-wendao-graph-node-id";
/// Canonical graph-neighbors direction metadata header for Wendao Flight
/// requests.
pub const WENDAO_GRAPH_DIRECTION_HEADER: &str = "x-wendao-graph-direction";
/// Canonical graph-neighbors hop-limit metadata header for Wendao Flight
/// requests.
pub const WENDAO_GRAPH_HOPS_HEADER: &str = "x-wendao-graph-hops";
/// Canonical graph-neighbors result-limit metadata header for Wendao Flight
/// requests.
pub const WENDAO_GRAPH_LIMIT_HEADER: &str = "x-wendao-graph-limit";
/// Stable route for the graph-neighbors contract.
pub const GRAPH_NEIGHBORS_ROUTE: &str = "/graph/neighbors";
/// Stable default hop distance for graph-neighbors requests.
pub const GRAPH_NEIGHBORS_DEFAULT_HOPS: usize = 2;
/// Stable default result limit for graph-neighbors requests.
pub const GRAPH_NEIGHBORS_DEFAULT_LIMIT: usize = 50;
const GRAPH_NEIGHBORS_MAX_HOPS: usize = 8;
const GRAPH_NEIGHBORS_MAX_LIMIT: usize = 300;

/// Validate and normalize the stable graph-neighbors request contract.
///
/// # Errors
///
/// Returns an error when the requested node identifier is blank.
pub fn validate_graph_neighbors_request(
    node_id: &str,
    direction: Option<&str>,
    hops: Option<usize>,
    limit: Option<usize>,
) -> Result<(String, String, usize, usize), String> {
    let normalized_node_id = node_id.trim();
    if normalized_node_id.is_empty() {
        return Err("graph neighbors requires a non-empty node id".to_string());
    }

    let normalized_direction = match direction
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("incoming") => "incoming",
        Some("outgoing") => "outgoing",
        _ => "both",
    };
    let normalized_hops = hops
        .unwrap_or(GRAPH_NEIGHBORS_DEFAULT_HOPS)
        .clamp(1, GRAPH_NEIGHBORS_MAX_HOPS);
    let normalized_limit = limit
        .unwrap_or(GRAPH_NEIGHBORS_DEFAULT_LIMIT)
        .clamp(1, GRAPH_NEIGHBORS_MAX_LIMIT);

    Ok((
        normalized_node_id.to_string(),
        normalized_direction.to_string(),
        normalized_hops,
        normalized_limit,
    ))
}
