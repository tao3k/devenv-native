use super::{candidate, should_stop_candidate_discovery};

#[test]
fn candidate_discovery_early_stop_waits_for_attempt_floor_and_source_budget() {
    let candidates = (0..32)
        .map(|index| {
            candidate(
                if index == 0 {
                    "packages/rust/crates/xiuxian-wendao-julia/tests/unit/integration_support/wendaograph/search_strategy/candidate_discovery.rs"
                } else {
                    Box::leak(format!("docs/{index}.md").into_boxed_str())
                },
                if index == 0 {
                    "SearchStrategyFlow LinkGraph relation path"
                } else {
                    "SearchStrategyFlow evidence"
                },
                0.80,
            )
        })
        .collect::<Vec<_>>();

    assert!(!should_stop_candidate_discovery(23, &candidates));
    assert!(should_stop_candidate_discovery(24, &candidates));
}

#[test]
fn candidate_discovery_early_stop_accepts_required_evidence_after_scoped_attempts() {
    let mut candidates = vec![
        candidate(
            "packages/rust/crates/xiuxian-wendao-julia/README.md",
            "SearchStrategyFlow ownership boundary",
            0.91,
        ),
        candidate("docs/testing/README.md", "Default validation path", 0.88),
        candidate(
            "packages/rust/crates/xiuxian-wendao-julia/tests/unit/integration_support/wendaograph/search_strategy/candidate_discovery.rs",
            "Search Strategy Flow Link Graph Python Julia TOML",
            0.84,
        ),
    ];
    candidates.extend((0..13).map(|index| {
        candidate(
            Box::leak(format!("docs/search-strategy-flow-extra-{index}.md").into_boxed_str()),
            "SearchStrategyFlow supporting evidence",
            0.80,
        )
    }));

    assert!(!should_stop_candidate_discovery(11, &candidates));
    assert!(should_stop_candidate_discovery(12, &candidates));
}

#[test]
fn candidate_discovery_required_evidence_stop_keeps_missing_relation_open() {
    let candidates = vec![
        candidate(
            "packages/rust/crates/xiuxian-wendao-julia/README.md",
            "SearchStrategyFlow ownership boundary",
            0.91,
        ),
        candidate("docs/testing/README.md", "Default validation path", 0.88),
    ];

    assert!(!should_stop_candidate_discovery(12, &candidates));
}

#[test]
fn candidate_discovery_max_candidate_stop_keeps_missing_relation_open() {
    let candidates = (0..32)
        .map(|index| {
            candidate(
                Box::leak(format!("docs/{index}.md").into_boxed_str()),
                "SearchStrategyFlow evidence",
                0.80,
            )
        })
        .collect::<Vec<_>>();

    assert!(!should_stop_candidate_discovery(24, &candidates));
}

#[test]
fn candidate_discovery_early_stop_counts_unique_source_paths() {
    let candidates = (0..32)
        .map(|index| {
            candidate(
                if index % 2 == 0 {
                    "docs/repeated.md"
                } else {
                    "docs/other.md"
                },
                "SearchStrategyFlow evidence",
                0.80,
            )
        })
        .collect::<Vec<_>>();

    assert!(!should_stop_candidate_discovery(24, &candidates));
}
