use super::{GRAPH_HOPS, GRAPH_LIMIT};

const _: () = {
    assert!(
        GRAPH_LIMIT <= 12,
        "SearchStrategyFlow graph evidence should stay within the compact frontier budget"
    );
};

#[test]
fn graph_materialization_uses_direct_relation_evidence_budget() {
    assert_eq!(
        GRAPH_HOPS, 1,
        "SearchStrategyFlow materialization should prove direct relation evidence, not expand a two-hop neighborhood"
    );
}
