use super::{
    RepoSearchAttempt, apply_exact_markdown_attempt_score_floor, candidate,
    candidate_from_exact_markdown_attempt,
};

#[test]
fn candidate_discovery_can_seed_exact_markdown_path_after_empty_gateway_rows() {
    let seed = candidate_from_exact_markdown_attempt(&RepoSearchAttempt {
        query: "Docling OCR shard provenance page index".to_owned(),
        path_prefix: "packages/rust/crates/xiuxian-wendao-attachments/README.md".to_owned(),
    })
    .unwrap_or_else(|| panic!("exact Markdown path should produce a bounded seed candidate"));

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
