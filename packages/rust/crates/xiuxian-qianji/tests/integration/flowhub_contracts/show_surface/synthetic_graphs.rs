use super::*;

#[test]
fn show_flowhub_graph_surfaces_unknown_graph_nodes() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let root = create_flowhub_with_undeclared_mermaid_nodes_case(&temp_dir);

    let show = show_flowhub_graph(root.join("plan/codex-plan.mmd"))
        .unwrap_or_else(|error| panic!("Mermaid graph with unknown nodes should show: {error}"));

    assert_eq!(show.unknown_graph_nodes, vec!["style".to_string()]);
    assert!(show.nodes.iter().any(|node| {
        node.label == "style"
            && node.kind.is_none()
            && node.agent_action
                == "do not rely on this node until the Flowhub graph contract is corrected"
    }));
    let rendered = render_flowhub_graph_show(&show);
    assert!(rendered.contains("`style`"));
    assert!(!rendered.contains("Kind: unknown"));
    assert!(
        rendered.contains("do not rely on this node until the Flowhub graph contract is corrected")
    );
    assert!(rendered.contains("Undeclared graph nodes: `style`."));
}

#[test]
fn show_flowhub_graph_preserves_raw_mermaid_but_ignores_presentation_directives_in_semantics() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let root = create_flowhub_with_mermaid_presentation_directives_case(&temp_dir);

    let show = show_flowhub_graph(root.join("plan/codex-plan.mmd")).unwrap_or_else(|error| {
        panic!("Mermaid graph with presentation directives should show: {error}")
    });

    assert_eq!(show.topology, FlowhubGraphTopology::BoundedLoop);
    assert_eq!(
        show.declared_topology,
        Some(FlowhubGraphTopology::BoundedLoop)
    );
    assert!(show.mermaid.contains("classDef highlight"));
    assert!(show.mermaid.contains("style C"));
    assert!(show.mermaid.contains("click G"));
    assert!(show.unknown_graph_nodes.is_empty());
    assert!(!show.nodes.iter().any(|node| node.label == "highlight"));
    assert!(
        !show
            .nodes
            .iter()
            .any(|node| node.label.contains("https://"))
    );
    assert!(
        show.nodes
            .iter()
            .any(|node| node.label == "flowchart alignment")
    );

    let rendered = render_flowhub_graph_show(&show);
    assert!(rendered.contains("classDef highlight"));
    assert!(rendered.contains("style C"));
    assert!(rendered.contains("click G"));
    assert!(!rendered.contains("`highlight`"));
}
