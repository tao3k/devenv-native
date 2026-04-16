use crate::link_graph::runtime_config::resolve_link_graph_coactivation_runtime;
use xiuxian_wendao_runtime::config::{
    DEFAULT_LINK_GRAPH_COACTIVATION_HOP_DECAY_SCALE, DEFAULT_LINK_GRAPH_COACTIVATION_MAX_HOPS,
    DEFAULT_LINK_GRAPH_COACTIVATION_MAX_NEIGHBORS_PER_DIRECTION,
    DEFAULT_LINK_GRAPH_COACTIVATION_TOUCH_QUEUE_DEPTH,
};

#[test]
fn test_coactivation_touch_queue_depth_default() {
    let runtime = resolve_link_graph_coactivation_runtime();
    assert_eq!(runtime.max_hops, DEFAULT_LINK_GRAPH_COACTIVATION_MAX_HOPS);
    assert_eq!(
        runtime.max_total_propagations,
        DEFAULT_LINK_GRAPH_COACTIVATION_MAX_NEIGHBORS_PER_DIRECTION.saturating_mul(2)
    );
    assert!(
        (runtime.hop_decay_scale - DEFAULT_LINK_GRAPH_COACTIVATION_HOP_DECAY_SCALE).abs()
            <= f64::EPSILON,
        "unexpected hop_decay_scale: {}",
        runtime.hop_decay_scale
    );
    assert_eq!(
        runtime.touch_queue_depth,
        DEFAULT_LINK_GRAPH_COACTIVATION_TOUCH_QUEUE_DEPTH
    );
}
