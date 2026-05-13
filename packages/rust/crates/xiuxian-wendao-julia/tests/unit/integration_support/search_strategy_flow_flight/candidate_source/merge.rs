use super::{candidate, merge_candidate_discovery_result};

#[test]
fn candidate_discovery_merge_keeps_best_later_gateway_scores() {
    let mut candidates = vec![candidate(
        "packages/python/wendao-knowledge-retrieval-benchmark/docs/profile_contract.md",
        "SearchStrategyFlow frontier rows",
        0.70,
    )];
    let better_candidate = candidate(
        "packages/python/wendao-knowledge-retrieval-benchmark/docs/profile_contract.md",
        "SearchStrategyFlow frontier rows",
        0.98,
    );

    merge_candidate_discovery_result(&mut candidates, better_candidate);

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].evidence_coverage, 0.98);
    assert_eq!(candidates[0].graph_score, 0.98);
    assert_eq!(candidates[0].authority_score, 0.98);
    assert_eq!(candidates[0].structural_score, 0.98);
    assert!(candidates[0].uncertainty < 0.31);
}
