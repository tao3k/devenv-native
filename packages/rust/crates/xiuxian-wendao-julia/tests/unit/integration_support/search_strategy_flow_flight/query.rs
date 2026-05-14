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

#[test]
fn candidate_discovery_queries_start_with_full_intent_for_corpus_recall() {
    let attempts = candidate_discovery_queries(
        "locate the benchmark profile contract for SearchStrategyFlow corpus accuracy",
    );

    assert_eq!(
        attempts
            .first()
            .map(|attempt| (attempt.query.as_str(), attempt.path_prefix.as_str())),
        Some((
            "locate the benchmark profile contract for SearchStrategyFlow corpus accuracy",
            "",
        )),
        "corpus recall intents should let Gateway retrieval score the specific intent before generic SearchStrategyFlow probes",
    );
    assert!(
        attempts.iter().position(|attempt| {
            attempt.query == "SearchStrategyFlow"
                && attempt.path_prefix == "packages/rust/crates/xiuxian-wendao-julia/README.md"
        }) > Some(0),
        "generic SearchStrategyFlow route probes should remain available after specific intent attempts"
    );
}

#[test]
fn candidate_discovery_queries_include_policy_package_and_roadmap_surfaces() {
    let governance_attempts = candidate_discovery_queries(
        "Find the Markdown governance rules for modularity, debt closure, and warning cleanup in this repository.",
    );
    assert!(governance_attempts.iter().any(|attempt| {
        attempt.query == "modularity debt warning cleanup" && attempt.path_prefix == "AGENTS.md"
    }));
    assert!(governance_attempts.iter().any(|attempt| {
        attempt.query == "Hyper Modularity"
            && attempt.path_prefix == "docs/standards/AUDITOR_CODEX.md"
    }));
    assert!(governance_attempts.iter().any(|attempt| {
        attempt.query == "modularity debt warning cleanup"
            && attempt.path_prefix == "docs/standards"
    }));

    let docling_attempts = candidate_discovery_queries(
        "Find the Markdown package documentation for attachment-side Docling structure, OCR shard provenance, and page index ordering.",
    );
    assert!(docling_attempts.iter().any(|attempt| {
        attempt.query == "Docling OCR shard provenance page index"
            && attempt.path_prefix == "packages/rust/crates/xiuxian-wendao-attachments/README.md"
    }));
    assert!(docling_attempts.iter().any(|attempt| {
        attempt.query == "Docling OCR shard provenance page index"
            && attempt.path_prefix == "packages/python/xiuxian-wendao-analyzer/README.md"
    }));
    assert!(
        !docling_attempts
            .iter()
            .any(|attempt| attempt.path_prefix
                == "packages/rust/crates/xiuxian-wendao/docs/06_roadmap/403_document_projection_and_retrieval_enhancement.md"),
        "attachment package documentation should not pull the global page-index roadmap"
    );

    let projected_page_attempts = candidate_discovery_queries(
        "Find the Markdown roadmap explaining projected documentation pages, page index, and graph-enhanced retrieval.",
    );
    assert!(projected_page_attempts.iter().any(|attempt| {
        attempt.query == "projected documentation pages graph enhanced retrieval"
            && attempt.path_prefix == "packages/rust/crates/xiuxian-wendao/docs/06_roadmap"
    }));
    assert!(projected_page_attempts.iter().any(|attempt| {
        attempt.query == "projected documentation pages graph enhanced retrieval"
            && attempt.path_prefix
                == "packages/rust/crates/xiuxian-wendao/docs/06_roadmap/403_document_projection_and_retrieval_enhancement.md"
    }));

    let studio_attempts = candidate_discovery_queries(
        "Find the Markdown package docs that define Studio ownership of SearchStrategyFlow Flight materialization.",
    );
    assert!(studio_attempts.iter().any(|attempt| {
        attempt.query == "Studio SearchStrategyFlow Flight materialization ownership"
            && attempt.path_prefix == "packages/rust/crates/xiuxian-wendao-studio/README.md"
    }));
    assert!(studio_attempts.iter().any(|attempt| {
        attempt.query == "SearchStrategyFlow Flight materialization bridge"
            && attempt.path_prefix == "packages/rust/crates/xiuxian-wendao-julia/README.md"
    }));

    let query_engine_attempts = candidate_discovery_queries(
        "Find the RFC Markdown section that establishes the Wendao query engine ownership boundary and source authority.",
    );
    assert!(query_engine_attempts.iter().any(|attempt| {
        attempt.query == "Wendao query engine ownership boundary source authority"
            && attempt.path_prefix == "docs/rfcs/2026-03-26-wendao-query-engine-rfc.md"
    }));

    let testing_attempts = candidate_discovery_queries(
        "Find the Markdown validation path that explains local validation and CI test proof.",
    );
    assert!(testing_attempts.iter().any(|attempt| {
        attempt.query == "local validation CI test proof"
            && attempt.path_prefix == "docs/developer/testing.md"
    }));

    let memory_attempts = candidate_discovery_queries(
        "Find the Markdown RFC that places SearchStrategyFlow in the working-knowledge memory layer.",
    );
    assert!(memory_attempts.iter().any(|attempt| {
        attempt.query == "validated SearchStrategyFlow working knowledge memory layer"
            && attempt.path_prefix == "docs/rfcs/2026-04-05-wendao-memory-layer-boundaries-rfc.md"
    }));

    let benchmark_attempts = candidate_discovery_queries(
        "Find the Markdown benchmark profile contract for SearchStrategyFlow frontier rows and required evidence coverage.",
    );
    assert!(benchmark_attempts.iter().any(|attempt| {
        attempt.query == "SearchStrategyFlow frontier rows required evidence coverage"
            && attempt.path_prefix
                == "packages/python/wendao-knowledge-retrieval-benchmark/docs/profile_contract.md"
    }));

    let link_graph_attempts = candidate_discovery_queries(
        "Find the Markdown standard that explains LinkGraph code adaptation and graph search evidence.",
    );
    assert!(link_graph_attempts.iter().any(|attempt| {
        attempt.query == "LinkGraph code adaptation graph search evidence"
            && attempt.path_prefix == "docs/02_dev/standards/LINK_GRAPH_CODE_ADAPTATION.md"
    }));
    assert!(
        !link_graph_attempts
            .iter()
            .any(|attempt| attempt.path_prefix == "AGENTS.md"
                || attempt.path_prefix == "docs/standards/AUDITOR_CODEX.md"),
        "ordinary standard queries must not pull governance/modularity surfaces"
    );

    let polyglot_attempts = candidate_discovery_queries(
        "Find the Markdown RFC and audit for the polyglot compute orchestrator boundary calibration.",
    );
    assert!(polyglot_attempts.iter().any(|attempt| {
        attempt.query == "polyglot compute orchestrator boundary calibration"
            && attempt.path_prefix == "docs/rfcs/2026-05-04-polyglot-compute-orchestrator-rfc.md"
    }));
    assert!(polyglot_attempts.iter().any(|attempt| {
        attempt.query == "polyglot compute orchestrator boundary calibration audit"
            && attempt.path_prefix == "docs/rfcs/2026-05-04-polyglot-compute-orchestrator-audit.md"
    }));
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
