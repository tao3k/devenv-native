use super::{
    GRAPH_NEIGHBORS_DEFAULT_HOPS, GRAPH_NEIGHBORS_DEFAULT_LIMIT, validate_graph_neighbors_request,
};
use crate::transport::GraphNeighborsRequest;

#[test]
fn graph_neighbors_request_validation_accepts_canonical_request() {
    assert_eq!(
        validate_graph_neighbors_request(
            "kernel/docs/index.md",
            Some("outgoing"),
            Some(3),
            Some(25)
        ),
        Ok(GraphNeighborsRequest {
            node_id: "kernel/docs/index.md".to_string(),
            direction: "outgoing".to_string(),
            hops: 3,
            limit: 25,
        })
    );
}

#[test]
fn graph_neighbors_request_validation_normalizes_defaults_and_clamps_bounds() {
    assert_eq!(
        validate_graph_neighbors_request(
            "kernel/docs/index.md",
            Some("invalid"),
            Some(0),
            Some(999)
        ),
        Ok(GraphNeighborsRequest {
            node_id: "kernel/docs/index.md".to_string(),
            direction: "both".to_string(),
            hops: 1,
            limit: 300,
        })
    );
    assert_eq!(
        validate_graph_neighbors_request("kernel/docs/index.md", None, None, None),
        Ok(GraphNeighborsRequest {
            node_id: "kernel/docs/index.md".to_string(),
            direction: "both".to_string(),
            hops: GRAPH_NEIGHBORS_DEFAULT_HOPS,
            limit: GRAPH_NEIGHBORS_DEFAULT_LIMIT,
        })
    );
}

#[test]
fn graph_neighbors_request_validation_rejects_blank_node_id() {
    assert_eq!(
        validate_graph_neighbors_request("   ", Some("both"), Some(2), Some(20)),
        Err("graph neighbors requires a non-empty node id".to_string())
    );
}
