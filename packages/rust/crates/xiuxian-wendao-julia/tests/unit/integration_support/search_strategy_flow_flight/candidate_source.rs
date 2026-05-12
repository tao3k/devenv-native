use crate::integration_support::search_strategy_flow_candidates::SearchStrategyFlowCandidateInput;

use super::{
    calibrate_candidate_discovery_scores, candidate_discovery_attempt_receipt,
    rank_candidate_discovery_results, retain_unique_candidate_sources,
    should_stop_candidate_discovery,
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
fn candidate_discovery_attempt_receipt_records_elapsed_time() {
    let receipt = candidate_discovery_attempt_receipt("search strategy", "docs", 3, 42);

    assert_eq!(receipt.get("rowCount"), Some(&serde_json::json!(3)));
    assert_eq!(receipt.get("elapsedMs"), Some(&serde_json::json!(42)));
}

#[test]
fn candidate_discovery_early_stop_waits_for_attempt_floor_and_source_budget() {
    let candidates = (0..12)
        .map(|index| {
            candidate(
                Box::leak(format!("docs/{index}.md").into_boxed_str()),
                "SearchStrategyFlow evidence",
                0.80,
            )
        })
        .collect::<Vec<_>>();

    assert!(!should_stop_candidate_discovery(19, &candidates));
    assert!(should_stop_candidate_discovery(20, &candidates));
}

#[test]
fn candidate_discovery_early_stop_counts_unique_source_paths() {
    let candidates = (0..12)
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

    assert!(!should_stop_candidate_discovery(20, &candidates));
}

fn candidate(
    relative_path: &'static str,
    title: &'static str,
    score: f64,
) -> SearchStrategyFlowCandidateInput {
    SearchStrategyFlowCandidateInput {
        relative_path: relative_path.to_owned(),
        heading_anchor: title.to_ascii_lowercase().replace(' ', "-"),
        title: title.to_owned(),
        line_start: 1,
        line_end: 8,
        context_cost: 8,
        evidence_coverage: score,
        graph_score: score,
        authority_score: score,
        structural_score: score,
        uncertainty: 1.0 - score,
        blocked: false,
        edge_kinds: Vec::new(),
    }
}
