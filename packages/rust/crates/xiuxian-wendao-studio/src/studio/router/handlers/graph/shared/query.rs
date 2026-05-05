use xiuxian_wendao::link_graph::LinkGraphDirection;

#[cfg(test)]
use serde::Deserialize;

const DEFAULT_GRAPH_HOPS: usize = 2;
const DEFAULT_GRAPH_LIMIT: usize = 50;
const MAX_GRAPH_HOPS: usize = 8;
const MAX_GRAPH_LIMIT: usize = 300;

/// Query parameters for graph-neighbor traversal.
#[cfg(test)]
#[derive(Debug, Deserialize)]
pub struct GraphNeighborsQuery {
    /// Optional direction override for neighbor traversal.
    pub direction: Option<String>,
    /// Optional maximum hop distance.
    pub hops: Option<usize>,
    /// Optional maximum number of neighbors to return.
    pub limit: Option<usize>,
}

pub(crate) fn parse_direction(direction: Option<&str>) -> LinkGraphDirection {
    match direction
        .map(str::trim)
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("incoming") => LinkGraphDirection::Incoming,
        Some("outgoing") => LinkGraphDirection::Outgoing,
        _ => LinkGraphDirection::Both,
    }
}

pub(crate) fn normalize_hops(hops: Option<usize>) -> usize {
    hops.unwrap_or(DEFAULT_GRAPH_HOPS).clamp(1, MAX_GRAPH_HOPS)
}

pub(crate) fn normalize_limit(limit: Option<usize>) -> usize {
    limit
        .unwrap_or(DEFAULT_GRAPH_LIMIT)
        .clamp(1, MAX_GRAPH_LIMIT)
}
