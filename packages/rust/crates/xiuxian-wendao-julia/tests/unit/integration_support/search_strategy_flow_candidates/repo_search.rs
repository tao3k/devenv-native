use super::{
    SearchStrategyFlowRepoSearchHit, search_strategy_flow_candidate_input_from_repo_search_hit,
};

#[test]
fn builds_repo_search_candidate_with_flight_source_edges() {
    let hit = SearchStrategyFlowRepoSearchHit {
        relative_path: "docs/search.md",
        title: Some("Search Strategy"),
        best_section: Some("Query Understanding"),
        line_start: Some(10),
        line_end: Some(14),
        score: Some(0.9),
    };
    let candidate = search_strategy_flow_candidate_input_from_repo_search_hit(&hit);

    assert_eq!(candidate.relative_path, "docs/search.md");
    assert_eq!(candidate.heading_anchor, "query-understanding");
    assert_eq!(candidate.line_start, 10);
    assert_eq!(candidate.line_end, 14);
    assert_eq!(candidate.context_cost, 40);
    assert!(candidate.evidence_coverage > 0.8);
    assert!(candidate.edge_kinds.contains(&"arrow-flight".to_owned()));
    assert!(candidate.edge_kinds.contains(&"repo-search".to_owned()));
    assert!(
        candidate
            .edge_kinds
            .contains(&"native-parser-override".to_owned())
    );
    assert!(
        candidate
            .edge_kinds
            .contains(&"effective-parser:markdown-lang-parser".to_owned())
    );
}

#[test]
fn repo_search_candidates_use_parser_overrides_before_generic_ast() {
    let rust_hit = SearchStrategyFlowRepoSearchHit {
        relative_path: "src/lib.rs",
        title: Some("Rust Route"),
        best_section: None,
        line_start: None,
        line_end: None,
        score: None,
    };
    let julia_hit = SearchStrategyFlowRepoSearchHit {
        relative_path: "src/SearchStrategyFlow.jl",
        title: Some("Julia Route"),
        best_section: None,
        line_start: None,
        line_end: None,
        score: None,
    };
    let typescript_hit = SearchStrategyFlowRepoSearchHit {
        relative_path: "src/app.tsx",
        title: Some("TypeScript Route"),
        best_section: None,
        line_start: None,
        line_end: None,
        score: None,
    };

    let rust_candidate = search_strategy_flow_candidate_input_from_repo_search_hit(&rust_hit);
    let julia_candidate = search_strategy_flow_candidate_input_from_repo_search_hit(&julia_hit);
    let typescript_candidate =
        search_strategy_flow_candidate_input_from_repo_search_hit(&typescript_hit);

    assert!(
        rust_candidate
            .edge_kinds
            .contains(&"effective-parser:rust-lang-parser".to_owned())
    );
    assert!(
        rust_candidate
            .edge_kinds
            .contains(&"baseline-parser:xiuxian-ast:rust".to_owned())
    );
    assert!(
        julia_candidate
            .edge_kinds
            .contains(&"effective-parser:julia-lang-parser".to_owned())
    );
    assert!(
        typescript_candidate
            .edge_kinds
            .contains(&"effective-parser:xiuxian-ast:typescript".to_owned())
    );
    assert!(
        typescript_candidate
            .edge_kinds
            .contains(&"baseline-parser:xiuxian-ast:typescript".to_owned())
    );
}

#[test]
fn repo_search_candidates_cover_flight_index_source_and_config_families() {
    for (path, title, expected_edges) in [
        (
            "packages/rust/crates/xiuxian-wendao/src/link_graph/index/ppr/mod.rs",
            "Rust PPR Search Strategy",
            &[
                "parser-priority:local-override",
                "native-parser-override",
                "effective-parser:rust-lang-parser",
                "baseline-parser:xiuxian-ast:rust",
            ][..],
        ),
        (
            "packages/python/xiuxian-wendao-analyzer/src/xiuxian_wendao_analyzer/worker.py",
            "Python Analyzer Worker",
            &[
                "parser-priority:general-baseline",
                "general-ast-baseline",
                "effective-parser:xiuxian-ast:python",
                "baseline-parser:xiuxian-ast:python",
            ][..],
        ),
        (
            ".data/WendaoGraph.jl/src/reasoning/search_strategy_flow/frontier.jl",
            "Julia Frontier Strategy",
            &[
                "parser-priority:local-override",
                "plugin-parser-override",
                "effective-parser:julia-lang-parser",
            ][..],
        ),
        (
            "wendao.toml",
            "Wendao Repository Configuration",
            &[
                "parser-priority:general-baseline",
                "general-ast-baseline",
                "effective-parser:xiuxian-ast:toml",
                "baseline-parser:xiuxian-ast:toml",
            ][..],
        ),
    ] {
        let candidate = search_strategy_flow_candidate_input_from_repo_search_hit(
            &SearchStrategyFlowRepoSearchHit {
                relative_path: path,
                title: Some(title),
                best_section: None,
                line_start: Some(1),
                line_end: Some(16),
                score: Some(0.82),
            },
        );

        assert_eq!(candidate.relative_path, path);
        assert!(candidate.edge_kinds.contains(&"arrow-flight".to_owned()));
        assert!(candidate.edge_kinds.contains(&"repo-search".to_owned()));
        for expected_edge in expected_edges {
            assert!(
                candidate.edge_kinds.contains(&(*expected_edge).to_owned()),
                "{path} should carry `{expected_edge}`, got {:?}",
                candidate.edge_kinds
            );
        }
    }
}
