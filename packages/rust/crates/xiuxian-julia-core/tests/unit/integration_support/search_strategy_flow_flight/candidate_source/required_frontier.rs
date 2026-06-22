use super::{
    CandidateDiscoveryRequiredEvidence, calibrate_candidate_discovery_scores, candidate,
    candidate_matches_validation_path_evidence, rank_candidate_discovery_results,
    retain_required_evidence_frontier, retain_unique_candidate_sources,
};

#[test]
fn candidate_discovery_trims_required_evidence_frontier_after_coverage() {
    let mut candidates = vec![
        candidate(
            "packages/rust/crates/xiuxian-julia-core/docs/index.md",
            "SearchStrategyFlow ownership boundary",
            0.86,
        ),
        candidate("docs/testing/README.md", "Default validation path", 0.84),
    ];
    candidates.extend((0..20).map(|index| {
        candidate(
            Box::leak(format!("docs/search-strategy-flow-extra-{index}.md").into_boxed_str()),
            "SearchStrategyFlow supporting evidence",
            0.98 - f64::from(index) / 100.0,
        )
    }));

    calibrate_candidate_discovery_scores(&mut candidates);
    rank_candidate_discovery_results(&mut candidates);
    retain_unique_candidate_sources(&mut candidates);
    retain_required_evidence_frontier(
        &mut candidates,
        CandidateDiscoveryRequiredEvidence {
            ownership_boundary: true,
            validation_path: true,
            relation_path: false,
        },
    );

    assert_eq!(candidates.len(), 4);
    assert!(candidates.iter().any(|candidate| {
        candidate.relative_path == "packages/rust/crates/xiuxian-julia-core/docs/index.md"
    }));
    assert!(
        candidates
            .iter()
            .any(|candidate| candidate.relative_path == "docs/testing/README.md")
    );
}

#[test]
fn candidate_discovery_treats_audit_markdown_as_validation_evidence() {
    let audit_candidate = candidate(
        "docs/rfcs/2026-05-04-polyglot-compute-orchestrator-audit.md",
        "Polyglot Compute Orchestrator Audit",
        0.82,
    );

    assert!(candidate_matches_validation_path_evidence(&audit_candidate));
}
