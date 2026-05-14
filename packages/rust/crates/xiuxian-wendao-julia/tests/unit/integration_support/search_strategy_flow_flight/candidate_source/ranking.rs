use super::{
    CandidateDiscoveryRankingMode, RepoSearchAttempt, apply_exact_markdown_attempt_score_floor,
    calibrate_candidate_discovery_scores, candidate, candidate_discovery_priority,
    candidate_from_exact_markdown_attempt, candidate_matches_relation_path_evidence,
    candidate_with_edges, rank_candidate_discovery_results,
    rank_candidate_discovery_results_for_intent, ranking_mode_for_intent,
    retain_unique_candidate_sources,
};

#[test]
fn candidate_discovery_ranking_prefers_authority_markdown_over_tests() {
    let mut candidates = vec![
        candidate(
            "packages/rust/crates/xiuxian-wendao-studio/tests/unit/gateway/studio/search/handlers/flight/repo_search/provider.rs",
            "Rust repo-search provider test",
            0.99,
        ),
        candidate(
            "packages/rust/crates/xiuxian-wendao-julia/docs/index.md",
            "SearchStrategyFlow Rust Flight Bridge",
            0.80,
        ),
        candidate("docs/00_vision/VS_OBSIDIAN.md", "Wendao LinkGraph", 0.98),
        candidate(
            "docs/rfcs/2026-04-05-wendao-memory-layer-boundaries-rfc.md",
            "Validated SearchStrategyFlow ownership boundary",
            0.82,
        ),
        candidate("wendao.toml", "Wendao Repository Configuration", 0.95),
    ];

    rank_candidate_discovery_results(&mut candidates);

    assert_eq!(
        candidates[0].relative_path,
        "packages/rust/crates/xiuxian-wendao-julia/docs/index.md"
    );
    assert_eq!(
        candidates[1].relative_path,
        "docs/rfcs/2026-04-05-wendao-memory-layer-boundaries-rfc.md"
    );
    assert!(
        candidates
            .iter()
            .position(|candidate| candidate.relative_path == "docs/00_vision/VS_OBSIDIAN.md")
            .is_some_and(|index| index > 1),
        "generic LinkGraph docs should not outrank owner or policy authority docs"
    );
    assert!(
        candidates
            .last()
            .is_some_and(|candidate| candidate.relative_path.contains("/tests/")),
        "test paths should be retained but ranked after authority docs"
    );
}

#[test]
fn candidate_discovery_corpus_intent_keeps_gateway_scores_ahead_of_static_authority() {
    let mut candidates = vec![
        candidate(
            "packages/rust/crates/xiuxian-wendao-julia/docs/index.md",
            "SearchStrategyFlow Rust Flight Bridge",
            0.80,
        ),
        candidate(
            "packages/python/wendao-knowledge-retrieval-benchmark/docs/profile_contract.md",
            "Benchmark Profile Contract",
            0.98,
        ),
        candidate(
            "packages/rust/crates/xiuxian-wendao-julia/tests/unit/integration_support/wendaograph/search_strategy/candidate_discovery.rs",
            "SearchStrategyFlow LinkGraph relation path",
            0.99,
        ),
    ];

    rank_candidate_discovery_results_for_intent(
        &mut candidates,
        "locate the benchmark profile contract for SearchStrategyFlow corpus accuracy",
    );

    assert_eq!(
        ranking_mode_for_intent(
            "locate the benchmark profile contract for SearchStrategyFlow corpus accuracy"
        ),
        CandidateDiscoveryRankingMode::CorpusRecall
    );
    assert_eq!(
        candidates[0].relative_path,
        "packages/python/wendao-knowledge-retrieval-benchmark/docs/profile_contract.md"
    );
    assert!(
        candidates
            .last()
            .is_some_and(|candidate| candidate.relative_path.contains("/tests/")),
        "test paths should not outrank high-score Markdown corpus hits"
    );
}

#[test]
fn candidate_discovery_calibration_gives_owner_authority_a_score_signal() {
    let mut candidates = vec![
        candidate(
            "packages/rust/crates/xiuxian-wendao-julia/docs/index.md",
            "SearchStrategyFlow Rust Flight Bridge",
            0.80,
        ),
        candidate(
            "docs/rfcs/2026-04-05-wendao-memory-layer-boundaries-rfc.md",
            "Validated SearchStrategyFlow ownership boundary",
            0.82,
        ),
        candidate("docs/00_vision/VS_OBSIDIAN.md", "Wendao LinkGraph", 0.78),
    ];

    calibrate_candidate_discovery_scores(&mut candidates);

    assert!(candidates[0].authority_score > candidates[1].authority_score);
    assert!(candidates[1].authority_score > candidates[2].authority_score);
    assert!(candidates[0].uncertainty < candidates[2].uncertainty);
}

#[test]
fn candidate_discovery_ranking_keeps_relation_evidence_a_required_bucket() {
    let relation_candidate = candidate_with_edges(
        "packages/rust/crates/xiuxian-wendao-julia/tests/unit/integration_support/wendaograph/search_strategy/candidate_discovery.rs",
        "Search strategy flow LinkGraph path",
        0.80,
        &["link-graph", "relation"],
    );
    let generic_test_candidate = candidate(
        "packages/rust/crates/xiuxian-wendao-julia/tests/unit/integration_support/wendaograph/search_strategy/required_evidence.rs",
        "Find the SearchStrategyFlow ownership boundary and validation path",
        0.90,
    );

    assert!(
        candidate_discovery_priority(&relation_candidate)
            < candidate_discovery_priority(&generic_test_candidate),
        "required relation evidence must not be pruned behind generic test candidates"
    );
    assert!(candidate_matches_relation_path_evidence(
        &relation_candidate
    ));
    assert!(
        !candidate_matches_relation_path_evidence(&generic_test_candidate),
        "generic SearchStrategyFlow test hits must not satisfy relation_path"
    );
}

#[test]
fn candidate_discovery_keeps_one_best_candidate_per_source_path() {
    let mut candidates = vec![
        candidate(
            "packages/rust/crates/xiuxian-wendao-julia/docs/index.md",
            "SearchStrategyFlow Rust Flight Bridge",
            0.80,
        ),
        candidate(
            "packages/rust/crates/xiuxian-wendao-julia/docs/index.md",
            "Promote generic LinkGraph mentions over required evidence",
            0.80,
        ),
        candidate("docs/testing/README.md", "Default validation path", 0.80),
        candidate(
            "docs/rfcs/2026-03-26-wendao-query-engine-rfc.md",
            "Ownership boundary",
            0.80,
        ),
    ];

    calibrate_candidate_discovery_scores(&mut candidates);
    rank_candidate_discovery_results(&mut candidates);
    retain_unique_candidate_sources(&mut candidates);

    let index_count = candidates
        .iter()
        .filter(|candidate| {
            candidate.relative_path == "packages/rust/crates/xiuxian-wendao-julia/docs/index.md"
        })
        .count();
    assert_eq!(index_count, 1);
    assert!(
        candidates
            .iter()
            .any(|candidate| { candidate.relative_path == "docs/testing/README.md" })
    );
    assert!(candidates.iter().any(|candidate| {
        candidate.relative_path == "docs/rfcs/2026-03-26-wendao-query-engine-rfc.md"
    }));
}

#[test]
fn candidate_discovery_can_seed_exact_markdown_path_after_empty_gateway_rows() {
    let seed = candidate_from_exact_markdown_attempt(&RepoSearchAttempt {
        query: "Docling OCR shard provenance page index".to_owned(),
        path_prefix: "packages/rust/crates/xiuxian-wendao-attachments/README.md".to_owned(),
    })
    .expect("exact Markdown path should produce a bounded seed candidate");

    assert_eq!(
        seed.relative_path,
        "packages/rust/crates/xiuxian-wendao-attachments/README.md"
    );
    assert_eq!(seed.heading_anchor, "document");
    assert!(
        seed.title
            .contains("Docling OCR shard provenance page index")
    );
    assert!(seed.evidence_coverage >= 0.98);
    assert!(seed.uncertainty <= 0.04);
    assert!(
        seed.edge_kinds
            .iter()
            .any(|kind| kind == "intent-exact-markdown-seed")
    );

    assert!(
        candidate_from_exact_markdown_attempt(&RepoSearchAttempt {
            query: "Docling OCR shard provenance page index".to_owned(),
            path_prefix: "packages/rust/crates/xiuxian-wendao-attachments".to_owned(),
        })
        .is_none()
    );
    assert!(
        candidate_from_exact_markdown_attempt(&RepoSearchAttempt {
            query: "SearchStrategyFlow".to_owned(),
            path_prefix: "packages/rust/crates/xiuxian-wendao-julia/README.md".to_owned(),
        })
        .is_none(),
        "generic route probes should not become intent-exact Markdown seeds"
    );
}

#[test]
fn candidate_discovery_boosts_gateway_rows_from_exact_markdown_attempts() {
    let mut candidates = vec![candidate(
        "docs/rfcs/2026-05-04-polyglot-compute-orchestrator-rfc.md",
        "Polyglot compute orchestrator boundary",
        0.72,
    )];
    apply_exact_markdown_attempt_score_floor(
        &mut candidates,
        &RepoSearchAttempt {
            query: "polyglot compute orchestrator boundary calibration".to_owned(),
            path_prefix: "docs/rfcs/2026-05-04-polyglot-compute-orchestrator-rfc.md".to_owned(),
        },
    );

    let boosted = &candidates[0];
    assert!(boosted.evidence_coverage >= 0.98);
    assert!(boosted.graph_score >= 0.96);
    assert!(boosted.uncertainty <= 0.04);
    assert!(
        boosted
            .edge_kinds
            .iter()
            .any(|kind| kind == "intent-exact-markdown-seed")
    );
}

#[test]
fn candidate_discovery_does_not_boost_generic_route_markdown_probes() {
    let mut candidates = vec![candidate(
        "packages/rust/crates/xiuxian-wendao-julia/README.md",
        "SearchStrategyFlow Flight materialization",
        0.72,
    )];
    apply_exact_markdown_attempt_score_floor(
        &mut candidates,
        &RepoSearchAttempt {
            query: "SearchStrategyFlow".to_owned(),
            path_prefix: "packages/rust/crates/xiuxian-wendao-julia/README.md".to_owned(),
        },
    );

    let candidate = &candidates[0];
    assert!(candidate.evidence_coverage < 0.98);
    assert!(
        !candidate
            .edge_kinds
            .iter()
            .any(|kind| kind == "intent-exact-markdown-seed")
    );
}
