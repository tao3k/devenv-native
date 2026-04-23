use super::parse_mermaid_flowchart;
use crate::flowhub::MermaidNodeKind;

#[test]
fn parses_flowchart_with_module_and_scenario_nodes() {
    let parsed = parse_mermaid_flowchart(
        "flowchart LR\n  A[\"coding\"] --> B[\"rust\"]\n  B --> C[\"diagnostics\"]\n",
        "codex-plan",
        &["coding".to_string(), "rust".to_string()],
    )
    .unwrap_or_else(|error| panic!("flowchart should parse: {error}"));

    assert_eq!(parsed.merimind_graph_name, "codex-plan");
    assert_eq!(parsed.direction, "LR");
    assert_eq!(parsed.edges.len(), 2);
    assert_eq!(parsed.nodes.len(), 3);
    assert_eq!(parsed.nodes[0].kind, MermaidNodeKind::Module);
    assert_eq!(parsed.nodes[1].kind, MermaidNodeKind::Module);
    assert_eq!(parsed.nodes[2].kind, MermaidNodeKind::Scenario);
}

#[test]
fn accepts_bare_node_references_after_explicit_labels() {
    let parsed = parse_mermaid_flowchart(
        "flowchart LR\n  A[\"coding\"] --> B[\"rust\"]\n  B --> C[\"blueprint\"]\n  C --> D[\"plan\"]\n",
        "codex-plan",
        &[
            "coding".to_string(),
            "rust".to_string(),
            "blueprint".to_string(),
            "plan".to_string(),
        ],
    )
    .unwrap_or_else(|error| panic!("flowchart should parse: {error}"));

    let rust_node = parsed
        .nodes
        .iter()
        .find(|node| node.id == "B")
        .unwrap_or_else(|| panic!("rust node should exist"));
    assert_eq!(rust_node.label, "rust");
    assert_eq!(rust_node.kind, MermaidNodeKind::Module);
}

#[test]
fn repeated_node_labels_follow_mermaid_last_text_wins_semantics() {
    let parsed = parse_mermaid_flowchart(
        "flowchart LR\n  A[\"coding\"] --> B[\"rust\"]\n  A[\"plan\"] --> C[\"diagnostics\"]\n",
        "codex-plan",
        &["coding".to_string(), "rust".to_string(), "plan".to_string()],
    )
    .unwrap_or_else(|error| panic!("flowchart should accept repeated labels: {error}"));

    let plan_node = parsed
        .nodes
        .iter()
        .find(|node| node.id == "A")
        .unwrap_or_else(|| panic!("node A should exist"));
    assert_eq!(plan_node.label, "plan");
    assert_eq!(plan_node.kind, MermaidNodeKind::Module);
    assert_eq!(parsed.edges.len(), 2);
}

#[test]
fn accepts_presentation_directives_without_private_syntax_stripping() {
    let parsed = parse_mermaid_flowchart(
        "flowchart LR\n  A[\"coding\"] --> B[\"rust\"]\n  B --> C[\"blueprint\"]\n  C --> D[\"plan\"]\n  classDef highlight fill:#f9f,stroke:#333,stroke-width:2px;\n  class A,B highlight;\n  style C fill:#e0f7fa,stroke:#006064;\n  click D \"https://example.com/plan\" \"plan docs\"\n",
        "codex-plan",
        &[
            "coding".to_string(),
            "rust".to_string(),
            "blueprint".to_string(),
            "plan".to_string(),
        ],
    )
    .unwrap_or_else(|error| panic!("presentation directives should not break parse: {error}"));

    assert_eq!(parsed.direction, "LR");
    assert_eq!(parsed.edges.len(), 3);
    assert_eq!(
        parsed
            .nodes
            .iter()
            .map(|node| node.label.as_str())
            .collect::<Vec<_>>(),
        vec!["coding", "rust", "blueprint", "plan"]
    );
    assert!(!parsed.nodes.iter().any(|node| node.label == "highlight"));
    assert!(
        !parsed
            .nodes
            .iter()
            .any(|node| node.label.contains("https://"))
    );
}

#[test]
fn accepts_subgraphs_and_expanded_node_shape_metadata() {
    let parsed = parse_mermaid_flowchart(
        "flowchart TB\n  subgraph DOCS[Docs Search]\n    A@{ shape: rounded, label: \"wendao\" } --> B{{\"diagnostics\"}}\n  end\n  B --> C[\"archive\"]\n",
        "DOC_SEARCH",
        &["wendao".to_string()],
    )
    .unwrap_or_else(|error| panic!("flowchart should parse richer Mermaid syntax: {error}"));

    assert_eq!(parsed.direction, "TB");
    assert_eq!(parsed.edges.len(), 2);
    assert_eq!(parsed.nodes.len(), 3);
    assert_eq!(parsed.nodes[0].label, "wendao");
    assert_eq!(parsed.nodes[0].kind, MermaidNodeKind::Module);
    assert_eq!(parsed.nodes[1].label, "diagnostics");
    assert_eq!(parsed.nodes[2].label, "archive");
}

#[test]
fn accepts_graph_alias_direction_shorthand_and_markdown_labels() {
    let parsed = parse_mermaid_flowchart(
        "graph >\n  A[\"wendao\"] -- \"`**rich** edge`\" --> B(\"`**diagnostics**`\")\n",
        "DOC_SEARCH",
        &["wendao".to_string()],
    )
    .unwrap_or_else(|error| panic!("graph alias and markdown labels should parse: {error}"));

    assert_eq!(parsed.direction, "LR");
    assert_eq!(parsed.edges.len(), 1);
    assert_eq!(parsed.nodes.len(), 2);
    assert_eq!(parsed.nodes[0].label, "wendao");
    assert_eq!(parsed.nodes[0].kind, MermaidNodeKind::Module);
    assert_eq!(parsed.nodes[1].label, "**diagnostics**");
}

#[test]
fn accepts_node_ids_with_dashes_and_pipe_edge_labels() {
    let parsed = parse_mermaid_flowchart(
        "flowchart TD\n  wi-fi[\"wendao\"] -->|Other text| retrieval-node[\"retrieval-context\"]\n",
        "DOC_SEARCH",
        &["wendao".to_string()],
    )
    .unwrap_or_else(|error| panic!("dashed ids and pipe labels should parse: {error}"));

    assert_eq!(parsed.direction, "TB");
    assert_eq!(parsed.edges.len(), 1);
    assert_eq!(parsed.nodes.len(), 2);
    assert_eq!(parsed.nodes[0].id, "wi-fi");
    assert_eq!(parsed.nodes[0].label, "wendao");
    assert_eq!(parsed.nodes[1].id, "retrieval-node");
    assert_eq!(parsed.nodes[1].label, "retrieval-context");
}

#[test]
fn flowchart_without_explicit_direction_defaults_to_top_to_bottom() {
    let parsed = parse_mermaid_flowchart(
        "flowchart\n  A[\"wendao\"] --> B[\"diagnostics\"]\n",
        "DOC_SEARCH",
        &["wendao".to_string()],
    )
    .unwrap_or_else(|error| panic!("flowchart without direction should parse: {error}"));

    assert_eq!(parsed.direction, "TB");
    assert_eq!(parsed.nodes.len(), 2);
    assert_eq!(parsed.edges.len(), 1);
}

#[test]
fn preserves_caller_resolved_graph_name_override() {
    let parsed = parse_mermaid_flowchart(
        "flowchart TB\n  A[\"wendao\"] --> B[\"diagnostics\"]\n",
        "DOC_SEARCH",
        &["wendao".to_string()],
    )
    .unwrap_or_else(|error| panic!("flowchart should preserve caller graph name: {error}"));

    assert_eq!(parsed.merimind_graph_name, "DOC_SEARCH");
    assert_eq!(parsed.direction, "TB");
    assert_eq!(parsed.nodes.len(), 2);
    assert_eq!(parsed.nodes[0].kind, MermaidNodeKind::Module);
    assert_eq!(parsed.nodes[1].kind, MermaidNodeKind::Scenario);
}
