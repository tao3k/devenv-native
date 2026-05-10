use std::fs;

use super::{
    MARKDOWN_HEADING_CANDIDATE_SOURCE, MAX_CANDIDATES, SearchStrategyFlowRepoSearchHit,
    discover_search_strategy_flow_candidate_inputs,
    search_strategy_flow_candidate_input_batch_from_markdown,
    search_strategy_flow_candidate_input_from_repo_search_hit,
};

#[test]
fn discovers_heading_sections_from_real_markdown_shape() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let docs_dir = temp_dir.path().join("docs");
    fs::create_dir_all(&docs_dir)?;
    fs::write(
        docs_dir.join("search.md"),
        "# Search Strategy Flow\n\nIntro.\n\n## Query Understanding\n\nReasoning tree page index links.\n\n## Other\n\nOther text.\n",
    )?;
    fs::write(
        docs_dir.join("unrelated.md"),
        "# Unrelated\n\nDeployment notes only.\n",
    )?;

    let candidates = discover_search_strategy_flow_candidate_inputs(
        "query understanding reasoning tree",
        temp_dir.path(),
    )?;

    let Some(first) = candidates.first() else {
        panic!("expected first candidate");
    };
    assert_eq!(first.relative_path, "docs/search.md");
    assert_eq!(first.heading_anchor, "query-understanding");
    assert!(first.evidence_coverage > 0.8);
    assert!(first.context_cost > 0);
    assert!(first.edge_kinds.contains(&"rust-discovered".to_owned()));
    Ok(())
}

#[test]
fn discovery_preserves_route_diverse_candidates_before_julia_pruning()
-> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let search_dir = temp_dir.path().join("docs/30_search_strategy");
    let page_index_dir = temp_dir.path().join("docs/20_page_index");
    let graph_dir = temp_dir.path().join("docs/10_graph_compute");
    fs::create_dir_all(&search_dir)?;
    fs::create_dir_all(&page_index_dir)?;
    fs::create_dir_all(&graph_dir)?;

    for index in 0..16 {
        fs::write(
            search_dir.join(format!("search_{index:02}.md")),
            format!(
                "# SearchStrategyFlow Query Understanding {index}\n\nSearchStrategyFlow intent strategy flow query understanding branch pruning.\n",
            ),
        )?;
    }
    fs::write(
        page_index_dir.join("reasoning_tree.md"),
        "# PageIndex Parent Child Evidence\n\nPageIndex reasoning tree parent child section spans and disclosure frontier.\n",
    )?;
    fs::write(
        graph_dir.join("link_graph.md"),
        "# LinkGraph Relation Fanout\n\nLinkGraph relation fanout connects section anchors and provenance edges.\n",
    )?;
    fs::write(
        temp_dir.path().join("docs/index.md"),
        "# Documentation Index\n\nSearchStrategyFlow PageIndex LinkGraph relation path index.\n",
    )?;

    let candidates = discover_search_strategy_flow_candidate_inputs(
        "SearchStrategyFlow PageIndex LinkGraph relation path",
        temp_dir.path(),
    )?;

    assert_eq!(candidates.len(), MAX_CANDIDATES);
    assert!(candidates.iter().any(|candidate| {
        candidate
            .relative_path
            .starts_with("docs/30_search_strategy/")
    }));
    assert!(
        candidates
            .iter()
            .any(|candidate| candidate.relative_path.starts_with("docs/20_page_index/"))
    );
    assert!(candidates.iter().any(|candidate| {
        candidate
            .relative_path
            .starts_with("docs/10_graph_compute/")
    }));
    Ok(())
}

#[test]
fn serializes_tsv_without_losing_candidate_boundaries() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    fs::write(
        temp_dir.path().join("doc.md"),
        "# Query\tUnderstanding\n\nLine one.\nLine two.\n",
    )?;

    let batch = search_strategy_flow_candidate_input_batch_from_markdown(
        "query understanding",
        temp_dir.path(),
    )?;

    assert_eq!(batch.source, MARKDOWN_HEADING_CANDIDATE_SOURCE);
    assert_eq!(batch.row_count, 1);
    assert!(batch.tsv.contains("doc.md"));
    assert!(batch.tsv.contains("Query\\tUnderstanding"));
    assert_eq!(batch.tsv.lines().count(), 1);
    Ok(())
}

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
