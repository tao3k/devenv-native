pub(crate) fn frontier_route_bucket(candidate_id: &str) -> &'static str {
    let normalized = candidate_id.to_ascii_lowercase();
    let (source_path, heading_anchor) = normalized
        .split_once('#')
        .map_or((normalized.as_str(), ""), |(path, anchor)| (path, anchor));

    if !frontier_source_is_code_or_test(source_path)
        && (frontier_candidate_is_authority(heading_anchor)
            || frontier_source_is_authority(source_path, normalized.as_str()))
    {
        return "authority";
    }
    if frontier_candidate_is_validation(heading_anchor) {
        return "validation";
    }
    if frontier_audit_document_is_validation(source_path, heading_anchor) {
        return "validation";
    }
    if let Some(route) = frontier_structural_source_route(source_path) {
        return route;
    }
    if let Some(route) = frontier_explicit_candidate_route(heading_anchor, source_path) {
        return route;
    }
    if frontier_markdown_authority_path(source_path, normalized.as_str()) {
        return "authority";
    }

    "general"
}

fn frontier_structural_source_route(source_path: &str) -> Option<&'static str> {
    if candidate_path_matches(
        source_path,
        &[
            "docs/30_search_strategy",
            "search_strategy_flow",
            "search-strategy-flow",
        ],
    ) {
        return Some("search_strategy");
    }
    if candidate_path_matches(
        source_path,
        &[
            "docs/20_page_index",
            "page_index",
            "pageindex",
            "reasoning_tree",
            "reasoning-tree",
        ],
    ) {
        return Some("page_index");
    }
    if candidate_path_matches(
        source_path,
        &[
            "docs/10_graph_compute",
            "link_graph",
            "link-graph",
            "linkgraph",
            "graph_compute",
            "relation",
        ],
    ) {
        return Some("link_graph");
    }
    if candidate_path_matches(
        source_path,
        &[
            "docs/90_validation",
            "docs/testing",
            "validation",
            "verify",
            "gate",
        ],
    ) {
        return Some("validation");
    }

    None
}

fn frontier_explicit_candidate_route(
    heading_anchor: &str,
    source_path: &str,
) -> Option<&'static str> {
    if frontier_candidate_is_link_graph(heading_anchor) {
        return Some("link_graph");
    }
    if frontier_candidate_is_validation(heading_anchor) {
        return Some("validation");
    }
    if frontier_candidate_is_page_index(heading_anchor) {
        return Some("page_index");
    }
    if frontier_candidate_is_search_strategy(heading_anchor) {
        return Some("search_strategy");
    }
    if candidate_path_matches(
        source_path,
        &[
            "packages/python/wendao-knowledge-retrieval-benchmark/docs/profile_contract.md",
            "packages/python/wendao-knowledge-retrieval-benchmark/docs/architecture.md",
        ],
    ) {
        return Some("validation");
    }
    if candidate_path_matches(
        source_path,
        &[
            "packages/rust/crates/xiuxian-wendao-attachments/readme.md",
            "packages/python/xiuxian-wendao-analyzer/readme.md",
        ],
    ) {
        return Some("link_graph");
    }

    None
}

fn frontier_candidate_is_authority(text: &str) -> bool {
    candidate_contains(
        text,
        &[
            "authority",
            "ownership",
            "owner-boundary",
            "owner-boundaries",
            "ownership-boundary",
            "ownership_boundary",
            "package-owner",
            "source-authority",
            "ssot",
            "single-source-of-truth",
        ],
    )
}

fn frontier_source_is_authority(source_path: &str, candidate_id: &str) -> bool {
    source_path == "docs/rfcs/2026-03-26-wendao-query-engine-rfc.md"
        || source_path == "packages/rust/crates/xiuxian-julia-core/readme.md"
        || source_path.starts_with("packages/rust/crates/xiuxian-julia-core/docs/")
        || source_path == "packages/rust/crates/xiuxian-wendao-studio/readme.md"
        || candidate_id.contains("current-ownership-matrix")
}

fn frontier_candidate_is_validation(text: &str) -> bool {
    candidate_contains(
        text,
        &[
            "validation",
            "validated",
            "validate",
            "verification",
            "verify",
            "verified",
            "gate",
            "package-test",
            "promotion-boundary",
            "proof",
            "test-proof",
            "evidence-calibration",
            "calibration",
            "audit",
            "benchmark",
            "baseline",
            "profile_contract",
            "profile-contract",
            "contract",
            "coverage",
            "quality",
            "fallback",
            "materialization",
            "closing-report",
        ],
    )
}

fn frontier_audit_document_is_validation(source_path: &str, heading_anchor: &str) -> bool {
    is_audit_report_markdown_path(source_path)
        && (heading_anchor.is_empty() || heading_anchor == "document")
}

fn is_audit_report_markdown_path(source_path: &str) -> bool {
    source_path.contains("-audit.")
        || source_path.contains("_audit.")
        || source_path.ends_with("-audit.md")
        || source_path.ends_with("_audit.md")
}

fn frontier_candidate_is_link_graph(text: &str) -> bool {
    candidate_contains(
        text,
        &[
            "link_graph",
            "link-graph",
            "linkgraph",
            "graph_compute",
            "graph-compute",
            "relation",
            "relationship",
            "ppr",
            "fanout",
            "placement",
            "belongs",
        ],
    )
}

fn frontier_candidate_is_page_index(text: &str) -> bool {
    candidate_contains(
        text,
        &[
            "page_index",
            "page-index",
            "pageindex",
            "reasoning_tree",
            "reasoning-tree",
            "document_projection",
            "document-projection",
            "projected-doc",
            "reading-order",
            "section-grounding",
        ],
    )
}

fn frontier_candidate_is_search_strategy(text: &str) -> bool {
    candidate_contains(
        text,
        &[
            "search_strategy",
            "search-strategy",
            "searchstrategyflow",
            "strategy-flow",
            "strategy flow",
        ],
    )
}

fn frontier_markdown_authority_path(source_path: &str, candidate_id: &str) -> bool {
    let path = std::path::Path::new(source_path);
    let is_markdown = path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"));
    if !is_markdown {
        return false;
    }

    source_path == "agents.md"
        || source_path.starts_with("docs/rfcs/")
        || source_path.starts_with("docs/standards/")
        || (source_path.starts_with("docs/developer/")
            && frontier_candidate_is_authority(candidate_id))
        || (source_path.starts_with("packages/")
            && path
                .file_name()
                .is_some_and(|file_name| file_name.eq_ignore_ascii_case("readme.md")))
}

fn frontier_source_is_code_or_test(source_path: &str) -> bool {
    source_path.contains("/tests/")
        || source_path.contains("/test/")
        || source_path.starts_with("tests/")
        || [".rs", ".ts", ".tsx", ".js", ".jsx", ".jl", ".py"]
            .iter()
            .any(|extension| source_path.ends_with(extension))
}

fn candidate_contains(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn candidate_path_matches(source_path: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| source_path.contains(needle))
}
