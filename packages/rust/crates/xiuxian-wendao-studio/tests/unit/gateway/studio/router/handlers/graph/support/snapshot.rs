use serde_json::json;

use crate::contracts::GraphNeighborsResponse;

pub(crate) fn graph_neighbors_snapshot_payload(
    response: GraphNeighborsResponse,
) -> serde_json::Value {
    json!({
        "center": {
            "distance": response.center.distance,
            "id": response.center.id,
            "isCenter": response.center.is_center,
            "label": response.center.label,
            "nodeType": response.center.node_type,
            "path": response.center.path,
        },
        "links": response.links.into_iter().map(|link| {
            json!({
                "direction": link.direction,
                "distance": link.distance,
                "source": link.source,
                "target": link.target,
            })
        }).collect::<Vec<_>>(),
        "nodes": response.nodes.into_iter().map(|node| {
            json!({
                "distance": node.distance,
                "id": node.id,
                "isCenter": node.is_center,
                "label": node.label,
                "nodeType": node.node_type,
                "path": node.path,
            })
        }).collect::<Vec<_>>(),
        "totalLinks": response.total_links,
        "totalNodes": response.total_nodes,
    })
}
