use super::support::{
    make_graph_fixture, markdown_analysis_response_for_symbol,
    markdown_analysis_response_with_section_graph, push_ui_config_from_toml,
    sorted_graph_links_payload, sorted_graph_nodes_payload,
};
use super::*;

#[test]
fn graph_neighbors_from_markdown_analysis_returns_graph_payload() {
    let analysis = markdown_analysis_response_with_section_graph();
    let response = graph_neighbors_from_markdown_analysis(&analysis);

    assert_studio_json_snapshot(
        "graph_neighbors_markdown_analysis_payload",
        json!({
            "center": {
                "id": response.center.id,
                "label": response.center.label,
                "path": response.center.path,
                "nodeType": response.center.node_type,
                "isCenter": response.center.is_center,
                "distance": response.center.distance,
            },
            "nodes": sorted_graph_nodes_payload(response.nodes),
            "links": sorted_graph_links_payload(response.links),
            "totalNodes": response.total_nodes,
            "totalLinks": response.total_links,
        }),
    );
}

#[tokio::test]
async fn graph_neighbors_markdown_symbol_uses_shared_definition_resolution() {
    let fixture = make_graph_fixture(vec![
        (
            "docs/index.md",
            "# Index\n\nObserve `AlphaService` from the runtime notes.\n",
        ),
        (
            "packages/rust/crates/demo/src/service.rs",
            "pub struct AlphaService;\n",
        ),
        (
            "packages/rust/crates/other/src/service.rs",
            "pub struct AlphaService;\n",
        ),
    ]);
    push_ui_config_from_toml(
        &fixture,
        r#"
[link_graph.projects.kernel]
root = "."
dirs = ["docs", "packages"]
"#,
    );

    let mut response = graph_neighbors_from_markdown_analysis(
        &crate::gateway::studio::types::MarkdownAnalysisResponse {
            path: "docs/index.md".to_string(),
            document_hash: "doc-hash".to_string(),
            node_count: 3,
            edge_count: 2,
            nodes: vec![
                crate::gateway::studio::types::AnalysisNode {
                    id: "docs/index.md#document".to_string(),
                    kind: crate::gateway::studio::types::AnalysisNodeKind::Document,
                    label: "index.md".to_string(),
                    depth: 0,
                    line_start: 1,
                    line_end: 3,
                    parent_id: None,
                },
                crate::gateway::studio::types::AnalysisNode {
                    id: "docs/index.md#observe:3:observe".to_string(),
                    kind: crate::gateway::studio::types::AnalysisNodeKind::Observation,
                    label:
                        ":OBSERVE: lang:rust scope:\"packages/rust/crates/other/**\" \"AlphaService\""
                            .to_string(),
                    depth: 1,
                    line_start: 3,
                    line_end: 3,
                    parent_id: Some("docs/index.md#document".to_string()),
                },
                crate::gateway::studio::types::AnalysisNode {
                    id: "docs/index.md#symbol:alphaservice".to_string(),
                    kind: crate::gateway::studio::types::AnalysisNodeKind::Symbol,
                    label: "AlphaService".to_string(),
                    depth: 2,
                    line_start: 3,
                    line_end: 3,
                    parent_id: Some("docs/index.md#observe:3:observe".to_string()),
                },
            ],
            edges: vec![
                crate::gateway::studio::types::AnalysisEdge {
                    id: "e1".to_string(),
                    kind: crate::gateway::studio::types::AnalysisEdgeKind::Contains,
                    source_id: "docs/index.md#document".to_string(),
                    target_id: "docs/index.md#observe:3:observe".to_string(),
                    label: Some("contains".to_string()),
                    evidence: crate::gateway::studio::types::AnalysisEvidence {
                        path: "docs/index.md".to_string(),
                        line_start: 3,
                        line_end: 3,
                        confidence: 1.0,
                    },
                },
                crate::gateway::studio::types::AnalysisEdge {
                    id: "e2".to_string(),
                    kind: crate::gateway::studio::types::AnalysisEdgeKind::References,
                    source_id: "docs/index.md#observe:3:observe".to_string(),
                    target_id: "docs/index.md#symbol:alphaservice".to_string(),
                    label: Some("AlphaService".to_string()),
                    evidence: crate::gateway::studio::types::AnalysisEvidence {
                        path: "docs/index.md".to_string(),
                        line_start: 3,
                        line_end: 3,
                        confidence: 1.0,
                    },
                },
            ],
            projections: Vec::new(),
            diagnostics: Vec::new(),
        },
    );
    let result = decorate_markdown_graph_navigation(fixture.state.as_ref(), &mut response).await;
    let Ok(()) = result else {
        panic!("expected markdown graph navigation decoration to succeed");
    };

    let Some(symbol_node) = response
        .nodes
        .iter()
        .find(|node| node.label == "AlphaService" && node.id.contains("symbol:"))
    else {
        panic!("expected markdown graph payload to include observation symbol node");
    };

    assert_eq!(
        symbol_node
            .navigation_target
            .as_ref()
            .map(|target| target.path.as_str()),
        Some("packages/rust/crates/other/src/service.rs")
    );
    assert_eq!(
        symbol_node
            .navigation_target
            .as_ref()
            .and_then(|target| target.line),
        Some(1)
    );
    assert_eq!(
        symbol_node
            .navigation_target
            .as_ref()
            .and_then(|target| target.column),
        Some(1)
    );
}

#[tokio::test]
async fn graph_neighbors_markdown_symbol_prefers_observe_language_hint() {
    let fixture = make_graph_fixture(vec![
        (
            "docs/index.md",
            "# Index\n\nObserve `AlphaService` from the runtime notes.\n",
        ),
        (
            "packages/rust/crates/demo/src/service.rs",
            "pub struct AlphaService;\n",
        ),
        (
            "packages/python/demo/service.py",
            "class AlphaService:\n    pass\n",
        ),
    ]);
    push_ui_config_from_toml(
        &fixture,
        r#"
[link_graph.projects.kernel]
root = "."
dirs = ["docs", "packages"]
"#,
    );

    let analysis = markdown_analysis_response_for_symbol(":OBSERVE: lang:python \"AlphaService\"");
    let mut response = graph_neighbors_from_markdown_analysis(&analysis);
    let result = decorate_markdown_graph_navigation(fixture.state.as_ref(), &mut response).await;
    let Ok(()) = result else {
        panic!("expected markdown graph navigation decoration to succeed");
    };

    let Some(symbol_node) = response
        .nodes
        .iter()
        .find(|node| node.label == "AlphaService" && node.id.contains("symbol:"))
    else {
        panic!("expected markdown graph payload to include observation symbol node");
    };

    assert_eq!(
        symbol_node
            .navigation_target
            .as_ref()
            .map(|target| target.path.as_str()),
        Some("packages/python/demo/service.py")
    );
    assert_eq!(
        symbol_node
            .navigation_target
            .as_ref()
            .and_then(|target| target.line),
        Some(1)
    );
    assert_eq!(
        symbol_node
            .navigation_target
            .as_ref()
            .and_then(|target| target.column),
        Some(1)
    );
}
