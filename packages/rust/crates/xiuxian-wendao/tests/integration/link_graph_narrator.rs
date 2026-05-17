//! Integration tests for `xiuxian_wendao::link_graph::narrator`.

use xiuxian_wendao::{LinkGraphHit, narrate_subgraph};

#[test]
fn test_narrate_subgraph_empty() {
    assert_eq!(narrate_subgraph(&[]), "");
}

#[test]
fn test_narrate_single_hit() {
    let hit = LinkGraphHit {
        stem: "node_a".to_string(),
        title: "Node A".to_string(),
        path: "a.md".to_string().into(),
        doc_type: None,
        tags: vec!["doc".to_string()],
        score: 1.0,
        best_section: Some(String::new()),
        match_reason: Some("graph_rank>fts".to_string()),
    };
    let output = narrate_subgraph(&[hit]);
    assert!(output.contains("[Concept: Node A]"));
    assert!(output.contains("  Path: a.md"));
    assert!(output.contains("  Score: 1.0000"));
    assert!(output.contains("  Match Reason: graph_rank>fts"));
}
