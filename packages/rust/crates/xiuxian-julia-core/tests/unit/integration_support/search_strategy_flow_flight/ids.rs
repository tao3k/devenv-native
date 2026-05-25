use super::graph_node_display_id_candidates;

#[test]
fn graph_node_display_id_candidates_keep_source_then_markdown_surrogates() {
    assert_eq!(
        graph_node_display_id_candidates("repo", "src/lib.rs"),
        vec![
            "repo/src/lib.rs",
            "src/lib.rs",
            "repo/src/lib.rs.md",
            "src/lib.rs.md",
            "repo/src/lib.md",
            "src/lib.md"
        ]
    );
}

#[test]
fn graph_node_display_id_candidates_do_not_duplicate_markdown_sources() {
    assert_eq!(
        graph_node_display_id_candidates("repo", "docs/search.md"),
        vec!["repo/docs/search.md", "docs/search.md"]
    );
}
