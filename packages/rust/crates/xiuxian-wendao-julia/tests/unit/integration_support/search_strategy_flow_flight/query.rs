use super::candidate_discovery_queries;

#[test]
fn candidate_discovery_queries_prioritize_route_scoped_attempts() {
    let attempts = candidate_discovery_queries(
        "find how SearchStrategyFlow uses ownership boundary validation path PageIndex and LinkGraph relation paths",
    );

    let first_required_attempts = attempts
        .iter()
        .take(4)
        .map(|attempt| (attempt.query.as_str(), attempt.path_prefix.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        first_required_attempts,
        vec![
            (
                "SearchStrategyFlow",
                "packages/rust/crates/xiuxian-wendao-julia/README.md"
            ),
            ("ownership boundary", "docs/rfcs"),
            ("validation path", "docs/testing"),
            (
                "Search Strategy Flow Link Graph",
                "packages/rust/crates/xiuxian-wendao-julia/tests/unit/integration_support/wendaograph/search_strategy"
            ),
        ],
        "required-evidence attempts should be front-loaded before broad recall attempts"
    );

    let broad_attempt = position_matching(
        &attempts,
        |attempt| {
            attempt.query.starts_with("find how SearchStrategyFlow")
                && attempt.path_prefix.is_empty()
        },
        "broad intent attempt should exist",
    );
    let route_attempt = position_matching(
        &attempts,
        |attempt| attempt.path_prefix == "docs/30_search_strategy",
        "SearchStrategyFlow docs route attempt should exist",
    );
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
    let relation_attempt = position_matching(
        &attempts,
        |attempt| {
            attempt.query == "Search Strategy Flow Link Graph"
                && attempt.path_prefix
                    == "packages/rust/crates/xiuxian-wendao-julia/tests/unit/integration_support/wendaograph/search_strategy"
        },
        "relation path scoped attempt should exist",
    );
    assert!(
        relation_attempt < broad_attempt,
        "relation path scoped attempts must run before broad repo search attempts"
    );
}

#[test]
fn candidate_discovery_queries_include_relation_attempt_for_required_evidence_intent() {
    let attempts = candidate_discovery_queries(
        "find the SearchStrategyFlow ownership boundary and validation path",
    );

    let first_required_attempts = attempts
        .iter()
        .take(4)
        .map(|attempt| (attempt.query.as_str(), attempt.path_prefix.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        first_required_attempts,
        vec![
            (
                "SearchStrategyFlow",
                "packages/rust/crates/xiuxian-wendao-julia/README.md"
            ),
            ("ownership boundary", "docs/rfcs"),
            ("validation path", "docs/testing"),
            (
                "Search Strategy Flow Link Graph",
                "packages/rust/crates/xiuxian-wendao-julia/tests/unit/integration_support/wendaograph/search_strategy"
            ),
        ],
        "SearchStrategyFlow authority/validation intents still require an early relation path attempt"
    );
}

fn position_matching<T>(
    items: &[T],
    predicate: impl Fn(&T) -> bool,
    missing_message: &str,
) -> usize {
    for (index, item) in items.iter().enumerate() {
        if predicate(item) {
            return index;
        }
    }
    panic!("{missing_message}");
}
