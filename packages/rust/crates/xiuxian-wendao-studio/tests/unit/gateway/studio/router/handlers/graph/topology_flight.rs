use crate::studio::router::handlers::graph::topology_flight::topology_3d_response_batch;
use xiuxian_wendao::search::contracts::{
    Topology3dPayload, TopologyCluster, TopologyLink, TopologyNode,
};

#[test]
fn topology_3d_response_batch_preserves_row_kinds() {
    let batch = topology_3d_response_batch(&Topology3dPayload {
        nodes: vec![TopologyNode {
            id: "kernel/docs/alpha.md".to_string(),
            name: "alpha".to_string(),
            node_type: "doc".to_string(),
            position: [1.0, 2.0, 3.0],
            cluster_id: Some("kernel".to_string()),
        }],
        links: vec![TopologyLink {
            from: "kernel/docs/alpha.md".to_string(),
            to: "kernel/docs/beta.md".to_string(),
            label: None,
        }],
        clusters: vec![TopologyCluster {
            id: "kernel".to_string(),
            name: "kernel".to_string(),
            centroid: [0.0, 0.0, 0.0],
            node_count: 1,
            color: "#abcdef".to_string(),
        }],
    })
    .unwrap_or_else(|error| panic!("build topology batch: {error}"));

    assert_eq!(batch.num_rows(), 3);
}
