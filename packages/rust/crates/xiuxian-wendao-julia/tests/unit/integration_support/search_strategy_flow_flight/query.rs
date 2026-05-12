use super::candidate_discovery_queries;

#[test]
fn candidate_discovery_queries_prioritize_route_scoped_attempts() {
    let attempts = candidate_discovery_queries(
        "find how SearchStrategyFlow uses ownership boundary validation path PageIndex and LinkGraph relation paths",
    );

    let broad_attempt = attempts
        .iter()
        .position(|attempt| {
            attempt.query.starts_with("find how SearchStrategyFlow")
                && attempt.path_prefix.is_empty()
        })
        .expect("broad intent attempt should exist");
    let route_attempt = attempts
        .iter()
        .position(|attempt| attempt.path_prefix == "docs/30_search_strategy")
        .expect("SearchStrategyFlow docs route attempt should exist");
    assert!(
        route_attempt < broad_attempt,
        "route-scoped docs attempts must run before broad repo search attempts"
    );
    assert!(attempts.iter().any(|attempt| {
        attempt.query == "SearchStrategyFlow"
            && attempt.path_prefix == "packages/rust/crates/xiuxian-wendao-julia/docs"
    }));
    assert!(attempts.iter().any(|attempt| {
        attempt.query == "ownership boundary" && attempt.path_prefix == "docs/rfcs"
    }));
    assert!(attempts.iter().any(|attempt| {
        attempt.query == "validation path" && attempt.path_prefix == "docs/testing"
    }));
}
