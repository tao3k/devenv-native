use std::fs;

use super::{
    MARKDOWN_HEADING_CANDIDATE_SOURCE, MAX_CANDIDATES,
    discover_search_strategy_flow_candidate_inputs,
    search_strategy_flow_candidate_input_batch_from_markdown,
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
    let receipt: serde_json::Value = serde_json::from_str(&batch.discovery_receipt_json)?;
    assert_eq!(
        receipt.get("transport"),
        Some(&serde_json::json!("local-markdown-scan"))
    );
    assert_eq!(
        receipt.get("candidateInputCount"),
        Some(&serde_json::json!(1))
    );
    Ok(())
}
