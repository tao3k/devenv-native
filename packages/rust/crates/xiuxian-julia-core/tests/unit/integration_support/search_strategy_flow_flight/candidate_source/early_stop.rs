use super::{CandidateDiscoveryRequiredEvidence, candidate, should_stop_candidate_discovery};

#[test]
fn candidate_discovery_early_stop_waits_for_attempt_floor_and_source_budget() {
    let candidates = (0..32)
        .map(|index| {
            candidate(
                if index == 0 {
                    "packages/rust/crates/xiuxian-julia-core/tests/unit/integration_support/wendaograph/search_strategy/candidate_discovery.rs"
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

    assert!(!should_stop_candidate_discovery(
        23,
        &candidates,
        CandidateDiscoveryRequiredEvidence::all(),
    ));
    assert!(should_stop_candidate_discovery(
        24,
        &candidates,
        CandidateDiscoveryRequiredEvidence::all(),
    ));
}

#[test]
fn candidate_discovery_early_stop_accepts_intent_required_evidence_after_scoped_attempts() {
    let mut candidates = vec![
        candidate(
            "packages/rust/crates/xiuxian-julia-core/README.md",
            "SearchStrategyFlow ownership boundary",
            0.91,
        ),
        candidate("docs/testing/README.md", "Default validation path", 0.88),
        candidate(
            "packages/rust/crates/xiuxian-julia-core/tests/unit/integration_support/wendaograph/search_strategy/candidate_discovery.rs",
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

    let required = CandidateDiscoveryRequiredEvidence::all();
    assert!(!should_stop_candidate_discovery(5, &candidates, required));
    assert!(should_stop_candidate_discovery(6, &candidates, required));
}

#[test]
fn candidate_discovery_required_evidence_stop_keeps_intent_missing_relation_open() {
    let candidates = vec![
        candidate(
            "packages/rust/crates/xiuxian-julia-core/README.md",
            "SearchStrategyFlow ownership boundary",
            0.91,
        ),
        candidate("docs/testing/README.md", "Default validation path", 0.88),
        candidate(
            "docs/search-strategy-flow-support-a.md",
            "SearchStrategyFlow supporting evidence",
            0.80,
        ),
        candidate(
            "docs/search-strategy-flow-support-b.md",
            "SearchStrategyFlow supporting evidence",
            0.79,
        ),
    ];

    assert!(!should_stop_candidate_discovery(
        6,
        &candidates,
        CandidateDiscoveryRequiredEvidence::all(),
    ));
    assert!(should_stop_candidate_discovery(
        6,
        &candidates,
        CandidateDiscoveryRequiredEvidence {
            ownership_boundary: true,
            validation_path: true,
            relation_path: false,
        },
    ));
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

    assert!(!should_stop_candidate_discovery(
        24,
        &candidates,
        CandidateDiscoveryRequiredEvidence::all(),
    ));
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

    assert!(!should_stop_candidate_discovery(
        24,
        &candidates,
        CandidateDiscoveryRequiredEvidence::all(),
    ));
}

#[test]
fn candidate_discovery_early_stop_recognizes_governance_authority_scope() {
    let candidates = vec![
        candidate("AGENTS.md", "Modularity debt warning cleanup", 0.95),
        candidate("docs/standards/AUDITOR_CODEX.md", "Hyper Modularity", 0.93),
        candidate("docs/support-a.md", "Governance support", 0.80),
        candidate("docs/support-b.md", "Warning support", 0.80),
    ];
    let required = CandidateDiscoveryRequiredEvidence::from_intent(
        "Find Markdown governance rules for modularity debt and warning cleanup",
    );

    assert!(should_stop_candidate_discovery(6, &candidates, required));
}

#[test]
fn materialization_intent_requires_validation_path_evidence() {
    let required = CandidateDiscoveryRequiredEvidence::from_intent(
        "Find the Markdown package docs that define Studio ownership of SearchStrategyFlow Flight materialization.",
    );

    assert!(required.ownership_boundary);
    assert!(required.validation_path);
}
